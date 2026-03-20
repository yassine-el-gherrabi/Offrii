use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::items::{ItemResponse, ListItemsQuery, SharedCircleInfo};
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::errors::AppError;
use crate::traits;

/// Allowed sort columns (whitelist to prevent SQL injection).
const ALLOWED_SORTS: &[&str] = &["created_at", "priority", "name"];

/// Allowed order directions.
const ALLOWED_ORDERS: &[&str] = &["asc", "desc"];

/// Maximum page number to prevent huge OFFSET scans.
const MAX_PAGE: i64 = 1000;

/// Redis cache TTL for list responses (seconds).
const LIST_CACHE_TTL_SECS: i64 = 300;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgItemService {
    pool: PgPool,
    item_repo: Arc<dyn traits::ItemRepo>,
    circle_item_repo: Arc<dyn traits::CircleItemRepo>,
    redis: redis::Client,
}

impl PgItemService {
    pub fn new(
        pool: PgPool,
        item_repo: Arc<dyn traits::ItemRepo>,
        circle_item_repo: Arc<dyn traits::CircleItemRepo>,
        redis: redis::Client,
    ) -> Self {
        Self {
            pool,
            item_repo,
            circle_item_repo,
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

    /// Validate that a category_id exists (categories are global).
    async fn validate_category_exists(&self, category_id: Uuid) -> Result<(), AppError> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1)")
                .bind(category_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

        if !exists {
            return Err(AppError::BadRequest("category not found".into()));
        }
        Ok(())
    }

    /// Enrich a list of ItemResponses with shared circle info.
    async fn enrich_with_circles(&self, items: &mut [ItemResponse], user_id: Uuid) {
        if items.is_empty() {
            return;
        }
        let item_ids: Vec<Uuid> = items.iter().map(|i| i.id).collect();

        // 1. Explicit circle_items shares (selection mode)
        let circle_map = self
            .circle_item_repo
            .list_circle_names_for_items(&item_ids)
            .await
            .unwrap_or_default();

        for item in items.iter_mut() {
            if let Some(circles) = circle_map.get(&item.id) {
                item.shared_circles = circles
                    .iter()
                    .map(|(id, name, is_direct, image_url)| SharedCircleInfo {
                        id: *id,
                        name: name.clone(),
                        is_direct: *is_direct,
                        image_url: image_url.clone(),
                    })
                    .collect();
            }
        }

        // 2. Rule-based shares (all/categories modes) — add circles not already present
        #[allow(clippy::type_complexity)]
        let rule_circles: Vec<(Uuid, String, bool, Option<String>, String, Vec<Uuid>)> =
            sqlx::query_as(
                "SELECT c.id, \
                   CASE WHEN c.name IS NOT NULL THEN c.name \
                        ELSE COALESCE(u.display_name, u.username, '') \
                   END AS name, \
                   c.is_direct, c.image_url, \
                   r.share_mode, r.category_ids \
                 FROM circle_share_rules r \
                 JOIN circles c ON c.id = r.circle_id \
                 LEFT JOIN circle_members cm ON cm.circle_id = c.id AND cm.user_id != r.user_id AND c.is_direct = true \
                 LEFT JOIN users u ON u.id = cm.user_id \
                 WHERE r.user_id = $1 AND r.share_mode IN ('all', 'categories')",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        for item in items.iter_mut() {
            if item.is_private {
                continue; // private items never shared via rules
            }
            for (cid, cname, is_direct, img, mode, cat_ids) in &rule_circles {
                // Skip if already added via circle_items
                if item.shared_circles.iter().any(|sc| sc.id == *cid) {
                    continue;
                }
                let matches = match mode.as_str() {
                    "all" => true,
                    "categories" => {
                        if let Some(item_cat) = item.category_id {
                            cat_ids.contains(&item_cat)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                if matches {
                    item.shared_circles.push(SharedCircleInfo {
                        id: *cid,
                        name: cname.clone(),
                        is_direct: *is_direct,
                        image_url: img.clone(),
                    });
                }
            }
        }
    }
}

/// Hash the query parameters to create a unique cache key suffix.
fn hash_query(query: &ListItemsQuery, sort: &str, order: &str, page: i64, limit: i64) -> u64 {
    let mut hasher = DefaultHasher::new();
    query.status.hash(&mut hasher);
    query.category_id.hash(&mut hasher);
    sort.hash(&mut hasher);
    order.hash(&mut hasher);
    page.hash(&mut hasher);
    limit.hash(&mut hasher);
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
        image_url: Option<&str>,
        links: Option<&[String]>,
        is_private: Option<bool>,
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
            self.validate_category_exists(cid).await?;
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
                image_url,
                links,
                is_private.unwrap_or(false),
            )
            .await
            .map_err(AppError::Internal)?;

        // Invalidate list cache
        self.bump_version(user_id).await;

        // Async OG fetch for the first link
        if let Some(links) = links
            && let Some(first_link) = links.first()
        {
            let item_repo = self.item_repo.clone();
            let link = first_link.clone();
            let item_id = item.id;
            tracing::info!(item_id = %item_id, link = %link, "OG fetch starting");
            tokio::spawn(async move {
                match crate::services::og_service::fetch_og_metadata(&link).await {
                    Ok(og) => {
                        tracing::info!(
                            item_id = %item_id,
                            og_image = ?og.image_url,
                            og_title = ?og.title,
                            "OG fetch succeeded"
                        );
                        let _ = item_repo
                            .update_og_metadata(
                                item_id,
                                og.image_url.as_deref(),
                                og.title.as_deref(),
                                og.site_name.as_deref(),
                            )
                            .await;
                    }
                    Err(e) => {
                        tracing::warn!(item_id = %item_id, error = %e, "OG fetch failed");
                    }
                }
            });
        }

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

        let mut resp = ItemResponse::from(item);
        self.enrich_with_circles(std::slice::from_mut(&mut resp), user_id)
            .await;
        Ok(resp)
    }

    #[tracing::instrument(skip(self))]
    async fn list_items(
        &self,
        user_id: Uuid,
        query: &ListItemsQuery,
    ) -> Result<PaginatedResponse<ItemResponse>, AppError> {
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

        let (page, limit, offset) = normalize_pagination(query.page, query.limit);
        if page > MAX_PAGE {
            return Err(AppError::BadRequest(format!(
                "page must be at most {MAX_PAGE}"
            )));
        }

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
        let query_hash = hash_query(query, sort, order, page, limit);

        let cache_key = format!("items:{user_id}:v{ver}:{query_hash}");
        if let Some(cached) = self.get_cached_list(&cache_key).await
            && let Ok(resp) = serde_json::from_str::<PaginatedResponse<ItemResponse>>(&cached)
        {
            return Ok(resp);
        }

        // Cache miss → query DB
        let status_filter = query.status.as_deref();
        let cat_ids = query.resolved_category_ids();
        let cat_ids_ref = cat_ids.as_deref();
        let (items, total) = tokio::try_join!(
            async {
                self.item_repo
                    .list(
                        user_id,
                        status_filter,
                        cat_ids_ref,
                        sort,
                        order,
                        limit,
                        offset,
                    )
                    .await
                    .map_err(AppError::Internal)
            },
            async {
                self.item_repo
                    .count(user_id, status_filter, cat_ids_ref)
                    .await
                    .map_err(AppError::Internal)
            },
        )?;

        let mut item_responses: Vec<ItemResponse> =
            items.into_iter().map(ItemResponse::from).collect();
        self.enrich_with_circles(&mut item_responses, user_id).await;

        let response = PaginatedResponse::new(item_responses, total, page, limit);

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
        image_url: Option<Option<&str>>,
        links: Option<&[String]>,
        is_private: Option<bool>,
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
            self.validate_category_exists(cid).await?;
        }

        // Check at least one field is being updated
        let has_update = name.is_some()
            || description.is_some()
            || url.is_some()
            || estimated_price.is_some()
            || priority.is_some()
            || category_id.is_some()
            || status.is_some()
            || image_url.is_some()
            || links.is_some()
            || is_private.is_some();

        if !has_update {
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
                image_url,
                links,
                is_private,
            )
            .await
            .map_err(AppError::Internal)?;

        let item = match item {
            Some(item) => item,
            None => {
                if let Some(new_status) = status
                    && self
                        .item_repo
                        .find_by_id(id, user_id)
                        .await
                        .map_err(AppError::Internal)?
                        .is_some()
                {
                    return Err(AppError::Conflict(format!(
                        "item is already '{new_status}'"
                    )));
                }
                return Err(AppError::NotFound("item not found".into()));
            }
        };

        // When an item is set to private, remove all circle_items shares
        if is_private == Some(true) {
            sqlx::query("DELETE FROM circle_items WHERE item_id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
            tracing::info!(item_id = %id, "removed all circle_items shares (item set to private)");
        }

        // Invalidate list cache
        self.bump_version(user_id).await;

        // Async OG fetch for the first link (if links changed)
        if let Some(links) = links
            && let Some(first_link) = links.first()
        {
            let item_repo = self.item_repo.clone();
            let link = first_link.clone();
            let item_id = item.id;
            tracing::info!(item_id = %item_id, link = %link, "OG fetch starting (update)");
            tokio::spawn(async move {
                match crate::services::og_service::fetch_og_metadata(&link).await {
                    Ok(og) => {
                        tracing::info!(
                            item_id = %item_id,
                            og_image = ?og.image_url,
                            og_title = ?og.title,
                            "OG fetch succeeded (update)"
                        );
                        let _ = item_repo
                            .update_og_metadata(
                                item_id,
                                og.image_url.as_deref(),
                                og.title.as_deref(),
                                og.site_name.as_deref(),
                            )
                            .await;
                    }
                    Err(e) => {
                        tracing::warn!(item_id = %item_id, error = %e, "OG fetch failed (update)");
                    }
                }
            });
        }

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

    #[tracing::instrument(skip(self))]
    async fn claim_item(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError> {
        let owner_id = self
            .item_repo
            .claim_item(item_id, claimer_id)
            .await
            .map_err(AppError::Internal)?;

        if let Some(owner_id) = owner_id {
            self.bump_version(owner_id).await;
            return Ok(());
        }

        // Disambiguate: why did 0 rows match?
        match self
            .item_repo
            .find_by_id_any_user(item_id)
            .await
            .map_err(AppError::Internal)?
        {
            None => Err(AppError::NotFound("item not found".into())),
            Some(item) if item.user_id == claimer_id => {
                Err(AppError::BadRequest("cannot claim your own item".into()))
            }
            Some(item) if item.claimed_by.is_some() || item.claimed_via.is_some() => {
                Err(AppError::Conflict("item already claimed".into()))
            }
            Some(_) => Err(AppError::BadRequest("item is not active".into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn unclaim_item(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError> {
        let owner_id = self
            .item_repo
            .unclaim_item(item_id, claimer_id)
            .await
            .map_err(AppError::Internal)?;

        if let Some(owner_id) = owner_id {
            self.bump_version(owner_id).await;
            return Ok(());
        }

        // Disambiguate: why did 0 rows match?
        match self
            .item_repo
            .find_by_id_any_user(item_id)
            .await
            .map_err(AppError::Internal)?
        {
            None => Err(AppError::NotFound("item not found".into())),
            Some(item) if item.claimed_by.is_none() && item.claimed_via.is_none() => {
                Err(AppError::Conflict("item is not claimed".into()))
            }
            Some(item) if item.claimed_by != Some(claimer_id) => Err(AppError::Unauthorized(
                "only the claimer can unclaim".into(),
            )),
            Some(_) => Err(AppError::NotFound("item not found".into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn owner_unclaim_web_item(&self, item_id: Uuid, owner_id: Uuid) -> Result<(), AppError> {
        let result = self
            .item_repo
            .owner_unclaim_web_item(item_id, owner_id)
            .await
            .map_err(AppError::Internal)?;

        if result.is_some() {
            self.bump_version(owner_id).await;
            return Ok(());
        }

        // Disambiguate
        match self
            .item_repo
            .find_by_id(item_id, owner_id)
            .await
            .map_err(AppError::Internal)?
        {
            None => Err(AppError::NotFound("item not found".into())),
            Some(item) if item.claimed_via.as_deref() == Some("app") => Err(AppError::Forbidden(
                "cannot remove an app claim — only the claimer can unclaim".into(),
            )),
            Some(item) if item.claimed_via.is_none() => {
                Err(AppError::Conflict("item is not claimed".into()))
            }
            Some(_) => Err(AppError::NotFound("item not found".into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn batch_delete_items(&self, ids: &[Uuid], user_id: Uuid) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Err(AppError::BadRequest("ids must not be empty".into()));
        }
        if ids.len() > 100 {
            return Err(AppError::BadRequest(
                "cannot delete more than 100 items at once".into(),
            ));
        }

        // Batch soft-delete via single SQL query
        let result = sqlx::query(
            "UPDATE items SET status = 'deleted' WHERE id = ANY($1) AND user_id = $2 AND status != 'deleted'"
        )
        .bind(ids)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        let count = result.rows_affected();

        if count > 0 {
            self.bump_version(user_id).await;
        }

        Ok(count)
    }
}
