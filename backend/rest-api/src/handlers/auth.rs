use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use axum::routing::{get, post};
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

fn extract_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default()
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
        .route("/change-password", post(change_password))
        .route("/forgot-password", post(forgot_password))
        .route("/verify-reset-code", post(verify_reset_code))
        .route("/reset-password", post(reset_password))
        .route("/verify-email", post(verify_email).get(verify_email_get))
        .route("/resend-verification", post(resend_verification))
        .route("/google", post(google_auth))
        .route("/apple", post(apple_auth))
        .route("/verify-email-change", get(verify_email_change_get))
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
#[tracing::instrument(skip(state, headers, req))]
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    validate_request(&req)?;

    // CGU acceptance is required
    if req.terms_accepted != Some(true) {
        return Err(AppError::BadRequest("terms_accepted is required".into()));
    }

    let ip = extract_client_ip(&headers);
    let ua = extract_user_agent(&headers);

    let response = state
        .auth
        .register(
            &req.email,
            &req.password,
            req.display_name.as_deref(),
            req.username.as_deref(),
            &ip,
            &ua,
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
#[tracing::instrument(skip(state, headers, req))]
async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    validate_request(&req)?;

    let ip = extract_client_ip(&headers);
    let ua = extract_user_agent(&headers);

    let response = state
        .auth
        .login(&req.identifier, &req.password, &ip, &ua)
        .await?;

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
) -> Result<Json<AuthResponse>, AppError> {
    validate_request(&req)?;

    let response = state
        .auth
        .change_password(auth_user.user_id, &req.current_password, &req.new_password)
        .await?;

    Ok(Json(response))
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

#[derive(Debug, serde::Deserialize)]
struct VerifyEmailQuery {
    token: String,
}

async fn verify_email_get(
    State(state): State<AppState>,
    Query(query): Query<VerifyEmailQuery>,
) -> Html<String> {
    match state.auth.verify_email(&query.token).await {
        Ok(()) => Html(verify_email_page(
            "Email vérifié",
            "Votre adresse email a bien été vérifiée. Vous pouvez retourner sur l'application.",
            true,
        )),
        Err(_) => Html(verify_email_page(
            "Lien invalide ou expiré",
            "Ce lien de vérification est invalide ou a expiré. Renvoyez un email de vérification depuis l'application.",
            false,
        )),
    }
}

#[derive(Debug, serde::Deserialize)]
struct VerifyEmailChangeQuery {
    token: String,
}

async fn verify_email_change_get(
    State(state): State<AppState>,
    Query(query): Query<VerifyEmailChangeQuery>,
) -> Html<String> {
    match state.auth.confirm_email_change(&query.token).await {
        Ok(()) => Html(verify_email_page(
            "Email modifié",
            "Votre adresse email a bien été modifiée. Vous devez vous reconnecter avec votre nouvel email.",
            true,
        )),
        Err(_) => Html(verify_email_page(
            "Lien invalide ou expiré",
            "Ce lien de changement d'email est invalide ou a expiré. Réessayez depuis l'application.",
            false,
        )),
    }
}

fn verify_email_page(title: &str, message: &str, success: bool) -> String {
    let color = if success { "#FF6B6B" } else { "#ef4444" };
    let icon_svg = if success {
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="32" height="32"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/></svg>"#
    } else {
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="32" height="32"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12 19 6.41z"/></svg>"#
    };
    let logo_url = "https://cdn.offrii.com/branding/logo-1024.png";

    let mut h = String::with_capacity(4096);
    h.push_str("<!DOCTYPE html><html lang=\"fr\"><head><meta charset=\"utf-8\">");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    h.push_str(&format!("<title>{title} \u{2014} Offrii</title>"));
    h.push_str("<meta name=\"theme-color\" content=\"#FF6B6B\">");
    h.push_str("<link rel=\"icon\" type=\"image/png\" sizes=\"32x32\" href=\"/favicon.png\">");
    h.push_str("<link rel=\"icon\" type=\"image/x-icon\" href=\"/favicon.ico\">");
    h.push_str(&format!(
        "<style>\
         *{{margin:0;padding:0;box-sizing:border-box}}\
         body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0;background:#FFFAF9;-webkit-font-smoothing:antialiased;overflow-x:hidden}}\
         .blob{{position:fixed;border-radius:50%;opacity:0.08;filter:blur(60px);z-index:0;pointer-events:none}}\
         .blob-1{{width:300px;height:300px;background:#FF6B6B;top:-100px;right:-50px}}\
         .blob-2{{width:250px;height:250px;background:#FFB347;bottom:-80px;left:-60px}}\
         .wrap{{position:relative;z-index:1;text-align:center;padding:48px 32px;max-width:420px;width:100%}}\
         .brand{{display:flex;align-items:center;justify-content:center;gap:10px;margin-bottom:32px}}\
         .brand img{{width:56px;height:56px;border-radius:14px}}\
         .brand span{{font-size:20px;font-weight:700;color:#FF6B6B;letter-spacing:-0.02em}}\
         .card{{text-align:center;padding:40px 32px;background:#fff;border-radius:16px;box-shadow:0 4px 24px rgba(0,0,0,0.06)}}\
         .icon{{display:inline-flex;align-items:center;justify-content:center;width:64px;height:64px;border-radius:50%;background:{color}1a;margin-bottom:20px}}\
         .icon svg{{fill:{color}}}\
         h1{{font-size:22px;font-weight:700;color:#1a1a2e;margin:0 0 12px}}\
         p{{font-size:15px;color:#6b7280;line-height:1.6;margin:0}}\
         .ft{{margin-top:24px;font-size:12px;color:#9ca3af}}\
         .ft-brand{{color:#FF6B6B;font-weight:600}}\
         .ft a{{color:#9ca3af;text-decoration:underline}}\
         </style>"
    ));
    h.push_str("</head><body>");
    h.push_str("<div class=\"blob blob-1\"></div><div class=\"blob blob-2\"></div>");
    h.push_str("<div class=\"wrap\">");
    h.push_str(&format!(
        "<div class=\"brand\"><img src=\"{logo_url}\" alt=\"Offrii\"><span>Offrii</span></div>"
    ));
    h.push_str("<div class=\"card\">");
    h.push_str(&format!("<div class=\"icon\">{icon_svg}</div>"));
    h.push_str(&format!("<h1>{title}</h1>"));
    h.push_str(&format!("<p>{message}</p>"));
    h.push_str("</div>");
    h.push_str("<div class=\"ft\">");
    h.push_str(
        "<p><span class=\"ft-brand\">Offrii</span> \u{2014} Offre, partage, fais plaisir.<br>",
    );
    h.push_str("<a href=\"https://offrii.com\">offrii.com</a></p>");
    h.push_str("</div></div></body></html>");
    h
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
#[tracing::instrument(skip(state, headers, req))]
async fn google_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<GoogleAuthRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    validate_request(&req)?;

    let ip = extract_client_ip(&headers);
    let ua = extract_user_agent(&headers);

    let response = state
        .auth
        .oauth_login(
            "google",
            &req.id_token,
            req.display_name.as_deref(),
            &ip,
            &ua,
        )
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
#[tracing::instrument(skip(state, headers, req))]
async fn apple_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AppleAuthRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    validate_request(&req)?;

    let ip = extract_client_ip(&headers);
    let ua = extract_user_agent(&headers);

    let response = state
        .auth
        .oauth_login(
            "apple",
            &req.id_token,
            req.display_name.as_deref(),
            &ip,
            &ua,
        )
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
            terms_accepted: Some(true),
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
            terms_accepted: Some(true),
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
            terms_accepted: Some(true),
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
            terms_accepted: Some(true),
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
            terms_accepted: Some(true),
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
            terms_accepted: Some(true),
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
