use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use validator::Validate;

use crate::AppState;
use crate::dto::auth::{
    AuthResponse, LoginRequest, RefreshRequest, RefreshResponse, RegisterRequest,
};
use crate::errors::AppError;
use crate::middleware::AuthUser;

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

#[tracing::instrument(skip(state, req))]
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    validate_request(&req)?;

    let response = state
        .auth
        .register(&req.email, &req.password, req.display_name.as_deref())
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state, req))]
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    validate_request(&req)?;

    let response = state.auth.login(&req.email, &req.password).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state, req))]
async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    validate_request(&req)?;

    let response = state.auth.refresh(&req.refresh_token).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state, auth_user))]
async fn logout(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    state.auth.logout(auth_user.user_id).await?;

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
