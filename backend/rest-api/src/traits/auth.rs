use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse};
use crate::errors::AppError;
use crate::models::RefreshToken;

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
pub trait AuthService: Send + Sync {
    async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
        username: Option<&str>,
        ip: &str,
        user_agent: &str,
    ) -> Result<AuthResponse, AppError>;

    async fn login(
        &self,
        identifier: &str,
        password: &str,
        ip: &str,
        user_agent: &str,
    ) -> Result<AuthResponse, AppError>;

    async fn refresh(&self, raw_refresh_token: &str) -> Result<RefreshResponse, AppError>;

    async fn logout(&self, user_id: Uuid, jti: &str, token_exp: usize) -> Result<(), AppError>;

    async fn invalidate_all_tokens(&self, user_id: Uuid) -> Result<(), AppError>;

    async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<AuthResponse, AppError>;

    async fn forgot_password(&self, email: &str) -> Result<(), AppError>;

    async fn verify_reset_code(&self, email: &str, code: &str) -> Result<(), AppError>;

    async fn reset_password(
        &self,
        email: &str,
        code: &str,
        new_password: &str,
    ) -> Result<(), AppError>;

    async fn oauth_login(
        &self,
        provider: &str,
        id_token: &str,
        display_name: Option<&str>,
        ip: &str,
        user_agent: &str,
    ) -> Result<AuthResponse, AppError>;

    async fn verify_email(&self, token: &str) -> Result<(), AppError>;

    async fn resend_verification(&self, user_id: Uuid) -> Result<(), AppError>;

    async fn request_email_change(&self, user_id: Uuid, new_email: &str) -> Result<(), AppError>;

    async fn confirm_email_change(&self, token: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_password_reset_code(&self, to: &str, code: &str) -> Result<(), AppError>;
    async fn send_welcome_email(
        &self,
        to: &str,
        display_name: Option<&str>,
    ) -> Result<(), AppError>;
    async fn send_welcome_and_verify_email(
        &self,
        to: &str,
        display_name: Option<&str>,
        token: &str,
    ) -> Result<(), AppError>;
    async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), AppError>;
    async fn send_password_changed_email(&self, to: &str) -> Result<(), AppError>;
    async fn send_email_change_verification(&self, to: &str, token: &str) -> Result<(), AppError>;
    async fn send_email_changed_notification(
        &self,
        to: &str,
        new_email: &str,
    ) -> Result<(), AppError>;

    async fn send_inactivity_warning(&self, to: &str) -> Result<(), AppError>;
}
