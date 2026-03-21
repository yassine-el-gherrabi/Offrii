use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::dto::share_links::{
    CreateShareLinkRequest, ShareLinkListItem, ShareLinkResponse, UpdateShareLinkRequest,
};
use crate::errors::AppError;
use crate::middleware::AuthUser;

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_share_links).post(create_share_link))
        .route(
            "/{id}",
            axum::routing::delete(delete_share_link).patch(update_share_link),
        )
}

#[utoipa::path(
    post,
    path = "/share-links",
    request_body(content = Option<CreateShareLinkRequest>),
    responses(
        (status = 201, body = ShareLinkResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "ShareLinks"
)]
#[tracing::instrument(skip(state))]
async fn create_share_link(
    State(state): State<AppState>,
    auth_user: AuthUser,
    body: Option<Json<CreateShareLinkRequest>>,
) -> Result<(StatusCode, Json<ShareLinkResponse>), AppError> {
    let (expires_at, label, permissions, scope, scope_data) = match body {
        Some(Json(b)) => (b.expires_at, b.label, b.permissions, b.scope, b.scope_data),
        None => (None, None, None, None, None),
    };

    let response = state
        .share_links
        .create_share_link(
            auth_user.user_id,
            expires_at,
            label.as_deref(),
            permissions.as_deref(),
            scope.as_deref(),
            scope_data.as_ref(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/share-links",
    responses(
        (status = 200, body = Vec<ShareLinkListItem>),
    ),
    security(("bearer_auth" = [])),
    tag = "ShareLinks"
)]
#[tracing::instrument(skip(state))]
async fn list_share_links(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ShareLinkListItem>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let (data, total) = state
        .share_links
        .list_share_links(auth_user.user_id, limit, offset)
        .await?;
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    delete,
    path = "/share-links/{id}",
    params(("id" = Uuid, Path, description = "Share link ID")),
    responses(
        (status = 204, description = "Share link deleted"),
    ),
    security(("bearer_auth" = [])),
    tag = "ShareLinks"
)]
#[tracing::instrument(skip(state))]
async fn delete_share_link(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .share_links
        .delete_share_link(id, auth_user.user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/share-links/{id}",
    params(("id" = Uuid, Path, description = "Share link ID")),
    request_body = UpdateShareLinkRequest,
    responses(
        (status = 200, body = ShareLinkResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "ShareLinks"
)]
#[tracing::instrument(skip(state))]
async fn update_share_link(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateShareLinkRequest>,
) -> Result<Json<ShareLinkResponse>, AppError> {
    let response = state
        .share_links
        .update_share_link(id, auth_user.user_id, &body)
        .await?;

    Ok(Json(response))
}
