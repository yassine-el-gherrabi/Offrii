use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::friends::{
    FriendRequestResponse, FriendResponse, SendFriendRequestBody, SentFriendRequestResponse,
    UserSearchQuery, UserSearchResult,
};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/friend-requests", post(send_request).get(list_pending))
        .route("/friend-requests/sent", get(list_sent))
        .route("/friend-requests/{id}/accept", post(accept_request))
        .route("/friend-requests/{id}/cancel", delete(cancel_request))
        .route("/friend-requests/{id}", delete(decline_request))
        .route("/friends", get(list_friends))
        .route("/friends/{user_id}", delete(remove_friend))
}

pub fn search_router() -> Router<AppState> {
    Router::new().route("/search", get(search_users))
}

#[tracing::instrument(skip(state))]
async fn search_users(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<UserSearchQuery>,
) -> Result<Json<Vec<UserSearchResult>>, AppError> {
    let results = state.friends.search_users(&q.q, auth_user.user_id).await?;
    Ok(Json(results))
}

#[tracing::instrument(skip(state))]
async fn send_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(body): Json<SendFriendRequestBody>,
) -> Result<(StatusCode, Json<FriendRequestResponse>), AppError> {
    validate_request(&body)?;
    let response = state
        .friends
        .send_request(auth_user.user_id, &body.username)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn list_pending(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<FriendRequestResponse>>, AppError> {
    let responses = state
        .friends
        .list_pending_requests(auth_user.user_id)
        .await?;
    Ok(Json(responses))
}

#[tracing::instrument(skip(state))]
async fn list_sent(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<SentFriendRequestResponse>>, AppError> {
    let responses = state.friends.list_sent_requests(auth_user.user_id).await?;
    Ok(Json(responses))
}

#[tracing::instrument(skip(state))]
async fn cancel_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.friends.cancel_request(id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn accept_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<FriendResponse>, AppError> {
    let response = state.friends.accept_request(id, auth_user.user_id).await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn decline_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.friends.decline_request(id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn list_friends(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<FriendResponse>>, AppError> {
    let responses = state.friends.list_friends(auth_user.user_id).await?;
    Ok(Json(responses))
}

#[tracing::instrument(skip(state))]
async fn remove_friend(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .friends
        .remove_friend(auth_user.user_id, user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
