use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::categories::CategoryResponse;
use crate::errors::AppError;
use crate::traits;

/// Redis cache TTL for list responses (seconds).
const LIST_CACHE_TTL_SECS: i64 = 300;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCategoryService {
    category_repo: Arc<dyn traits::CategoryRepo>,
    redis: redis::Client,
}

/// Check if a sqlx database error is a unique-constraint violation (PG code 23505).
fn is_unique_violation(err: &anyhow::Error) -> bool {
    err.downcast_ref::<sqlx::Error>()
        .and_then(|e| match e {
            sqlx::Error::Database(db_err) => db_err.code().map(|c| c == "23505"),
            _ => None,
        })
        .unwrap_or(false)
}

impl PgCategoryService {
    pub fn new(category_repo: Arc<dyn traits::CategoryRepo>, redis: redis::Client) -> Self {
        Self {
            category_repo,
            redis,
        }
    }

    /// Increment the version counter for a user's categories cache.
    async fn bump_version(&self, user_id: Uuid) {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            tracing::warn!(%user_id, "redis unavailable – cannot bump categories cache version");
            return;
        };
        let ver_key = format!("categories:{user_id}:ver");
        let _: Result<i64, _> = redis::cmd("INCR")
            .arg(&ver_key)
            .query_async(&mut conn)
            .await;
    }

    /// Also bump items cache version (needed when deleting a category due to ON DELETE SET NULL).
    async fn bump_items_version(&self, user_id: Uuid) {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            tracing::warn!(%user_id, "redis unavailable – cannot bump items cache version");
            return;
        };
        let ver_key = format!("items:{user_id}:ver");
        let _: Result<i64, _> = redis::cmd("INCR")
            .arg(&ver_key)
            .query_async(&mut conn)
            .await;
    }

    /// Get the current version counter for a user's categories cache.
    async fn get_version(&self, user_id: Uuid) -> Option<i64> {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            tracing::warn!(%user_id, "redis unavailable – skipping cache read");
            return None;
        };
        let ver_key = format!("categories:{user_id}:ver");
        redis::cmd("GET")
            .arg(&ver_key)
            .query_async::<Option<i64>>(&mut conn)
            .await
            .ok()
            .flatten()
    }

    /// Try to get a cached list response.
    async fn get_cached_list(&self, cache_key: &str) -> Option<String> {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            tracing::warn!(cache_key, "redis unavailable – cache miss by default");
            return None;
        };
        redis::cmd("GET")
            .arg(cache_key)
            .query_async::<Option<String>>(&mut conn)
            .await
            .ok()
            .flatten()
    }

    /// Cache a list response.
    async fn set_cached_list(&self, cache_key: &str, value: &str) {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            tracing::warn!(cache_key, "redis unavailable – cannot cache list response");
            return;
        };
        let _: Result<(), _> = redis::cmd("SET")
            .arg(cache_key)
            .arg(value)
            .arg("EX")
            .arg(LIST_CACHE_TTL_SECS)
            .query_async(&mut conn)
            .await;
    }
}

#[async_trait]
impl traits::CategoryService for PgCategoryService {
    #[tracing::instrument(skip(self))]
    async fn list_categories(&self, user_id: Uuid) -> Result<Vec<CategoryResponse>, AppError> {
        // Try Redis cache (fail-open)
        let ver = self.get_version(user_id).await.unwrap_or(0);
        let cache_key = format!("categories:{user_id}:v{ver}");

        if let Some(cached) = self.get_cached_list(&cache_key).await
            && let Ok(resp) = serde_json::from_str::<Vec<CategoryResponse>>(&cached)
        {
            return Ok(resp);
        }

        // Cache miss → query DB
        let cats = self
            .category_repo
            .list_by_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        let response: Vec<CategoryResponse> =
            cats.into_iter().map(CategoryResponse::from).collect();

        // Cache the result (fail-open)
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.set_cached_list(&cache_key, &serialized).await;
        }

        Ok(response)
    }

    #[tracing::instrument(skip(self))]
    async fn create_category(
        &self,
        user_id: Uuid,
        name: &str,
        icon: Option<&str>,
    ) -> Result<CategoryResponse, AppError> {
        let cat = self
            .category_repo
            .create(user_id, name, icon)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    AppError::Conflict("a category with this name already exists".into())
                } else {
                    AppError::Internal(e)
                }
            })?;

        self.bump_version(user_id).await;

        Ok(CategoryResponse::from(cat))
    }

    #[tracing::instrument(skip(self))]
    async fn update_category(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        icon: Option<&str>,
        position: Option<i32>,
    ) -> Result<CategoryResponse, AppError> {
        if let Some(p) = position
            && p < 0
        {
            return Err(AppError::BadRequest("position must be >= 0".into()));
        }

        let has_update = name.is_some() || icon.is_some() || position.is_some();

        if !has_update {
            // Nothing to update — return current category
            let cat = self
                .category_repo
                .find_by_id(id, user_id)
                .await
                .map_err(AppError::Internal)?
                .ok_or_else(|| AppError::NotFound("category not found".into()))?;
            return Ok(CategoryResponse::from(cat));
        }

        let cat = self
            .category_repo
            .update(id, user_id, name, icon, position)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    AppError::Conflict("a category with this name already exists".into())
                } else {
                    AppError::Internal(e)
                }
            })?
            .ok_or_else(|| AppError::NotFound("category not found".into()))?;

        self.bump_version(user_id).await;

        Ok(CategoryResponse::from(cat))
    }

    #[tracing::instrument(skip(self))]
    async fn delete_category(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        // Check existence and is_default before deleting
        let cat = self
            .category_repo
            .find_by_id(id, user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("category not found".into()))?;

        if cat.is_default {
            return Err(AppError::BadRequest(
                "cannot delete a default category".into(),
            ));
        }

        let deleted = self
            .category_repo
            .delete(id, user_id)
            .await
            .map_err(AppError::Internal)?;

        if !deleted {
            return Err(AppError::NotFound("category not found".into()));
        }

        // Invalidate both categories and items caches
        // (ON DELETE SET NULL modifies items silently)
        self.bump_version(user_id).await;
        self.bump_items_version(user_id).await;

        Ok(())
    }
}
