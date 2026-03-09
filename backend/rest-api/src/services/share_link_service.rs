use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::items::ItemResponse;
use crate::dto::share_links::{
    ShareLinkListItem, ShareLinkResponse, SharedViewResponse, UpdateShareLinkRequest,
};
use crate::errors::AppError;
use crate::models::ShareLink;
use crate::repositories::share_link_repo;
use crate::traits;

/// Maximum number of share links per user.
const MAX_LINKS_PER_USER: i64 = 10;

/// Maximum number of items returned in a shared view.
const MAX_SHARED_VIEW_ITEMS: i64 = 500;

/// Maximum number of item IDs allowed in scope=selection.
const MAX_SELECTION_ITEMS: usize = 100;

const VALID_PERMISSIONS: &[&str] = &["view_only", "view_and_claim"];
const VALID_SCOPES: &[&str] = &["all", "category", "selection"];

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgShareLinkService {
    pool: PgPool,
    share_link_repo: Arc<dyn traits::ShareLinkRepo>,
    item_repo: Arc<dyn traits::ItemRepo>,
    user_repo: Arc<dyn traits::UserRepo>,
    app_base_url: String,
}

impl PgShareLinkService {
    pub fn new(
        pool: PgPool,
        share_link_repo: Arc<dyn traits::ShareLinkRepo>,
        item_repo: Arc<dyn traits::ItemRepo>,
        user_repo: Arc<dyn traits::UserRepo>,
        app_base_url: String,
    ) -> Self {
        Self {
            pool,
            share_link_repo,
            item_repo,
            user_repo,
            app_base_url,
        }
    }
}

fn generate_token() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// Returns an error if the share link has expired.
fn check_link_expiry(link: &ShareLink) -> Result<(), AppError> {
    if let Some(expires_at) = link.expires_at
        && expires_at < Utc::now()
    {
        return Err(AppError::Gone("share link has expired".into()));
    }
    Ok(())
}

/// Returns an error if the share link is deactivated.
fn check_link_active(link: &ShareLink) -> Result<(), AppError> {
    if !link.is_active {
        return Err(AppError::Gone(
            "this share link has been deactivated".into(),
        ));
    }
    Ok(())
}

/// Validate scope_data against the scope value.
fn validate_scope_data(
    scope: &str,
    scope_data: Option<&serde_json::Value>,
) -> Result<(), AppError> {
    match scope {
        "all" => {
            if scope_data.is_some() {
                return Err(AppError::BadRequest(
                    "scope_data must be null when scope is 'all'".into(),
                ));
            }
        }
        "category" => {
            let data = scope_data.ok_or_else(|| {
                AppError::BadRequest("scope_data is required for scope 'category'".into())
            })?;
            let cat_id = data.get("category_id").and_then(|v| v.as_str());
            if cat_id.is_none() {
                return Err(AppError::BadRequest(
                    "scope_data.category_id is required for scope 'category'".into(),
                ));
            }
            // Validate it's a UUID
            cat_id.unwrap().parse::<Uuid>().map_err(|_| {
                AppError::BadRequest("scope_data.category_id must be a valid UUID".into())
            })?;
        }
        "selection" => {
            let data = scope_data.ok_or_else(|| {
                AppError::BadRequest("scope_data is required for scope 'selection'".into())
            })?;
            let item_ids = data.get("item_ids").and_then(|v| v.as_array());
            match item_ids {
                None => {
                    return Err(AppError::BadRequest(
                        "scope_data.item_ids is required for scope 'selection'".into(),
                    ));
                }
                Some(ids) if ids.is_empty() => {
                    return Err(AppError::BadRequest(
                        "scope_data.item_ids must not be empty".into(),
                    ));
                }
                Some(ids) if ids.len() > MAX_SELECTION_ITEMS => {
                    return Err(AppError::BadRequest(format!(
                        "scope_data.item_ids must contain at most {MAX_SELECTION_ITEMS} items"
                    )));
                }
                Some(ids) => {
                    // Validate each is a UUID string
                    for id_val in ids {
                        let s = id_val.as_str().ok_or_else(|| {
                            AppError::BadRequest(
                                "scope_data.item_ids must be an array of UUID strings".into(),
                            )
                        })?;
                        s.parse::<Uuid>().map_err(|_| {
                            AppError::BadRequest(format!(
                                "invalid UUID in scope_data.item_ids: {s}"
                            ))
                        })?;
                    }
                }
            }
        }
        _ => {
            return Err(AppError::BadRequest(
                "invalid scope: must be one of 'all', 'category', 'selection'".into(),
            ));
        }
    }
    Ok(())
}

/// Extract category_id from scope_data.
fn extract_category_id(scope_data: &serde_json::Value) -> Result<Uuid, AppError> {
    scope_data["category_id"]
        .as_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("scope_data missing category_id")))?
        .parse::<Uuid>()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt category_id in scope_data: {e}")))
}

/// Extract item_ids from scope_data.
fn extract_item_ids(scope_data: &serde_json::Value) -> Result<Vec<Uuid>, AppError> {
    scope_data["item_ids"]
        .as_array()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("scope_data missing item_ids")))?
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!("non-string element in scope_data.item_ids"))
                })?
                .parse::<Uuid>()
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("corrupt UUID in scope_data.item_ids: {e}"))
                })
        })
        .collect()
}

/// Check if an item is within the scope of a share link.
fn item_in_scope(link: &ShareLink, item: &crate::models::Item) -> Result<(), AppError> {
    match link.scope.as_str() {
        "all" => Ok(()),
        "category" => {
            let data = link.scope_data.as_ref().ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("scope_data missing for category scope"))
            })?;
            let cat_id = extract_category_id(data)?;
            if item.category_id == Some(cat_id) {
                Ok(())
            } else {
                Err(AppError::NotFound("item not found".into()))
            }
        }
        "selection" => {
            let data = link.scope_data.as_ref().ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("scope_data missing for selection scope"))
            })?;
            let ids = extract_item_ids(data)?;
            if ids.contains(&item.id) {
                Ok(())
            } else {
                Err(AppError::NotFound("item not found".into()))
            }
        }
        _ => Err(AppError::NotFound("item not found".into())),
    }
}

/// Fetch items filtered by the link's scope.
async fn fetch_scoped_items(
    item_repo: &dyn traits::ItemRepo,
    link: &ShareLink,
) -> Result<Vec<crate::models::Item>, AppError> {
    match link.scope.as_str() {
        "all" => item_repo
            .list(
                link.user_id,
                Some("active"),
                None,
                "created_at",
                "desc",
                MAX_SHARED_VIEW_ITEMS,
                0,
            )
            .await
            .map_err(AppError::Internal),
        "category" => {
            let data = link.scope_data.as_ref().ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("scope_data missing for category scope"))
            })?;
            let cat_id = extract_category_id(data)?;
            item_repo
                .list(
                    link.user_id,
                    Some("active"),
                    Some(cat_id),
                    "created_at",
                    "desc",
                    MAX_SHARED_VIEW_ITEMS,
                    0,
                )
                .await
                .map_err(AppError::Internal)
        }
        "selection" => {
            let data = link.scope_data.as_ref().ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("scope_data missing for selection scope"))
            })?;
            let ids = extract_item_ids(data)?;
            item_repo
                .find_by_ids(link.user_id, &ids)
                .await
                .map_err(AppError::Internal)
        }
        _ => Ok(vec![]),
    }
}

#[async_trait]
impl traits::ShareLinkService for PgShareLinkService {
    #[tracing::instrument(skip(self))]
    async fn create_share_link(
        &self,
        user_id: Uuid,
        expires_at: Option<chrono::DateTime<Utc>>,
        label: Option<&str>,
        permissions: Option<&str>,
        scope: Option<&str>,
        scope_data: Option<&serde_json::Value>,
    ) -> Result<ShareLinkResponse, AppError> {
        let permissions = permissions.unwrap_or("view_and_claim");
        let scope = scope.unwrap_or("all");

        // Validate permissions
        if !VALID_PERMISSIONS.contains(&permissions) {
            return Err(AppError::BadRequest(
                "permissions must be 'view_only' or 'view_and_claim'".into(),
            ));
        }

        // Validate scope
        if !VALID_SCOPES.contains(&scope) {
            return Err(AppError::BadRequest(
                "scope must be 'all', 'category', or 'selection'".into(),
            ));
        }

        // Validate label length
        if let Some(l) = label
            && l.len() > 100
        {
            return Err(AppError::BadRequest(
                "label must be at most 100 characters".into(),
            ));
        }

        // Validate scope_data
        validate_scope_data(scope, scope_data)?;

        // Validate scope_data references belong to the user
        match scope {
            "category" => {
                let data = scope_data.ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!(
                        "scope_data missing after validation for category scope"
                    ))
                })?;
                let cat_id = extract_category_id(data)?;
                let _ = self
                    .item_repo
                    .list(
                        user_id,
                        Some("active"),
                        Some(cat_id),
                        "created_at",
                        "desc",
                        1,
                        0,
                    )
                    .await
                    .map_err(AppError::Internal)?;
            }
            "selection" => {
                let data = scope_data.ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!(
                        "scope_data missing after validation for selection scope"
                    ))
                })?;
                let ids = extract_item_ids(data)?;
                let found_items = self
                    .item_repo
                    .find_by_ids(user_id, &ids)
                    .await
                    .map_err(AppError::Internal)?;
                if found_items.len() != ids.len() {
                    return Err(AppError::BadRequest(
                        "some item IDs do not belong to you or do not exist".into(),
                    ));
                }
            }
            _ => {}
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Lock the user row to serialise concurrent share-link creation
        sqlx::query("SELECT 1 FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Count existing links inside the same transaction
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM share_links WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if count >= MAX_LINKS_PER_USER {
            return Err(AppError::BadRequest(format!(
                "maximum of {MAX_LINKS_PER_USER} share links allowed"
            )));
        }

        let token = generate_token();

        let link = share_link_repo::create(
            &mut *tx,
            user_id,
            &token,
            expires_at,
            label,
            permissions,
            scope,
            scope_data,
        )
        .await
        .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(ShareLinkResponse::from_model(link, &self.app_base_url))
    }

    #[tracing::instrument(skip(self))]
    async fn list_share_links(&self, user_id: Uuid) -> Result<Vec<ShareLinkListItem>, AppError> {
        let links = self
            .share_link_repo
            .list_by_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(links.into_iter().map(ShareLinkListItem::from).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn delete_share_link(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self
            .share_link_repo
            .delete(id, user_id)
            .await
            .map_err(AppError::Internal)?;

        if !deleted {
            return Err(AppError::NotFound("share link not found".into()));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_shared_view(&self, token: &str) -> Result<SharedViewResponse, AppError> {
        let link = self
            .share_link_repo
            .find_by_token(token)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("share link not found".into()))?;

        check_link_active(&link)?;
        check_link_expiry(&link)?;

        // Get owner info
        let user = self
            .user_repo
            .find_by_id(link.user_id)
            .await
            .map_err(AppError::Internal)?;

        let username = user.map(|u| u.username).unwrap_or_default();

        // Get items filtered by scope
        let items = fetch_scoped_items(self.item_repo.as_ref(), &link).await?;

        let items: Vec<ItemResponse> = items.into_iter().map(ItemResponse::from).collect();

        Ok(SharedViewResponse {
            user_username: username,
            permissions: link.permissions,
            items,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn update_share_link(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: &UpdateShareLinkRequest,
    ) -> Result<ShareLinkResponse, AppError> {
        // Reject empty PATCH body
        if req.label.is_none()
            && req.is_active.is_none()
            && req.permissions.is_none()
            && req.expires_at.is_none()
        {
            return Err(AppError::BadRequest(
                "at least one field must be provided".into(),
            ));
        }

        // Validate permissions if provided
        if let Some(ref p) = req.permissions
            && !VALID_PERMISSIONS.contains(&p.as_str())
        {
            return Err(AppError::BadRequest(
                "permissions must be 'view_only' or 'view_and_claim'".into(),
            ));
        }

        // Validate label length if provided
        if let Some(ref l) = req.label
            && l.len() > 100
        {
            return Err(AppError::BadRequest(
                "label must be at most 100 characters".into(),
            ));
        }

        let link = self
            .share_link_repo
            .update(
                id,
                user_id,
                req.label.as_deref(),
                req.is_active,
                req.permissions.as_deref(),
                req.expires_at,
            )
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("share link not found".into()))?;

        Ok(ShareLinkResponse::from_model(link, &self.app_base_url))
    }

    #[tracing::instrument(skip(self))]
    async fn claim_via_share(
        &self,
        token: &str,
        item_id: Uuid,
        claimer_id: Uuid,
    ) -> Result<(), AppError> {
        let link = self
            .share_link_repo
            .find_by_token(token)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("share link not found".into()))?;

        check_link_active(&link)?;
        check_link_expiry(&link)?;

        // Check permissions
        if link.permissions != "view_and_claim" {
            return Err(AppError::Forbidden(
                "this share link does not allow claiming items".into(),
            ));
        }

        // Verify item belongs to the share link owner
        let item = self
            .item_repo
            .find_by_id(item_id, link.user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("item not found".into()))?;

        // Check item is in scope (includes category check)
        item_in_scope(&link, &item)?;

        // Cannot claim your own item
        if item.user_id == claimer_id {
            return Err(AppError::BadRequest("cannot claim your own item".into()));
        }

        let owner_id = self
            .item_repo
            .claim_item(item_id, claimer_id)
            .await
            .map_err(AppError::Internal)?;

        if owner_id.is_none() {
            // Disambiguate
            let item = self
                .item_repo
                .find_by_id_any_user(item_id)
                .await
                .map_err(AppError::Internal)?;

            match item {
                None => return Err(AppError::NotFound("item not found".into())),
                Some(i) if i.claimed_by.is_some() => {
                    return Err(AppError::Conflict("item already claimed".into()));
                }
                Some(_) => return Err(AppError::BadRequest("item is not active".into())),
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn unclaim_via_share(
        &self,
        token: &str,
        item_id: Uuid,
        claimer_id: Uuid,
    ) -> Result<(), AppError> {
        let link = self
            .share_link_repo
            .find_by_token(token)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("share link not found".into()))?;

        check_link_active(&link)?;
        check_link_expiry(&link)?;

        // Check permissions
        if link.permissions != "view_and_claim" {
            return Err(AppError::Forbidden(
                "this share link does not allow claiming items".into(),
            ));
        }

        // Verify item belongs to the share link owner
        let item = self
            .item_repo
            .find_by_id(item_id, link.user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("item not found".into()))?;

        // Check item is in scope (includes category check)
        item_in_scope(&link, &item)?;

        let owner_id = self
            .item_repo
            .unclaim_item(item_id, claimer_id)
            .await
            .map_err(AppError::Internal)?;

        if owner_id.is_none() {
            match self
                .item_repo
                .find_by_id_any_user(item_id)
                .await
                .map_err(AppError::Internal)?
            {
                None => return Err(AppError::NotFound("item not found".into())),
                Some(i) if i.claimed_by.is_none() => {
                    return Err(AppError::Conflict("item is not claimed".into()));
                }
                Some(i) if i.claimed_by != Some(claimer_id) => {
                    return Err(AppError::Unauthorized(
                        "only the claimer can unclaim".into(),
                    ));
                }
                Some(_) => return Err(AppError::NotFound("item not found".into())),
            }
        }

        Ok(())
    }
}
