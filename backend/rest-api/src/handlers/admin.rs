use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::AppState;
use crate::dto::community_wishes::AdminWishResponse;
use crate::errors::AppError;
use crate::middleware::AdminUser;

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
) -> Result<Json<Vec<AdminWishResponse>>, AppError> {
    let response = state.community_wishes.admin_list_flagged().await?;
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
