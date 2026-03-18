use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::AppState;
use crate::dto::notifications::{NotificationResponse, UnreadCountResponse};
use crate::dto::pagination::PaginatedResponse;
use crate::errors::AppError;
use crate::middleware::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/read", post(mark_all_read))
        .route("/{id}/read", post(mark_read))
        .route("/unread-count", get(unread_count))
}

#[derive(Debug, serde::Deserialize)]
struct ListQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

async fn list_notifications(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<PaginatedResponse<NotificationResponse>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * limit;

    let (notifs, total) = tokio::try_join!(
        async {
            state
                .notifications
                .list_by_user(auth_user.user_id, limit, offset)
                .await
                .map_err(AppError::Internal)
        },
        async {
            state
                .notifications
                .count_total(auth_user.user_id)
                .await
                .map_err(AppError::Internal)
        },
    )?;

    // Look up actor names for display
    let actor_ids: Vec<Uuid> = notifs.iter().filter_map(|n| n.actor_id).collect();
    let actor_map: std::collections::HashMap<Uuid, String> = if actor_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        let rows: Vec<(Uuid, String, Option<String>)> =
            sqlx::query_as("SELECT id, username, display_name FROM users WHERE id = ANY($1)")
                .bind(&actor_ids)
                .fetch_all(&state.db)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

        rows.into_iter()
            .map(|(id, username, display_name)| (id, display_name.unwrap_or(username)))
            .collect()
    };

    let responses: Vec<NotificationResponse> = notifs
        .into_iter()
        .map(|n| {
            let actor_name = n.actor_id.and_then(|id| actor_map.get(&id).cloned());
            NotificationResponse::from_notification(n, actor_name)
        })
        .collect();

    Ok(Json(PaginatedResponse::new(responses, total, page, limit)))
}

async fn mark_all_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    state
        .notifications
        .mark_all_read(auth_user.user_id)
        .await
        .map_err(AppError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn mark_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .notifications
        .mark_read(id, auth_user.user_id)
        .await
        .map_err(AppError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn unread_count(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UnreadCountResponse>, AppError> {
    let count = state
        .notifications
        .count_unread(auth_user.user_id)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(UnreadCountResponse { count }))
}
