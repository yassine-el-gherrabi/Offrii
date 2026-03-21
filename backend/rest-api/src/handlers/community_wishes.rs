use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::community_wishes::{
    CreateWishRequest, ListWishesQuery, MyWishResponse, ReportWishRequest, UpdateWishRequest,
    WishDetailResponse, WishResponse,
};
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::errors::AppError;
use crate::middleware::{AuthUser, OptionalAuthUser};

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_wish).get(list_wishes))
        .route("/mine", get(list_my_wishes))
        .route("/my-offers", get(list_my_offers))
        .route("/recent-fulfilled", get(list_recent_fulfilled))
        .route(
            "/{id}",
            get(get_wish).patch(update_wish).delete(delete_wish),
        )
        .route("/{id}/close", post(close_wish))
        .route("/{id}/reopen", post(reopen_wish))
        .route("/{id}/offer", post(offer_wish).delete(withdraw_offer))
        .route("/{id}/reject", post(reject_offer))
        .route("/{id}/confirm", post(confirm_wish))
        .route("/{id}/report", post(report_wish))
        .route("/{id}/block", post(block_wish).delete(unblock_wish))
}

#[utoipa::path(
    post,
    path = "/community/wishes",
    request_body = CreateWishRequest,
    responses(
        (status = 201, body = MyWishResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn create_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateWishRequest>,
) -> Result<(StatusCode, Json<MyWishResponse>), AppError> {
    validate_request(&req)?;
    let response = state
        .community_wishes
        .create_wish(
            auth_user.user_id,
            &req.title,
            req.description.as_deref(),
            &req.category,
            req.is_anonymous,
            req.image_url.as_deref(),
            req.links.as_deref(),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/community/wishes",
    params(ListWishesQuery),
    responses(
        (status = 200, description = "Paginated response"),
    ),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn list_wishes(
    State(state): State<AppState>,
    opt_auth: OptionalAuthUser,
    Query(q): Query<ListWishesQuery>,
) -> Result<Json<PaginatedResponse<WishResponse>>, AppError> {
    validate_request(&q)?;
    let caller_id = opt_auth.0.map(|a| a.user_id);
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let response = state
        .community_wishes
        .list_wishes(caller_id, q.category.as_deref(), page, limit, offset)
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/community/wishes/{id}",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 200, body = WishDetailResponse),
        (status = 404, description = "Wish not found"),
    ),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn get_wish(
    State(state): State<AppState>,
    opt_auth: OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<WishDetailResponse>, AppError> {
    let caller_id = opt_auth.0.map(|a| a.user_id);
    let response = state.community_wishes.get_wish(id, caller_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/community/wishes/mine",
    responses(
        (status = 200, body = Vec<MyWishResponse>),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn list_my_wishes(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<MyWishResponse>>, AppError> {
    let response = state
        .community_wishes
        .list_my_wishes(auth_user.user_id)
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/community/wishes/my-offers",
    responses(
        (status = 200, body = Vec<WishResponse>),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn list_my_offers(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<WishResponse>>, AppError> {
    let response = state
        .community_wishes
        .list_my_offers(auth_user.user_id)
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/community/wishes/recent-fulfilled",
    responses(
        (status = 200, body = Vec<WishResponse>),
    ),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn list_recent_fulfilled(
    State(state): State<AppState>,
) -> Result<Json<Vec<WishResponse>>, AppError> {
    let response = state.community_wishes.list_recent_fulfilled().await?;
    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/community/wishes/{id}",
    params(("id" = Uuid, Path, description = "Wish ID")),
    request_body = UpdateWishRequest,
    responses(
        (status = 200, body = MyWishResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn update_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWishRequest>,
) -> Result<Json<MyWishResponse>, AppError> {
    validate_request(&req)?;
    let response = state
        .community_wishes
        .update_wish(
            id,
            auth_user.user_id,
            req.title.as_deref(),
            req.description.as_deref(),
            req.category.as_deref(),
            req.image_url.as_deref(),
            req.links.as_deref(),
        )
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/community/wishes/{id}/close",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Wish closed"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn close_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .close_wish(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/community/wishes/{id}",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Wish deleted"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn delete_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .delete_wish(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/community/wishes/{id}/reopen",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Wish reopened"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn reopen_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .reopen_wish(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/community/wishes/{id}/offer",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Offer placed"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn offer_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .offer_wish(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/community/wishes/{id}/offer",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Offer withdrawn"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn withdraw_offer(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .withdraw_offer(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/community/wishes/{id}/reject",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Offer rejected"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn reject_offer(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .reject_offer(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/community/wishes/{id}/confirm",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Wish confirmed as fulfilled"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn confirm_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .confirm_wish(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/community/wishes/{id}/report",
    params(("id" = Uuid, Path, description = "Wish ID")),
    request_body = ReportWishRequest,
    responses(
        (status = 204, description = "Wish reported"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn report_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ReportWishRequest>,
) -> Result<StatusCode, AppError> {
    let details = req.details.as_deref();
    if let Some(ref reason) = req.reason {
        validate_request(&req)?;
        state
            .community_wishes
            .report_wish(id, auth_user.user_id, reason, details)
            .await?;
    } else {
        state
            .community_wishes
            .report_wish(id, auth_user.user_id, "inappropriate", details)
            .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/community/wishes/{id}/block",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Wish blocked"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn block_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .block_wish(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/community/wishes/{id}/block",
    params(("id" = Uuid, Path, description = "Wish ID")),
    responses(
        (status = 204, description = "Wish unblocked"),
    ),
    security(("bearer_auth" = [])),
    tag = "Entraide"
)]
#[tracing::instrument(skip(state))]
async fn unblock_wish(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .community_wishes
        .unblock_wish(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
