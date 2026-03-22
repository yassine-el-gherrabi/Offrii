use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::AppState;
use crate::dto::notifications::{NotificationResponse, UnreadCountResponse};
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::errors::AppError;
use crate::middleware::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/read", post(mark_all_read))
        .route("/{id}/read", post(mark_read))
        .route("/{id}", delete(delete_notification))
        .route("/unread-count", get(unread_count))
}

#[derive(Debug, serde::Deserialize)]
struct ListQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/me/notifications",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("limit" = Option<i64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "Paginated response"),
    ),
    security(("bearer_auth" = [])),
    tag = "Notifications"
)]
async fn list_notifications(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<PaginatedResponse<NotificationResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);

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

    // Look up actor names via UserRepo (no raw SQL in handlers)
    let actor_ids: Vec<Uuid> = notifs.iter().filter_map(|n| n.actor_id).collect();
    let actor_map = state
        .users
        .find_display_names(&actor_ids)
        .await
        .unwrap_or_default();

    let responses: Vec<NotificationResponse> = notifs
        .into_iter()
        .map(|n| {
            let actor_name = n.actor_id.and_then(|id| actor_map.get(&id).cloned());
            NotificationResponse::from_notification(n, actor_name)
        })
        .collect();

    Ok(Json(PaginatedResponse::new(responses, total, page, limit)))
}

#[utoipa::path(
    post,
    path = "/me/notifications/read",
    responses(
        (status = 204, description = "All notifications marked as read"),
    ),
    security(("bearer_auth" = [])),
    tag = "Notifications"
)]
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

#[utoipa::path(
    post,
    path = "/me/notifications/{id}/read",
    params(("id" = Uuid, Path, description = "Notification ID")),
    responses(
        (status = 204, description = "Notification marked as read"),
    ),
    security(("bearer_auth" = [])),
    tag = "Notifications"
)]
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

#[utoipa::path(
    delete,
    path = "/me/notifications/{id}",
    params(("id" = Uuid, Path, description = "Notification ID")),
    responses(
        (status = 204, description = "Notification deleted"),
        (status = 404, description = "Notification not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "Notifications"
)]
async fn delete_notification(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = state
        .notifications
        .delete(id, auth_user.user_id)
        .await
        .map_err(AppError::Internal)?;

    if !deleted {
        return Err(AppError::NotFound("notification not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/me/notifications/unread-count",
    responses(
        (status = 200, body = UnreadCountResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Notifications"
)]
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
