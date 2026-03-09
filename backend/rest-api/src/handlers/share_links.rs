use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::AppState;
use crate::dto::share_links::{CreateShareLinkRequest, ShareLinkListItem, ShareLinkResponse};
use crate::errors::AppError;
use crate::middleware::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_share_links).post(create_share_link))
        .route("/{id}", axum::routing::delete(delete_share_link))
}

#[tracing::instrument(skip(state))]
async fn create_share_link(
    State(state): State<AppState>,
    auth_user: AuthUser,
    body: Option<Json<CreateShareLinkRequest>>,
) -> Result<(StatusCode, Json<ShareLinkResponse>), AppError> {
    let expires_at = body.and_then(|b| b.0.expires_at);

    let response = state
        .share_links
        .create_share_link(auth_user.user_id, expires_at)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn list_share_links(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<ShareLinkListItem>>, AppError> {
    let response = state
        .share_links
        .list_share_links(auth_user.user_id)
        .await?;

    Ok(Json(response))
}

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
