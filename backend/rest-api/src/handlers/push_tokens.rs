use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use axum::{Json, Router};
use validator::Validate;

use crate::AppState;
use crate::dto::push_tokens::{PushTokenResponse, RegisterPushTokenRequest};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(register_token))
        .route("/{token}", delete(unregister_token))
}

#[tracing::instrument(skip(state, auth_user, req))]
async fn register_token(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<RegisterPushTokenRequest>,
) -> Result<(StatusCode, Json<PushTokenResponse>), AppError> {
    validate_request(&req)?;
    let response = state
        .push_tokens
        .register_token(auth_user.user_id, &req.token, &req.platform)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state, auth_user))]
async fn unregister_token(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(token): Path<String>,
) -> Result<StatusCode, AppError> {
    state
        .push_tokens
        .unregister_token(auth_user.user_id, &token)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
