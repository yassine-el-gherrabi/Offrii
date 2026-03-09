use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse};
use crate::dto::categories::CategoryResponse;
use crate::dto::circles::{
    CircleDetailResponse, CircleItemResponse, CircleResponse, FeedResponse, InviteResponse,
    JoinResponse,
};
use crate::dto::items::{ItemResponse, ItemsListResponse, ListItemsQuery};
use crate::dto::push_tokens::PushTokenResponse;
use crate::dto::share_links::{
    ShareLinkListItem, ShareLinkResponse, SharedViewResponse, UpdateShareLinkRequest,
};
use crate::dto::users::{UserDataExport, UserProfileResponse};
use crate::errors::AppError;
use crate::models::{
    Category, Circle, CircleEvent, CircleInvite, CircleItem, CircleMember, Item, PushToken,
    RefreshToken, ShareLink, User,
};

// ── Repository traits ────────────────────────────────────────────────

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        username: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<User>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;

    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;

    async fn is_username_taken(
        &self,
        username: &str,
        exclude_user_id: Option<Uuid>,
    ) -> Result<bool>;

    #[allow(clippy::too_many_arguments)]
    async fn update_profile(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        username: Option<&str>,
        reminder_freq: Option<&str>,
        reminder_time: Option<NaiveTime>,
        timezone: Option<&str>,
        utc_reminder_hour: Option<i16>,
        locale: Option<&str>,
    ) -> Result<Option<User>>;

    async fn delete_user(&self, id: Uuid) -> Result<bool>;

    async fn find_eligible_for_reminder(&self, utc_hour: i16) -> Result<Vec<User>>;

    async fn update_password_hash(&self, id: Uuid, password_hash: &str) -> Result<bool>;

    async fn increment_token_version(&self, id: Uuid) -> Result<i32>;
}

#[async_trait]
pub trait RefreshTokenRepo: Send + Sync {
    async fn insert(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken>;

    async fn revoke_by_hash(&self, token_hash: &str) -> Result<bool>;

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<()>;

    async fn revoke_excess_for_user(&self, user_id: Uuid, keep: i64) -> Result<()>;
}

#[async_trait]
pub trait CategoryRepo: Send + Sync {
    async fn copy_defaults_for_user(&self, user_id: Uuid) -> Result<u64>;

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Category>>;

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Category>>;

    async fn create(&self, user_id: Uuid, name: &str, icon: Option<&str>) -> Result<Category>;

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        icon: Option<&str>,
        position: Option<i32>,
    ) -> Result<Option<Category>>;

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool>;
}

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ItemRepo: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: i16,
        category_id: Option<Uuid>,
    ) -> Result<Item>;

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Item>>;

    async fn list(
        &self,
        user_id: Uuid,
        status: Option<&str>,
        category_id: Option<Uuid>,
        sort: &str,
        order: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Item>>;

    async fn count(
        &self,
        user_id: Uuid,
        status: Option<&str>,
        category_id: Option<Uuid>,
    ) -> Result<i64>;

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: Option<i16>,
        category_id: Option<Option<Uuid>>,
        status: Option<&str>,
    ) -> Result<Option<Item>>;

    async fn soft_delete(&self, id: Uuid, user_id: Uuid) -> Result<bool>;

    async fn find_active_older_than(
        &self,
        user_id: Uuid,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<Item>>;

    async fn find_by_id_any_user(&self, id: Uuid) -> Result<Option<Item>>;

    async fn claim_item(&self, id: Uuid, claimer_id: Uuid) -> Result<Option<Uuid>>;

    async fn unclaim_item(&self, id: Uuid, claimer_id: Uuid) -> Result<Option<Uuid>>;

    async fn find_by_ids(&self, user_id: Uuid, ids: &[Uuid]) -> Result<Vec<Item>>;

    async fn find_by_ids_any_user(&self, ids: &[Uuid]) -> Result<Vec<Item>>;
}

#[async_trait]
pub trait PushTokenRepo: Send + Sync {
    async fn upsert(&self, user_id: Uuid, token: &str, platform: &str) -> Result<PushToken>;

    async fn delete_by_token(&self, user_id: Uuid, token: &str) -> Result<bool>;

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<PushToken>>;
}

// ── Service traits ──────────────────────────────────────────────────

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_password_reset_code(&self, to: &str, code: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
        username: Option<&str>,
    ) -> Result<AuthResponse, AppError>;

    async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, AppError>;

    async fn refresh(&self, raw_refresh_token: &str) -> Result<RefreshResponse, AppError>;

    async fn logout(&self, user_id: Uuid, jti: &str, token_exp: usize) -> Result<(), AppError>;

    async fn invalidate_all_tokens(&self, user_id: Uuid) -> Result<(), AppError>;

    async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError>;

    async fn forgot_password(&self, email: &str) -> Result<(), AppError>;

    async fn reset_password(
        &self,
        email: &str,
        code: &str,
        new_password: &str,
    ) -> Result<(), AppError>;
}

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ItemService: Send + Sync {
    async fn create_item(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: Option<i16>,
        category_id: Option<Uuid>,
    ) -> Result<ItemResponse, AppError>;

    async fn get_item(&self, id: Uuid, user_id: Uuid) -> Result<ItemResponse, AppError>;

    async fn list_items(
        &self,
        user_id: Uuid,
        query: &ListItemsQuery,
    ) -> Result<ItemsListResponse, AppError>;

    async fn update_item(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        url: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: Option<i16>,
        category_id: Option<Option<Uuid>>,
        status: Option<&str>,
    ) -> Result<ItemResponse, AppError>;

    async fn delete_item(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn claim_item(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;

    async fn unclaim_item(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait CategoryService: Send + Sync {
    async fn list_categories(&self, user_id: Uuid) -> Result<Vec<CategoryResponse>, AppError>;

    async fn create_category(
        &self,
        user_id: Uuid,
        name: &str,
        icon: Option<&str>,
    ) -> Result<CategoryResponse, AppError>;

    async fn update_category(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        icon: Option<&str>,
        position: Option<i32>,
    ) -> Result<CategoryResponse, AppError>;

    async fn delete_category(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileResponse, AppError>;

    async fn update_profile(
        &self,
        user_id: Uuid,
        req: &crate::dto::users::UpdateProfileRequest,
    ) -> Result<UserProfileResponse, AppError>;

    async fn delete_account(&self, user_id: Uuid) -> Result<(), AppError>;

    async fn export_data(&self, user_id: Uuid) -> Result<UserDataExport, AppError>;
}

#[async_trait]
pub trait PushTokenService: Send + Sync {
    async fn register_token(
        &self,
        user_id: Uuid,
        token: &str,
        platform: &str,
    ) -> Result<PushTokenResponse, AppError>;

    async fn unregister_token(&self, user_id: Uuid, token: &str) -> Result<(), AppError>;
}

// ── Circle traits ──────────────────────────────────────────────────

#[async_trait]
pub trait CircleRepo: Send + Sync {
    async fn create(&self, name: Option<&str>, owner_id: Uuid, is_direct: bool) -> Result<Circle>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Circle>>;

    async fn update_name(&self, id: Uuid, name: &str) -> Result<Option<Circle>>;

    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Returns (Circle, member_count) pairs, plus optional other_username for direct circles.
    async fn list_by_member(&self, user_id: Uuid) -> Result<Vec<(Circle, i64, Option<String>)>>;
}

#[async_trait]
pub trait CircleMemberRepo: Send + Sync {
    async fn add_member(&self, circle_id: Uuid, user_id: Uuid, role: &str) -> Result<CircleMember>;

    async fn remove_member(&self, circle_id: Uuid, user_id: Uuid) -> Result<bool>;

    async fn find_member(&self, circle_id: Uuid, user_id: Uuid) -> Result<Option<CircleMember>>;

    async fn list_members(&self, circle_id: Uuid) -> Result<Vec<CircleMember>>;

    async fn count_members(&self, circle_id: Uuid) -> Result<i64>;

    async fn find_direct_circle_between(&self, user_a: Uuid, user_b: Uuid) -> Result<Option<Uuid>>;
}

#[async_trait]
pub trait CircleInviteRepo: Send + Sync {
    async fn create(
        &self,
        circle_id: Uuid,
        token: &str,
        created_by: Uuid,
        expires_at: DateTime<Utc>,
        max_uses: i32,
    ) -> Result<CircleInvite>;

    async fn find_by_token(&self, token: &str) -> Result<Option<CircleInvite>>;

    async fn increment_use_count(&self, id: Uuid) -> Result<bool>;

    async fn list_active_by_circle(&self, circle_id: Uuid) -> Result<Vec<CircleInvite>>;

    async fn delete(&self, id: Uuid) -> Result<bool>;
}

#[async_trait]
pub trait CircleItemRepo: Send + Sync {
    async fn share_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        shared_by: Uuid,
    ) -> Result<CircleItem>;

    async fn unshare_item(&self, circle_id: Uuid, item_id: Uuid) -> Result<bool>;

    async fn list_by_circle(&self, circle_id: Uuid) -> Result<Vec<CircleItem>>;

    async fn find(&self, circle_id: Uuid, item_id: Uuid) -> Result<Option<CircleItem>>;

    async fn list_circles_for_item(&self, item_id: Uuid) -> Result<Vec<Uuid>>;
}

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

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait CircleService: Send + Sync {
    async fn create_circle(&self, user_id: Uuid, name: &str) -> Result<CircleResponse, AppError>;

    async fn list_circles(&self, user_id: Uuid) -> Result<Vec<CircleResponse>, AppError>;

    async fn get_circle(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<CircleDetailResponse, AppError>;

    async fn update_circle(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        name: &str,
    ) -> Result<CircleResponse, AppError>;

    async fn delete_circle(&self, circle_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn create_direct_circle(
        &self,
        owner_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<CircleResponse, AppError>;

    async fn create_invite(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        max_uses: Option<i32>,
        expires_in_hours: Option<i64>,
    ) -> Result<InviteResponse, AppError>;

    async fn join_via_invite(&self, token: &str, user_id: Uuid) -> Result<JoinResponse, AppError>;

    async fn remove_member(
        &self,
        circle_id: Uuid,
        target_user_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), AppError>;

    async fn list_invites(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<InviteResponse>, AppError>;

    async fn revoke_invite(
        &self,
        circle_id: Uuid,
        invite_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn share_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn list_circle_items(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<CircleItemResponse>, AppError>;

    async fn unshare_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_feed(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<FeedResponse, AppError>;

    async fn on_item_claimed(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;

    async fn on_item_unclaimed(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;
}

// ── Share link traits ───────────────────────────────────────────────

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

    async fn list_share_links(&self, user_id: Uuid) -> Result<Vec<ShareLinkListItem>, AppError>;

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
}

// ── Notification traits ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationOutcome {
    Sent,
    InvalidToken,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct NotificationRequest {
    pub device_token: String,
    pub title: String,
    pub body: String,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_batch(&self, messages: &[NotificationRequest]) -> Vec<NotificationOutcome>;
}

#[async_trait]
pub trait ReminderService: Send + Sync {
    async fn execute_hourly_tick(&self);
}

// ── Health trait ─────────────────────────────────────────────────────

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_db(&self) -> bool;
    async fn check_cache(&self) -> bool;
}
