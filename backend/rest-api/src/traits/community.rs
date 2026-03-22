use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::pagination::PaginatedResponse;
use crate::errors::AppError;
use crate::models::community_wish::WishStatus;
use crate::models::{CommunityWish, WishMessage, WishReport};
use crate::services::moderation_service::ModerationResult;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait CommunityWishRepo: Send + Sync {
    async fn create(
        &self,
        owner_id: Uuid,
        title: &str,
        description: Option<&str>,
        category: &str,
        is_anonymous: bool,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<CommunityWish>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CommunityWish>>;

    async fn list_open(
        &self,
        category: Option<&str>,
        limit: i64,
        offset: i64,
        caller_id: Option<Uuid>,
    ) -> Result<Vec<CommunityWish>>;

    async fn count_open(&self, category: Option<&str>, caller_id: Option<Uuid>) -> Result<i64>;

    async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<CommunityWish>>;

    async fn list_by_owner_paginated(
        &self,
        owner_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommunityWish>>;

    async fn count_by_owner(&self, owner_id: Uuid) -> Result<i64>;

    async fn list_by_donor(&self, donor_id: Uuid) -> Result<Vec<CommunityWish>>;

    async fn list_by_donor_paginated(
        &self,
        donor_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommunityWish>>;

    async fn count_by_donor(&self, donor_id: Uuid) -> Result<i64>;

    async fn count_active_by_owner(&self, owner_id: Uuid) -> Result<i64>;

    async fn update_status(
        &self,
        id: Uuid,
        status: WishStatus,
        moderation_note: Option<&str>,
    ) -> Result<bool>;

    async fn set_matched(
        &self,
        id: Uuid,
        donor_id: Uuid,
        matched_at: DateTime<Utc>,
    ) -> Result<bool>;

    async fn clear_match(&self, id: Uuid) -> Result<bool>;

    async fn set_fulfilled(&self, id: Uuid, fulfilled_at: DateTime<Utc>) -> Result<bool>;

    async fn set_closed(&self, id: Uuid, closed_at: DateTime<Utc>) -> Result<bool>;

    async fn increment_report_count(&self, id: Uuid) -> Result<i32>;

    async fn reset_reports(&self, id: Uuid) -> Result<bool>;

    async fn increment_reopen_count(&self, id: Uuid, now: DateTime<Utc>) -> Result<i32>;

    async fn update_content(
        &self,
        id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        category: Option<&str>,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<Option<CommunityWish>>;

    async fn list_flagged(&self, limit: i64, offset: i64) -> Result<Vec<CommunityWish>>;

    async fn count_flagged(&self) -> Result<i64>;

    async fn list_recent_fulfilled(&self, limit: i64) -> Result<Vec<CommunityWish>>;

    async fn delete(&self, id: Uuid) -> Result<bool>;
}

#[async_trait]
pub trait WishReportRepo: Send + Sync {
    async fn create(
        &self,
        wish_id: Uuid,
        reporter_id: Uuid,
        reason: &str,
        details: Option<&str>,
    ) -> Result<WishReport>;

    async fn has_reported(&self, wish_id: Uuid, reporter_id: Uuid) -> Result<bool>;

    async fn count_by_reporter_today(&self, reporter_id: Uuid) -> Result<i64>;

    async fn delete_by_wish(&self, wish_id: Uuid) -> Result<u64>;

    async fn reported_wish_ids(
        &self,
        wish_ids: &[Uuid],
        reporter_id: Uuid,
    ) -> Result<std::collections::HashSet<Uuid>>;
}

#[async_trait]
pub trait WishMessageRepo: Send + Sync {
    async fn create(&self, wish_id: Uuid, sender_id: Uuid, body: &str) -> Result<WishMessage>;

    async fn list_by_wish(
        &self,
        wish_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WishMessage>>;

    async fn count_by_wish(&self, wish_id: Uuid) -> Result<i64>;
}

// ── Moderation trait ────────────────────────────────────────────────

#[async_trait]
pub trait ModerationService: Send + Sync {
    async fn check_wish(
        &self,
        title: &str,
        description: Option<&str>,
        category: &str,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<ModerationResult, anyhow::Error>;
}

// ── Community wish service traits ───────────────────────────────────

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait CommunityWishService: Send + Sync {
    async fn create_wish(
        &self,
        user_id: Uuid,
        title: &str,
        description: Option<&str>,
        category: &str,
        is_anonymous: bool,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<crate::dto::community_wishes::MyWishResponse, AppError>;

    async fn list_wishes(
        &self,
        caller_id: Option<Uuid>,
        category: Option<&str>,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<crate::dto::community_wishes::WishResponse>, AppError>;

    async fn get_wish(
        &self,
        wish_id: Uuid,
        caller_id: Option<Uuid>,
    ) -> Result<crate::dto::community_wishes::WishDetailResponse, AppError>;

    async fn list_my_wishes(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<crate::dto::community_wishes::MyWishResponse>, i64), AppError>;

    async fn list_my_offers(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<crate::dto::community_wishes::WishResponse>, i64), AppError>;

    async fn list_recent_fulfilled(
        &self,
    ) -> Result<Vec<crate::dto::community_wishes::WishResponse>, AppError>;

    async fn update_wish(
        &self,
        wish_id: Uuid,
        user_id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        category: Option<&str>,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<crate::dto::community_wishes::MyWishResponse, AppError>;

    async fn close_wish(&self, wish_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn delete_wish(&self, wish_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn reopen_wish(&self, wish_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn offer_wish(&self, wish_id: Uuid, donor_id: Uuid) -> Result<(), AppError>;

    async fn withdraw_offer(&self, wish_id: Uuid, donor_id: Uuid) -> Result<(), AppError>;

    async fn reject_offer(&self, wish_id: Uuid, owner_id: Uuid) -> Result<(), AppError>;

    async fn confirm_wish(&self, wish_id: Uuid, owner_id: Uuid) -> Result<(), AppError>;

    async fn report_wish(
        &self,
        wish_id: Uuid,
        reporter_id: Uuid,
        reason: &str,
        details: Option<&str>,
    ) -> Result<(), AppError>;

    async fn block_wish(&self, wish_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn unblock_wish(&self, wish_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn admin_list_flagged(
        &self,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<crate::dto::community_wishes::AdminWishResponse>, AppError>;

    async fn admin_approve(&self, wish_id: Uuid) -> Result<(), AppError>;

    async fn admin_reject(&self, wish_id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait WishMessageService: Send + Sync {
    async fn send_message(
        &self,
        wish_id: Uuid,
        sender_id: Uuid,
        body: &str,
    ) -> Result<crate::dto::wish_messages::MessageResponse, AppError>;

    async fn list_messages(
        &self,
        wish_id: Uuid,
        user_id: Uuid,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<crate::dto::wish_messages::MessageResponse>, AppError>;
}
