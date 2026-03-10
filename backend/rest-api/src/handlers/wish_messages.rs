use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::dto::wish_messages::{ListMessagesQuery, MessageResponse, SendMessageRequest};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/community/wishes/{wish_id}/messages",
        get(list_messages).post(send_message),
    )
}

#[tracing::instrument(skip(state))]
async fn send_message(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(wish_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    validate_request(&req)?;
    let response = state
        .wish_messages
        .send_message(wish_id, auth_user.user_id, &req.body)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn list_messages(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(wish_id): Path<Uuid>,
    Query(q): Query<ListMessagesQuery>,
) -> Result<Json<PaginatedResponse<MessageResponse>>, AppError> {
    validate_request(&q)?;
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let response = state
        .wish_messages
        .list_messages(wish_id, auth_user.user_id, page, limit, offset)
        .await?;
    Ok(Json(response))
}
