use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::items::{ItemResponse, ItemsListResponse, ListItemsQuery};
use crate::errors::AppError;
use crate::traits;

/// Allowed sort columns (whitelist to prevent SQL injection).
const ALLOWED_SORTS: &[&str] = &["created_at", "priority", "name"];

/// Allowed order directions.
const ALLOWED_ORDERS: &[&str] = &["asc", "desc"];

/// Default and max per_page.
const DEFAULT_PER_PAGE: i64 = 50;
const MAX_PER_PAGE: i64 = 100;

/// Maximum page number to prevent huge OFFSET scans.
const MAX_PAGE: i64 = 1000;

/// Redis cache TTL for list responses (seconds).
const LIST_CACHE_TTL_SECS: i64 = 300;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgItemService {
    pool: PgPool,
    item_repo: Arc<dyn traits::ItemRepo>,
    redis: redis::Client,
}

impl PgItemService {
    pub fn new(pool: PgPool, item_repo: Arc<dyn traits::ItemRepo>, redis: redis::Client) -> Self {
        Self {
            pool,
            item_repo,
            redis,
        }
    }

    /// Increment the version counter for a user's items cache. Returns the new version.
    async fn bump_version(&self, user_id: Uuid) -> Option<i64> {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            tracing::warn!(%user_id, "redis unavailable – cannot bump cache version");
            return None;
        };
        let ver_key = format!("items:{user_id}:ver");
        redis::cmd("INCR")
            .arg(&ver_key)
            .query_async::<i64>(&mut conn)
            .await
            .ok()
    }

    /// Get the current version counter for a user's items cache.
    async fn get_version(&self, user_id: Uuid) -> Option<i64> {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            tracing::warn!(%user_id, "redis unavailable – skipping cache read");
            return None;
        };
        let ver_key = format!("items:{user_id}:ver");
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

    /// Validate that a category_id belongs to the given user.
    async fn validate_category_ownership(
        &self,
        category_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1 AND user_id = $2)",
        )
        .bind(category_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        if !exists {
            return Err(AppError::BadRequest(
                "category not found or does not belong to user".into(),
            ));
        }
        Ok(())
    }
}

/// Hash the query parameters to create a unique cache key suffix.
fn hash_query(query: &ListItemsQuery, sort: &str, order: &str, page: i64, per_page: i64) -> u64 {
    let mut hasher = DefaultHasher::new();
    query.status.hash(&mut hasher);
    query.category_id.hash(&mut hasher);
    sort.hash(&mut hasher);
    order.hash(&mut hasher);
    page.hash(&mut hasher);
    per_page.hash(&mut hasher);
    hasher.finish()
}

#[async_trait]
impl traits::ItemService for PgItemService {
    #[tracing::instrument(skip(self))]
    async fn create_item(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<Decimal>,
        priority: Option<i16>,
        category_id: Option<Uuid>,
    ) -> Result<ItemResponse, AppError> {
        let priority = priority.unwrap_or(2);
        if !(1..=3).contains(&priority) {
            return Err(AppError::BadRequest(
                "priority must be between 1 and 3".into(),
            ));
        }

        if let Some(price) = estimated_price
            && price < Decimal::ZERO
        {
            return Err(AppError::BadRequest("estimated_price must be >= 0".into()));
        }

        if let Some(cid) = category_id {
            self.validate_category_ownership(cid, user_id).await?;
        }

        let item = self
            .item_repo
            .create(
                user_id,
                name,
                description,
                url,
                estimated_price,
                priority,
                category_id,
            )
            .await
            .map_err(AppError::Internal)?;

        // Invalidate list cache
        self.bump_version(user_id).await;

        Ok(ItemResponse::from(item))
    }

    #[tracing::instrument(skip(self))]
    async fn get_item(&self, id: Uuid, user_id: Uuid) -> Result<ItemResponse, AppError> {
        let item = self
            .item_repo
            .find_by_id(id, user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("item not found".into()))?;

        Ok(ItemResponse::from(item))
    }

    #[tracing::instrument(skip(self))]
    async fn list_items(
        &self,
        user_id: Uuid,
        query: &ListItemsQuery,
    ) -> Result<ItemsListResponse, AppError> {
        // Validate & normalize query params
        let sort = query.sort.as_deref().unwrap_or("created_at");
        if !ALLOWED_SORTS.contains(&sort) {
            return Err(AppError::BadRequest(format!(
                "invalid sort field: {sort}. Allowed: {}",
                ALLOWED_SORTS.join(", ")
            )));
        }

        let order = query.order.as_deref().unwrap_or("desc");
        if !ALLOWED_ORDERS.contains(&order) {
            return Err(AppError::BadRequest(format!(
                "invalid order: {order}. Allowed: asc, desc"
            )));
        }

        let per_page = query
            .per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE);
        let page = query.page.unwrap_or(1).max(1);
        if page > MAX_PAGE {
            return Err(AppError::BadRequest(format!(
                "page must be at most {MAX_PAGE}"
            )));
        }
        let offset = (page - 1) * per_page;

        if let Some(ref s) = query.status
            && s != "active"
            && s != "purchased"
        {
            return Err(AppError::BadRequest(
                "status filter must be 'active' or 'purchased'".into(),
            ));
        }

        // Try Redis cache (fail-open). Treat missing version key as 0 so the
        // cache can be hit even before the first mutation.
        let ver = self.get_version(user_id).await.unwrap_or(0);
        let query_hash = hash_query(query, sort, order, page, per_page);

        let cache_key = format!("items:{user_id}:v{ver}:{query_hash}");
        if let Some(cached) = self.get_cached_list(&cache_key).await
            && let Ok(resp) = serde_json::from_str::<ItemsListResponse>(&cached)
        {
            return Ok(resp);
        }

        // Cache miss → query DB
        let status_filter = query.status.as_deref();
        let (items, total) = tokio::try_join!(
            async {
                self.item_repo
                    .list(
                        user_id,
                        status_filter,
                        query.category_id,
                        sort,
                        order,
                        per_page,
                        offset,
                    )
                    .await
                    .map_err(AppError::Internal)
            },
            async {
                self.item_repo
                    .count(user_id, status_filter, query.category_id)
                    .await
                    .map_err(AppError::Internal)
            },
        )?;

        let response = ItemsListResponse {
            items: items.into_iter().map(ItemResponse::from).collect(),
            total,
            page,
            per_page,
        };

        // Cache the result (fail-open) — reuse the same cache_key from the read path
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.set_cached_list(&cache_key, &serialized).await;
        }

        Ok(response)
    }

    #[tracing::instrument(skip(self))]
    async fn update_item(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<Decimal>,
        priority: Option<i16>,
        category_id: Option<Option<Uuid>>,
        status: Option<&str>,
    ) -> Result<ItemResponse, AppError> {
        if let Some(p) = priority
            && !(1..=3).contains(&p)
        {
            return Err(AppError::BadRequest(
                "priority must be between 1 and 3".into(),
            ));
        }

        if let Some(price) = estimated_price
            && price < Decimal::ZERO
        {
            return Err(AppError::BadRequest("estimated_price must be >= 0".into()));
        }

        if let Some(s) = status
            && s != "active"
            && s != "purchased"
        {
            return Err(AppError::BadRequest(
                "status must be 'active' or 'purchased'".into(),
            ));
        }

        if let Some(Some(cid)) = category_id {
            self.validate_category_ownership(cid, user_id).await?;
        }

        // Check at least one field is being updated
        let has_update = name.is_some()
            || description.is_some()
            || url.is_some()
            || estimated_price.is_some()
            || priority.is_some()
            || category_id.is_some()
            || status.is_some();

        if !has_update {
            // Nothing to update — just return the current item
            return self.get_item(id, user_id).await;
        }

        let item = self
            .item_repo
            .update(
                id,
                user_id,
                name,
                description,
                url,
                estimated_price,
                priority,
                category_id,
                status,
            )
            .await
            .map_err(AppError::Internal)?;

        // When status is provided, the repo adds `AND status != $new_status`
        // to the WHERE clause. Zero rows can mean either "not found" or
        // "already has that status". Disambiguate only on the error path.
        let item = match item {
            Some(item) => item,
            None if status.is_some() => {
                if self
                    .item_repo
                    .find_by_id(id, user_id)
                    .await
                    .map_err(AppError::Internal)?
                    .is_some()
                {
                    return Err(AppError::Conflict(format!(
                        "item is already '{}'",
                        status.unwrap()
                    )));
                }
                return Err(AppError::NotFound("item not found".into()));
            }
            None => return Err(AppError::NotFound("item not found".into())),
        };

        // Invalidate list cache
        self.bump_version(user_id).await;

        Ok(ItemResponse::from(item))
    }

    #[tracing::instrument(skip(self))]
    async fn delete_item(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self
            .item_repo
            .soft_delete(id, user_id)
            .await
            .map_err(AppError::Internal)?;

        if !deleted {
            return Err(AppError::NotFound("item not found".into()));
        }

        // Invalidate list cache
        self.bump_version(user_id).await;

        Ok(())
    }
}
