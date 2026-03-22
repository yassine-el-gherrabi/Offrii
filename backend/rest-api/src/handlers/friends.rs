use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::friends::{
    FriendRequestResponse, FriendResponse, SendFriendRequestBody, SentFriendRequestResponse,
    UserSearchQuery, UserSearchResult,
};
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::errors::AppError;
use crate::middleware::AuthUser;

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

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

#[utoipa::path(
    get,
    path = "/users/search",
    params(UserSearchQuery),
    responses(
        (status = 200, body = Vec<UserSearchResult>),
        (status = 400, description = "Validation error"),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn search_users(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<UserSearchQuery>,
) -> Result<Json<Vec<UserSearchResult>>, AppError> {
    validate_request(&q)?;
    let results = state.friends.search_users(&q.q, auth_user.user_id).await?;
    Ok(Json(results))
}

#[utoipa::path(
    post,
    path = "/me/friend-requests",
    request_body = SendFriendRequestBody,
    responses(
        (status = 201, body = FriendRequestResponse),
        (status = 400, description = "Validation error"),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
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

#[utoipa::path(
    get,
    path = "/me/friend-requests",
    responses(
        (status = 200, body = Vec<FriendRequestResponse>),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn list_pending(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<FriendRequestResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let (data, total) = state
        .friends
        .list_pending_requests(auth_user.user_id, limit, offset)
        .await?;
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    get,
    path = "/me/friend-requests/sent",
    responses(
        (status = 200, body = Vec<SentFriendRequestResponse>),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn list_sent(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<SentFriendRequestResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let (data, total) = state
        .friends
        .list_sent_requests(auth_user.user_id, limit, offset)
        .await?;
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    delete,
    path = "/me/friend-requests/{id}/cancel",
    params(("id" = Uuid, Path, description = "Friend request ID")),
    responses(
        (status = 204, description = "Request cancelled"),
        (status = 404, description = "Request not found"),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn cancel_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.friends.cancel_request(id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/me/friend-requests/{id}/accept",
    params(("id" = Uuid, Path, description = "Friend request ID")),
    responses(
        (status = 200, body = FriendResponse),
        (status = 404, description = "Request not found"),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn accept_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<FriendResponse>, AppError> {
    let response = state.friends.accept_request(id, auth_user.user_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/me/friend-requests/{id}",
    params(("id" = Uuid, Path, description = "Friend request ID")),
    responses(
        (status = 204, description = "Request declined"),
        (status = 404, description = "Request not found"),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn decline_request(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.friends.decline_request(id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/me/friends",
    responses(
        (status = 200, body = Vec<FriendResponse>),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn list_friends(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<FriendResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let (data, total) = state
        .friends
        .list_friends(auth_user.user_id, limit, offset)
        .await?;
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    delete,
    path = "/me/friends/{user_id}",
    params(("user_id" = Uuid, Path, description = "Friend's user ID")),
    responses(
        (status = 204, description = "Friend removed"),
        (status = 404, description = "Friend not found"),
    ),
    tag = "Friends",
    security(("bearer_auth" = [])),
)]
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
