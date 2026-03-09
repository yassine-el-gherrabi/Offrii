use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use uuid::Uuid;

use crate::dto::items::ItemResponse;
use crate::dto::share_links::{ShareLinkListItem, ShareLinkResponse, SharedViewResponse};
use crate::errors::AppError;
use crate::traits;

/// Maximum number of share links per user.
const MAX_LINKS_PER_USER: usize = 10;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgShareLinkService {
    share_link_repo: Arc<dyn traits::ShareLinkRepo>,
    item_repo: Arc<dyn traits::ItemRepo>,
    user_repo: Arc<dyn traits::UserRepo>,
    app_base_url: String,
}

impl PgShareLinkService {
    pub fn new(
        share_link_repo: Arc<dyn traits::ShareLinkRepo>,
        item_repo: Arc<dyn traits::ItemRepo>,
        user_repo: Arc<dyn traits::UserRepo>,
        app_base_url: String,
    ) -> Self {
        Self {
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

#[async_trait]
impl traits::ShareLinkService for PgShareLinkService {
    #[tracing::instrument(skip(self))]
    async fn create_share_link(
        &self,
        user_id: Uuid,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<ShareLinkResponse, AppError> {
        // Check limit
        let existing = self
            .share_link_repo
            .list_by_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        if existing.len() >= MAX_LINKS_PER_USER {
            return Err(AppError::BadRequest(format!(
                "maximum of {MAX_LINKS_PER_USER} share links allowed"
            )));
        }

        let token = generate_token();

        let link = self
            .share_link_repo
            .create(user_id, &token, expires_at)
            .await
            .map_err(AppError::Internal)?;

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

        // Check expiry
        if let Some(expires_at) = link.expires_at
            && expires_at < Utc::now()
        {
            return Err(AppError::NotFound("share link has expired".into()));
        }

        // Get owner info
        let user = self
            .user_repo
            .find_by_id(link.user_id)
            .await
            .map_err(AppError::Internal)?;

        let display_name = user.and_then(|u| u.display_name);

        // Get active items (not deleted, status = active)
        let items = self
            .item_repo
            .list(
                link.user_id,
                Some("active"),
                None,
                "created_at",
                "desc",
                100,
                0,
            )
            .await
            .map_err(AppError::Internal)?;

        let items: Vec<ItemResponse> = items.into_iter().map(ItemResponse::from).collect();

        Ok(SharedViewResponse {
            user_display_name: display_name,
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

        // Check expiry
        if let Some(expires_at) = link.expires_at
            && expires_at < Utc::now()
        {
            return Err(AppError::NotFound("share link has expired".into()));
        }

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

        // Check expiry
        if let Some(expires_at) = link.expires_at
            && expires_at < Utc::now()
        {
            return Err(AppError::NotFound("share link has expired".into()));
        }

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
