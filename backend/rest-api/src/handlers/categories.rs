use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::AppState;
use crate::dto::categories::CategoryResponse;
use crate::errors::AppError;
use crate::middleware::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_categories))
}

#[utoipa::path(
    get,
    path = "/categories",
    responses(
        (status = 200, body = Vec<CategoryResponse>),
    ),
    tag = "Categories",
    security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip(state))]
async fn list_categories(
    State(state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let response = state.categories.list_categories().await?;
    Ok(Json(response))
}
