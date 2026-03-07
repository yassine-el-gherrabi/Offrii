use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse};
use crate::dto::categories::CategoryResponse;
use crate::dto::items::{ItemResponse, ItemsListResponse, ListItemsQuery};
use crate::dto::push_tokens::PushTokenResponse;
use crate::dto::users::UserProfileResponse;
use crate::errors::AppError;
use crate::models::{Category, Item, PushToken, RefreshToken, User};

// ── Repository traits ────────────────────────────────────────────────

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<User>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;

    #[allow(clippy::too_many_arguments)]
    async fn update_profile(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        reminder_freq: Option<&str>,
        reminder_time: Option<NaiveTime>,
        timezone: Option<&str>,
        utc_reminder_hour: Option<i16>,
        locale: Option<&str>,
    ) -> Result<Option<User>>;

    async fn delete_user(&self, id: Uuid) -> Result<bool>;

    async fn find_eligible_for_reminder(&self, utc_hour: i16) -> Result<Vec<User>>;

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
}

#[async_trait]
pub trait PushTokenRepo: Send + Sync {
    async fn upsert(&self, user_id: Uuid, token: &str, platform: &str) -> Result<PushToken>;

    async fn delete_by_token(&self, user_id: Uuid, token: &str) -> Result<bool>;

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<PushToken>>;
}

// ── Service traits ──────────────────────────────────────────────────

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> Result<AuthResponse, AppError>;

    async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, AppError>;

    async fn refresh(&self, raw_refresh_token: &str) -> Result<RefreshResponse, AppError>;

    async fn logout(&self, user_id: Uuid, jti: &str, token_exp: usize) -> Result<(), AppError>;

    async fn invalidate_all_tokens(&self, user_id: Uuid) -> Result<(), AppError>;
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
