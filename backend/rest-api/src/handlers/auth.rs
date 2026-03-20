use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use validator::Validate;

use crate::AppState;
use crate::dto::auth::{
    AppleAuthRequest, AuthResponse, ChangePasswordRequest, ForgotPasswordRequest,
    GoogleAuthRequest, LoginRequest, RefreshRequest, RefreshResponse, RegisterRequest,
    ResetPasswordRequest, VerifyEmailRequest, VerifyResetCodeRequest,
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
        .route("/verify-reset-code", post(verify_reset_code))
        .route("/reset-password", post(reset_password))
        .route("/verify-email", post(verify_email))
        .route("/resend-verification", post(resend_verification))
        .route("/google", post(google_auth))
        .route("/apple", post(apple_auth))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, body = AuthResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    validate_request(&req)?;

    let response = state
        .auth
        .register(
            &req.email,
            &req.password,
            req.display_name.as_deref(),
            req.username.as_deref(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    validate_request(&req)?;

    let response = state.auth.login(&req.identifier, &req.password).await?;

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, body = RefreshResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid refresh token"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    validate_request(&req)?;

    let response = state.auth.refresh(&req.refresh_token).await?;

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 204, description = "Logged out"),
    ),
    tag = "Auth",
    security(("bearer_auth" = [])),
)]
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

#[utoipa::path(
    post,
    path = "/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 204, description = "Password changed"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth",
    security(("bearer_auth" = [])),
)]
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

#[utoipa::path(
    post,
    path = "/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset code sent"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;

    state.auth.forgot_password(&req.email).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/auth/verify-reset-code",
    request_body = VerifyResetCodeRequest,
    responses(
        (status = 200, description = "Code valid"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn verify_reset_code(
    State(state): State<AppState>,
    Json(req): Json<VerifyResetCodeRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;

    state.auth.verify_reset_code(&req.email, &req.code).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 204, description = "Password reset"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth"
)]
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

#[utoipa::path(
    post,
    path = "/auth/verify-email",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;

    state.auth.verify_email(&req.token).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/auth/resend-verification",
    responses(
        (status = 204, description = "Verification email resent"),
    ),
    tag = "Auth",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state, auth_user))]
async fn resend_verification(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    state.auth.resend_verification(auth_user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/auth/google",
    request_body = GoogleAuthRequest,
    responses(
        (status = 200, body = AuthResponse, description = "Existing user logged in"),
        (status = 201, body = AuthResponse, description = "New user created"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn google_auth(
    State(state): State<AppState>,
    Json(req): Json<GoogleAuthRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    validate_request(&req)?;

    let response = state
        .auth
        .oauth_login("google", &req.id_token, req.display_name.as_deref())
        .await?;

    let status = if response.is_new_user {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/apple",
    request_body = AppleAuthRequest,
    responses(
        (status = 200, body = AuthResponse, description = "Existing user logged in"),
        (status = 201, body = AuthResponse, description = "New user created"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip(state, req))]
async fn apple_auth(
    State(state): State<AppState>,
    Json(req): Json<AppleAuthRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    validate_request(&req)?;

    let response = state
        .auth
        .oauth_login("apple", &req.id_token, req.display_name.as_deref())
        .await?;

    let status = if response.is_new_user {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(response)))
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
            username: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_rejects_short_password() {
        let req = RegisterRequest {
            email: "user@example.com".into(),
            password: "short".into(),
            display_name: None,
            username: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_accepts_valid_input() {
        let req = RegisterRequest {
            email: "user@example.com".into(),
            password: "longpassword123".into(),
            display_name: Some("Alice".into()),
            username: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn register_accepts_valid_username() {
        let req = RegisterRequest {
            email: "user@example.com".into(),
            password: "longpassword123".into(),
            display_name: None,
            username: Some("alice_e2e".into()),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn register_rejects_too_short_username() {
        let req = RegisterRequest {
            email: "user@example.com".into(),
            password: "longpassword123".into(),
            display_name: None,
            username: Some("ab".into()),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_rejects_too_long_username() {
        let req = RegisterRequest {
            email: "user@example.com".into(),
            password: "longpassword123".into(),
            display_name: None,
            username: Some("a".repeat(31)),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_rejects_empty_identifier() {
        let req = LoginRequest {
            identifier: "".into(),
            password: "a".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_accepts_email() {
        let req = LoginRequest {
            identifier: "user@example.com".into(),
            password: "a".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn login_accepts_username() {
        let req = LoginRequest {
            identifier: "myusername".into(),
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

    // ── VerifyResetCodeRequest ────────────────────────────────────────

    #[test]
    fn verify_reset_code_rejects_invalid_email() {
        let req = VerifyResetCodeRequest {
            email: "bad".into(),
            code: "123456".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn verify_reset_code_rejects_short_code() {
        let req = VerifyResetCodeRequest {
            email: "user@example.com".into(),
            code: "123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn verify_reset_code_accepts_valid_input() {
        let req = VerifyResetCodeRequest {
            email: "user@example.com".into(),
            code: "123456".into(),
        };
        assert!(req.validate().is_ok());
    }
}
