use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::dto::community_wishes::AdminWishResponse;
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::errors::AppError;
use crate::middleware::AdminUser;

#[derive(Debug, Deserialize)]
struct ListFlaggedQuery {
    limit: Option<i64>,
    page: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/wishes/pending", get(list_pending))
        .route("/wishes/{id}/approve", post(approve_wish))
        .route("/wishes/{id}/reject", post(reject_wish))
}

#[tracing::instrument(skip(state))]
async fn list_pending(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(query): Query<ListFlaggedQuery>,
) -> Result<Json<PaginatedResponse<AdminWishResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(query.page, query.limit);
    let response = state
        .community_wishes
        .admin_list_flagged(page, limit, offset)
        .await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn approve_wish(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.community_wishes.admin_approve(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn reject_wish(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.community_wishes.admin_reject(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
