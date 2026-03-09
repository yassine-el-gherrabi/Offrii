use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::circles::{
    AddMemberRequest, CircleDetailResponse, CircleResponse, CreateCircleRequest,
    CreateInviteRequest, FeedQuery, FeedResponse, InviteResponse, JoinResponse, ShareItemRequest,
    UpdateCircleRequest,
};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_circles).post(create_circle))
        .route(
            "/{id}",
            get(get_circle).patch(update_circle).delete(delete_circle),
        )
        .route("/direct/{user_id}", post(create_direct_circle))
        .route("/{id}/invite", post(create_invite))
        .route("/join/{token}", post(join_via_invite))
        .route("/{id}/members", post(add_member))
        .route("/{id}/members/{uid}", delete(remove_member))
        .route("/{id}/invites", get(list_invites))
        .route("/{id}/invites/{iid}", delete(revoke_invite))
        .route("/{id}/items", post(share_item).get(list_circle_items))
        .route("/{id}/items/{iid}", delete(unshare_item))
        .route("/{id}/feed", get(get_feed))
}

#[tracing::instrument(skip(state))]
async fn create_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateCircleRequest>,
) -> Result<(StatusCode, Json<CircleResponse>), AppError> {
    validate_request(&req)?;
    let response = state
        .circles
        .create_circle(auth_user.user_id, &req.name)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn list_circles(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<CircleResponse>>, AppError> {
    let response = state.circles.list_circles(auth_user.user_id).await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn get_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CircleDetailResponse>, AppError> {
    let response = state.circles.get_circle(id, auth_user.user_id).await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn update_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCircleRequest>,
) -> Result<Json<CircleResponse>, AppError> {
    validate_request(&req)?;
    let response = state
        .circles
        .update_circle(id, auth_user.user_id, &req.name)
        .await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn delete_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.circles.delete_circle(id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn create_direct_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<(StatusCode, Json<CircleResponse>), AppError> {
    let response = state
        .circles
        .create_direct_circle(auth_user.user_id, user_id)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn add_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .add_member_by_id(id, req.user_id, auth_user.user_id)
        .await?;
    Ok(StatusCode::CREATED)
}

#[tracing::instrument(skip(state))]
async fn create_invite(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    body: Option<Json<CreateInviteRequest>>,
) -> Result<(StatusCode, Json<InviteResponse>), AppError> {
    let (max_uses, expires_in_hours) = match body {
        Some(Json(b)) => (b.max_uses, b.expires_in_hours),
        None => (None, None),
    };
    let response = state
        .circles
        .create_invite(id, auth_user.user_id, max_uses, expires_in_hours)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn join_via_invite(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(token): Path<String>,
) -> Result<Json<JoinResponse>, AppError> {
    let response = state
        .circles
        .join_via_invite(&token, auth_user.user_id)
        .await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn remove_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, uid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .remove_member(id, uid, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn list_invites(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<InviteResponse>>, AppError> {
    let response = state.circles.list_invites(id, auth_user.user_id).await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn revoke_invite(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, iid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .revoke_invite(id, iid, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn share_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ShareItemRequest>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .share_item(id, req.item_id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn list_circle_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::dto::circles::CircleItemResponse>>, AppError> {
    let response = state
        .circles
        .list_circle_items(id, auth_user.user_id)
        .await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn unshare_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, iid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .unshare_item(id, iid, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn get_feed(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<FeedQuery>,
) -> Result<Json<FeedResponse>, AppError> {
    let response = state
        .circles
        .get_feed(id, auth_user.user_id, q.page, q.per_page)
        .await?;
    Ok(Json(response))
}
