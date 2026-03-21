use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::share_links::{
    ShareLinkListItem, ShareLinkResponse, SharedViewResponse, UpdateShareLinkRequest,
};
use crate::errors::AppError;
use crate::models::{CircleEvent, ShareLink};

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ShareLinkRepo: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: Option<DateTime<Utc>>,
        label: Option<&str>,
        permissions: &str,
        scope: &str,
        scope_data: Option<&serde_json::Value>,
    ) -> Result<ShareLink>;

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<ShareLink>>;

    async fn list_by_user_paginated(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ShareLink>>;

    async fn count_by_user(&self, user_id: Uuid) -> Result<i64>;

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<ShareLink>>;

    async fn find_by_token(&self, token: &str) -> Result<Option<ShareLink>>;

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool>;

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        label: Option<&str>,
        is_active: Option<bool>,
        permissions: Option<&str>,
        expires_at: Option<Option<DateTime<Utc>>>,
    ) -> Result<Option<ShareLink>>;
}

#[async_trait]
pub trait ShareLinkService: Send + Sync {
    async fn create_share_link(
        &self,
        user_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
        label: Option<&str>,
        permissions: Option<&str>,
        scope: Option<&str>,
        scope_data: Option<&serde_json::Value>,
    ) -> Result<ShareLinkResponse, AppError>;

    async fn list_share_links(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ShareLinkListItem>, i64), AppError>;

    async fn delete_share_link(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn get_shared_view(&self, token: &str) -> Result<SharedViewResponse, AppError>;

    async fn update_share_link(
        &self,
        id: Uuid,
        user_id: Uuid,
        req: &UpdateShareLinkRequest,
    ) -> Result<ShareLinkResponse, AppError>;

    async fn claim_via_share(
        &self,
        token: &str,
        item_id: Uuid,
        claimer_id: Uuid,
    ) -> Result<(), AppError>;

    async fn unclaim_via_share(
        &self,
        token: &str,
        item_id: Uuid,
        claimer_id: Uuid,
    ) -> Result<(), AppError>;

    /// Web claim via share link (no auth required).
    async fn web_claim_via_share(
        &self,
        token: &str,
        item_id: Uuid,
        name: &str,
    ) -> Result<Uuid, AppError>;

    /// Web unclaim via share link (no auth required, uses web_claim_token).
    async fn web_unclaim_via_share(
        &self,
        token: &str,
        item_id: Uuid,
        web_claim_token: Uuid,
    ) -> Result<(), AppError>;
}

// ── Upload service ──────────────────────────────────────────────────

#[async_trait]
pub trait UploadService: Send + Sync {
    /// Upload an image, process it, store in R2, return the public CDN URL.
    /// `upload_type`: "item" (800px), "avatar" (400px square), "circle" (400px square)
    async fn upload_image(
        &self,
        data: &[u8],
        content_type: &str,
        upload_type: &str,
    ) -> Result<String, AppError>;

    /// Delete an image from R2 by its public URL.
    async fn delete_image(&self, url: &str) -> Result<(), AppError>;
}

// ── Health trait ─────────────────────────────────────────────────────

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_db(&self) -> bool;
    async fn check_cache(&self) -> bool;
}

// ── Circle event repo ───────────────────────────────────────────────

#[async_trait]
pub trait CircleEventRepo: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    async fn insert(
        &self,
        circle_id: Uuid,
        actor_id: Uuid,
        event_type: &str,
        target_item_id: Option<Uuid>,
        target_user_id: Option<Uuid>,
    ) -> Result<CircleEvent>;

    async fn list_by_circle(
        &self,
        circle_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CircleEvent>>;

    async fn count_by_circle(&self, circle_id: Uuid) -> Result<i64>;
}
