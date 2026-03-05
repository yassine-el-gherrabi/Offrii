use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse};
use crate::errors::AppError;
use crate::models::{RefreshToken, User};

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

    async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>>;
}

#[async_trait]
pub trait RefreshTokenRepo: Send + Sync {
    async fn insert(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken>;

    async fn find_active_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>>;

    async fn revoke_by_hash(&self, token_hash: &str) -> Result<bool>;

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<Vec<String>>;
}

// ── Cache trait ──────────────────────────────────────────────────────

#[async_trait]
pub trait TokenCache: Send + Sync {
    async fn store(&self, token_hash: &str, user_id: Uuid);
    async fn get(&self, token_hash: &str) -> Option<Uuid>;
    async fn delete(&self, token_hash: &str);
    async fn delete_many(&self, token_hashes: &[String]);
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

// ── Health trait ─────────────────────────────────────────────────────

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_db(&self) -> bool;
    async fn check_cache(&self) -> bool;
}
