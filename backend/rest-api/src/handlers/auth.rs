use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use validator::Validate;

use crate::AppState;
use crate::dto::auth::{
    AuthResponse, ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, RefreshRequest,
    RefreshResponse, RegisterRequest, ResetPasswordRequest,
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
        .route("/change-password", post(change_password))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
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
    state
        .auth
        .logout(auth_user.user_id, &auth_user.jti, auth_user.exp)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state, auth_user, req))]
async fn change_password(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;

    state
        .auth
        .change_password(auth_user.user_id, &req.current_password, &req.new_password)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state, req))]
async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;

    state.auth.forgot_password(&req.email).await?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state, req))]
async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;

    state
        .auth
        .reset_password(&req.email, &req.code, &req.new_password)
        .await?;

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

    // ── ChangePasswordRequest ────────────────────────────────────────

    #[test]
    fn change_password_rejects_empty_current() {
        let req = ChangePasswordRequest {
            current_password: "".into(),
            new_password: "newpassword123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn change_password_rejects_short_new_password() {
        let req = ChangePasswordRequest {
            current_password: "oldpassword".into(),
            new_password: "short".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn change_password_accepts_valid_input() {
        let req = ChangePasswordRequest {
            current_password: "oldpassword".into(),
            new_password: "newpassword123".into(),
        };
        assert!(req.validate().is_ok());
    }

    // ── ForgotPasswordRequest ────────────────────────────────────────

    #[test]
    fn forgot_password_rejects_invalid_email() {
        let req = ForgotPasswordRequest {
            email: "not-an-email".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn forgot_password_accepts_valid_email() {
        let req = ForgotPasswordRequest {
            email: "user@example.com".into(),
        };
        assert!(req.validate().is_ok());
    }

    // ── ResetPasswordRequest ─────────────────────────────────────────

    #[test]
    fn reset_password_rejects_invalid_email() {
        let req = ResetPasswordRequest {
            email: "bad".into(),
            code: "123456".into(),
            new_password: "newpassword123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn reset_password_rejects_short_code() {
        let req = ResetPasswordRequest {
            email: "user@example.com".into(),
            code: "123".into(),
            new_password: "newpassword123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn reset_password_rejects_long_code() {
        let req = ResetPasswordRequest {
            email: "user@example.com".into(),
            code: "1234567".into(),
            new_password: "newpassword123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn reset_password_rejects_short_password() {
        let req = ResetPasswordRequest {
            email: "user@example.com".into(),
            code: "123456".into(),
            new_password: "short".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn reset_password_accepts_valid_input() {
        let req = ResetPasswordRequest {
            email: "user@example.com".into(),
            code: "123456".into(),
            new_password: "newpassword123".into(),
        };
        assert!(req.validate().is_ok());
    }
}
