use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use validator::Validate;

use crate::AppState;
use crate::dto::auth::ChangeEmailRequest;
use crate::dto::users::{UpdateProfileRequest, UserDataExport, UserProfileResponse};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/profile",
            get(get_profile)
                .patch(update_profile)
                .delete(delete_account),
        )
        .route("/export", get(export_data))
        .route("/email", post(request_email_change))
}

#[utoipa::path(
    get,
    path = "/me/profile",
    responses(
        (status = 200, body = UserProfileResponse),
    ),
    tag = "Users",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state, auth_user))]
async fn get_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserProfileResponse>, AppError> {
    let response = state.users.get_profile(auth_user.user_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/me/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, body = UserProfileResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "Users",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state, auth_user, req))]
async fn update_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, AppError> {
    validate_request(&req)?;

    // Capture old avatar_url before update (for R2 cleanup)
    let old_avatar_url = if req.avatar_url.is_some() {
        state
            .users
            .get_profile(auth_user.user_id)
            .await
            .ok()
            .and_then(|p| p.avatar_url)
    } else {
        None
    };

    let response = state.users.update_profile(auth_user.user_id, &req).await?;

    // Best-effort R2 cleanup: delete old avatar if replaced
    if let Some(old_url) = &old_avatar_url
        && response.avatar_url.as_ref() != Some(old_url)
        && let Err(e) = state.uploads.delete_image(old_url).await
    {
        tracing::warn!(error = %e, "failed to delete old avatar");
    }

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/me/export",
    responses(
        (status = 200, body = UserDataExport),
    ),
    tag = "Users",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state, auth_user))]
async fn export_data(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserDataExport>, AppError> {
    let data = state.users.export_data(auth_user.user_id).await?;
    Ok(Json(data))
}

#[utoipa::path(
    delete,
    path = "/me/profile",
    responses(
        (status = 204, description = "Account deleted"),
    ),
    tag = "Users",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state, auth_user))]
async fn delete_account(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    state.users.delete_account(auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/me/email",
    request_body = ChangeEmailRequest,
    responses(
        (status = 204, description = "Verification email sent to new address"),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already in use"),
        (status = 429, description = "Too many requests"),
    ),
    tag = "Users",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state, auth_user, req))]
async fn request_email_change(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<ChangeEmailRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;
    state
        .auth
        .request_email_change(auth_user.user_id, &req.new_email)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
