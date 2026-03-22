use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::User;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "invalid email address"))]
    pub email: String,
    #[validate(length(
        min = 8,
        max = 128,
        message = "password must be between 8 and 128 characters"
    ))]
    pub password: String,
    #[validate(length(max = 100, message = "display name must be at most 100 characters"))]
    pub display_name: Option<String>,
    #[validate(length(
        min = 3,
        max = 30,
        message = "username must be between 3 and 30 characters"
    ))]
    pub username: Option<String>,
    pub terms_accepted: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "email or username is required"))]
    pub identifier: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RefreshRequest {
    #[validate(length(min = 1, message = "refresh_token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "current password is required"))]
    pub current_password: String,
    #[validate(length(
        min = 8,
        max = 128,
        message = "new password must be between 8 and 128 characters"
    ))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "invalid email address"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    #[validate(email(message = "invalid email address"))]
    pub email: String,
    #[validate(length(equal = 6, message = "code must be exactly 6 characters"))]
    pub code: String,
    #[validate(length(
        min = 8,
        max = 128,
        message = "password must be between 8 and 128 characters"
    ))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct VerifyResetCodeRequest {
    #[validate(email(message = "invalid email address"))]
    pub email: String,
    #[validate(length(equal = 6, message = "code must be exactly 6 characters"))]
    pub code: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct VerifyEmailRequest {
    #[validate(length(min = 1, max = 64, message = "token is required"))]
    pub token: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct GoogleAuthRequest {
    #[validate(length(min = 1, message = "id_token is required"))]
    pub id_token: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct AppleAuthRequest {
    #[validate(length(min = 1, message = "id_token is required"))]
    pub id_token: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ChangeEmailRequest {
    #[validate(email(message = "invalid email address"))]
    pub new_email: String,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub tokens: TokenPair,
    pub user: UserResponse,
    pub is_new_user: bool,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RefreshResponse {
    pub tokens: TokenPair,
}

// ── User response DTO ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub username_customized: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&User> for UserResponse {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            avatar_url: u.avatar_url.clone(),
            username_customized: u.username_customized,
            email_verified: u.email_verified,
            created_at: u.created_at,
        }
    }
}
