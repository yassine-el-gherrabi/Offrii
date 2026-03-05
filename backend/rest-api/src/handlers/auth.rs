use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::services::auth_service;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "invalid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 1, message = "refresh_token is required"))]
    pub refresh_token: String,
}

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<auth_service::AuthResponse>), AppError> {
    validate_request(&req)?;

    let response = auth_service::register(
        &state.db,
        &state.redis,
        &state.jwt,
        &req.email,
        &req.password,
        req.display_name.as_deref(),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<auth_service::AuthResponse>, AppError> {
    validate_request(&req)?;

    let response = auth_service::login(
        &state.db,
        &state.redis,
        &state.jwt,
        &req.email,
        &req.password,
    )
    .await?;

    Ok(Json(response))
}

async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<auth_service::RefreshResponse>, AppError> {
    validate_request(&req)?;

    let response =
        auth_service::refresh(&state.db, &state.redis, &state.jwt, &req.refresh_token).await?;

    Ok(Json(response))
}

async fn logout(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    auth_service::logout(&state.db, &state.redis, auth_user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_rejects_invalid_email() {
        let req = RegisterRequest {
            email: "not-an-email".into(),
            password: "longpassword".into(),
            display_name: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_rejects_short_password() {
        let req = RegisterRequest {
            email: "user@example.com".into(),
            password: "short".into(),
            display_name: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_accepts_valid_input() {
        let req = RegisterRequest {
            email: "user@example.com".into(),
            password: "longpassword123".into(),
            display_name: Some("Alice".into()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn login_rejects_invalid_email() {
        let req = LoginRequest {
            email: "bad-email".into(),
            password: "a".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_accepts_valid_input() {
        let req = LoginRequest {
            email: "user@example.com".into(),
            password: "a".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn refresh_rejects_empty_token() {
        let req = RefreshRequest {
            refresh_token: "".into(),
        };
        assert!(req.validate().is_err());
    }
}
