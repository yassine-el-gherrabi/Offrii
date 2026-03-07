use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use validator::Validate;

use crate::AppState;
use crate::dto::users::{UpdateProfileRequest, UserProfileResponse};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/me",
        get(get_profile)
            .patch(update_profile)
            .delete(delete_account),
    )
}

#[tracing::instrument(skip(state, auth_user))]
async fn get_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserProfileResponse>, AppError> {
    let response = state.users.get_profile(auth_user.user_id).await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state, auth_user, req))]
async fn update_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, AppError> {
    validate_request(&req)?;
    let response = state.users.update_profile(auth_user.user_id, &req).await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state, auth_user))]
async fn delete_account(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    state.users.delete_account(auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
