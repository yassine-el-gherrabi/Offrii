use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::items::ItemResponse;
use crate::dto::share_links::{ShareLinkListItem, ShareLinkResponse, SharedViewResponse};
use crate::errors::AppError;
use crate::models::ShareLink;
use crate::repositories::share_link_repo;
use crate::traits;

/// Maximum number of share links per user.
const MAX_LINKS_PER_USER: i64 = 10;

/// Maximum number of items returned in a shared view.
const MAX_SHARED_VIEW_ITEMS: i64 = 500;

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
        return Err(AppError::NotFound("share link has expired".into()));
    }
    Ok(())
}

#[async_trait]
impl traits::ShareLinkService for PgShareLinkService {
    #[tracing::instrument(skip(self))]
    async fn create_share_link(
        &self,
        user_id: Uuid,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<ShareLinkResponse, AppError> {
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

        let link = share_link_repo::create(&mut *tx, user_id, &token, expires_at)
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

        check_link_expiry(&link)?;

        // Get owner info
        let user = self
            .user_repo
            .find_by_id(link.user_id)
            .await
            .map_err(AppError::Internal)?;

        let username = user.map(|u| u.username).unwrap_or_default();

        // Get active items (not deleted, status = active)
        let items = self
            .item_repo
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
            .map_err(AppError::Internal)?;

        let items: Vec<ItemResponse> = items.into_iter().map(ItemResponse::from).collect();

        Ok(SharedViewResponse {
            user_username: username,
            items,
        })
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

        check_link_expiry(&link)?;

        // Verify item belongs to the share link owner
        let item = self
            .item_repo
            .find_by_id(item_id, link.user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("item not found".into()))?;

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

        check_link_expiry(&link)?;

        // Verify item belongs to the share link owner
        self.item_repo
            .find_by_id(item_id, link.user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("item not found".into()))?;

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
