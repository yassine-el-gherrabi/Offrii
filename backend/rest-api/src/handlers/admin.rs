use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::dto::community_wishes::AdminWishListResponse;
use crate::errors::AppError;
use crate::middleware::AdminUser;

#[derive(Debug, Deserialize)]
struct ListFlaggedQuery {
    limit: Option<i64>,
    offset: Option<i64>,
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
) -> Result<Json<AdminWishListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let response = state
        .community_wishes
        .admin_list_flagged(limit, offset)
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
