use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::AppState;
use crate::dto::share_links::SharedViewResponse;
use crate::errors::AppError;
use crate::middleware::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new().route("/{token}", get(get_shared_view)).route(
        "/{token}/items/{item_id}/claim",
        axum::routing::post(claim_via_share).delete(unclaim_via_share),
    )
}

#[tracing::instrument(skip(state))]
async fn get_shared_view(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<SharedViewResponse>, AppError> {
    let response = state.share_links.get_shared_view(&token).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn claim_via_share(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((token, item_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .share_links
        .claim_via_share(&token, item_id, auth_user.user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn unclaim_via_share(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((token, item_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .share_links
        .unclaim_via_share(&token, item_id, auth_user.user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
