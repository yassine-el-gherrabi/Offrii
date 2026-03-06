use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse};
use crate::dto::items::{ItemResponse, ItemsListResponse, ListItemsQuery};
use crate::errors::AppError;
use crate::models::{Item, RefreshToken, User};

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
}

// ── Service trait ────────────────────────────────────────────────────

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

    async fn logout(&self, user_id: Uuid) -> Result<(), AppError>;
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

// ── Health trait ─────────────────────────────────────────────────────

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_db(&self) -> bool;
    async fn check_cache(&self) -> bool;
}
