use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use serde::Deserialize;

use crate::dto::circles::{
    AddMemberRequest, BatchShareRequest, CircleDetailResponse, CircleEventResponse,
    CircleItemResponse, CircleResponse, CircleShareRuleSummary, CreateCircleRequest,
    CreateInviteRequest, FeedQuery, InviteResponse, JoinResponse, ReservationResponse,
    SetShareRuleRequest, ShareItemRequest, ShareRuleResponse, TransferOwnershipRequest,
    UpdateCircleRequest,
};
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::errors::AppError;
use crate::middleware::AuthUser;

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_circles).post(create_circle))
        .route(
            "/{id}",
            get(get_circle).patch(update_circle).delete(delete_circle),
        )
        .route("/direct/{user_id}", post(create_direct_circle))
        .route("/{id}/invite", post(create_invite))
        .route("/join/{token}", post(join_via_invite))
        .route("/{id}/members", post(add_member))
        .route("/{id}/members/{uid}", delete(remove_member))
        .route("/{id}/invites", get(list_invites))
        .route("/{id}/invites/{iid}", delete(revoke_invite))
        .route("/{id}/items", post(share_item).get(list_circle_items))
        .route("/{id}/items/batch", post(batch_share_items))
        .route(
            "/{id}/items/{iid}",
            get(get_circle_item).delete(unshare_item),
        )
        .route("/{id}/share-rule", get(get_share_rule).put(set_share_rule))
        .route("/{id}/feed", get(get_feed))
        .route("/{id}/transfer", post(transfer_ownership))
        .route("/my-reservations", get(list_reservations))
        .route("/my-share-rules", get(list_my_share_rules))
}

#[utoipa::path(
    post,
    path = "/circles",
    request_body = CreateCircleRequest,
    responses(
        (status = 201, body = CircleResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn create_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateCircleRequest>,
) -> Result<(StatusCode, Json<CircleResponse>), AppError> {
    validate_request(&req)?;
    let response = state
        .circles
        .create_circle(auth_user.user_id, &req.name)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/circles",
    responses(
        (status = 200, description = "Paginated response"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn list_circles(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<CircleResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let (data, total) = state
        .circles
        .list_circles(auth_user.user_id, limit, offset)
        .await?;
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    get,
    path = "/circles/{id}",
    params(("id" = Uuid, Path, description = "Circle ID")),
    responses(
        (status = 200, body = CircleDetailResponse),
        (status = 404, description = "Circle not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn get_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CircleDetailResponse>, AppError> {
    let response = state.circles.get_circle(id, auth_user.user_id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/circles/{id}",
    params(("id" = Uuid, Path, description = "Circle ID")),
    request_body = UpdateCircleRequest,
    responses(
        (status = 200, body = CircleResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn update_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCircleRequest>,
) -> Result<Json<CircleResponse>, AppError> {
    validate_request(&req)?;

    // Capture old image_url before update (for R2 cleanup)
    let old_image_url = if req.image_url.is_some() {
        state
            .circles
            .get_circle(id, auth_user.user_id)
            .await
            .ok()
            .and_then(|c| c.image_url)
    } else {
        None
    };

    let name = req.name.as_deref().unwrap_or("");
    let response = state
        .circles
        .update_circle(
            id,
            auth_user.user_id,
            name,
            req.image_url.as_ref().map(|v| v.as_deref()),
        )
        .await?;

    // Best-effort R2 cleanup: delete old image if replaced
    if let Some(old_url) = &old_image_url
        && response.image_url.as_ref() != Some(old_url)
        && let Err(e) = state.uploads.delete_image(old_url).await
    {
        tracing::warn!(error = %e, "failed to delete old circle image");
    }

    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/circles/{id}",
    params(("id" = Uuid, Path, description = "Circle ID")),
    responses(
        (status = 204, description = "Circle deleted"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn delete_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.circles.delete_circle(id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/circles/direct/{user_id}",
    params(("user_id" = Uuid, Path, description = "Target user ID")),
    responses(
        (status = 201, body = CircleResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn create_direct_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<(StatusCode, Json<CircleResponse>), AppError> {
    let response = state
        .circles
        .create_direct_circle(auth_user.user_id, user_id)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/circles/{id}/members",
    params(("id" = Uuid, Path, description = "Circle ID")),
    request_body = AddMemberRequest,
    responses(
        (status = 201, description = "Member added"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn add_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .add_member_by_id(id, req.user_id, auth_user.user_id)
        .await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    post,
    path = "/circles/{id}/invite",
    params(("id" = Uuid, Path, description = "Circle ID")),
    request_body(content = Option<CreateInviteRequest>),
    responses(
        (status = 201, body = InviteResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn create_invite(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    body: Option<Json<CreateInviteRequest>>,
) -> Result<(StatusCode, Json<InviteResponse>), AppError> {
    let (max_uses, expires_in_hours) = match body {
        Some(Json(b)) => (b.max_uses, b.expires_in_hours),
        None => (None, None),
    };
    let response = state
        .circles
        .create_invite(id, auth_user.user_id, max_uses, expires_in_hours)
        .await?
        .with_url(&state.app_base_url);
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/circles/join/{token}",
    params(("token" = String, Path, description = "Invite token")),
    responses(
        (status = 200, body = JoinResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn join_via_invite(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(token): Path<String>,
) -> Result<Json<JoinResponse>, AppError> {
    let response = state
        .circles
        .join_via_invite(&token, auth_user.user_id)
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/circles/{id}/members/{uid}",
    params(
        ("id" = Uuid, Path, description = "Circle ID"),
        ("uid" = Uuid, Path, description = "User ID to remove"),
    ),
    responses(
        (status = 204, description = "Member removed"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn remove_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, uid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .remove_member(id, uid, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/circles/{id}/invites",
    params(("id" = Uuid, Path, description = "Circle ID")),
    responses(
        (status = 200, description = "Paginated response"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn list_invites(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<InviteResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let base_url = &state.app_base_url;
    let (data, total) = state
        .circles
        .list_invites(id, auth_user.user_id, limit, offset)
        .await?;
    let data: Vec<InviteResponse> = data.into_iter().map(|r| r.with_url(base_url)).collect();
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    delete,
    path = "/circles/{id}/invites/{iid}",
    params(
        ("id" = Uuid, Path, description = "Circle ID"),
        ("iid" = Uuid, Path, description = "Invite ID"),
    ),
    responses(
        (status = 204, description = "Invite revoked"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn revoke_invite(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, iid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .revoke_invite(id, iid, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/circles/{id}/items",
    params(("id" = Uuid, Path, description = "Circle ID")),
    request_body = ShareItemRequest,
    responses(
        (status = 204, description = "Item shared"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn share_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ShareItemRequest>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .share_item(id, req.item_id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/circles/{id}/items/batch",
    params(("id" = Uuid, Path, description = "Circle ID")),
    request_body = BatchShareRequest,
    responses(
        (status = 204, description = "Items shared in batch"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn batch_share_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<crate::dto::circles::BatchShareRequest>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .batch_share_items(id, &req.item_ids, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/circles/{id}/share-rule",
    params(("id" = Uuid, Path, description = "Circle ID")),
    responses(
        (status = 200, body = ShareRuleResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn get_share_rule(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::dto::circles::ShareRuleResponse>, AppError> {
    // Membership check via circle service (no raw SQL in handlers)
    let circle = state.circles.get_circle(id, auth_user.user_id).await?;
    let _ = circle; // ensure membership; get_circle already checks

    let rule = state
        .share_rules
        .get(id, auth_user.user_id)
        .await
        .map_err(AppError::Internal)?;

    match rule {
        Some(r) => Ok(Json(crate::dto::circles::ShareRuleResponse {
            share_mode: r.share_mode,
            category_ids: r.category_ids,
            updated_at: r.updated_at,
        })),
        None => Ok(Json(crate::dto::circles::ShareRuleResponse {
            share_mode: "none".to_string(),
            category_ids: vec![],
            updated_at: chrono::Utc::now(),
        })),
    }
}

#[utoipa::path(
    put,
    path = "/circles/{id}/share-rule",
    params(("id" = Uuid, Path, description = "Circle ID")),
    request_body = SetShareRuleRequest,
    responses(
        (status = 204, description = "Share rule updated"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn set_share_rule(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<crate::dto::circles::SetShareRuleRequest>,
) -> Result<StatusCode, AppError> {
    // Membership check via circle service (no raw SQL in handlers)
    let circle = state.circles.get_circle(id, auth_user.user_id).await?;
    let _ = circle;

    // Validate share_mode
    let valid_modes = ["none", "all", "categories", "selection"];
    if !valid_modes.contains(&req.share_mode.as_str()) {
        return Err(AppError::BadRequest(
            "share_mode must be one of: none, all, categories, selection".into(),
        ));
    }

    // Clean up circle_items when switching away from selection mode
    // (or stopping sharing entirely) — via repo, not raw SQL
    if req.share_mode != "selection" {
        // circle_items deletion is best-effort
        let _ = state
            .circles
            .unshare_all_for_user(id, auth_user.user_id)
            .await;
    }

    if req.share_mode == "none" {
        state
            .share_rules
            .delete(id, auth_user.user_id)
            .await
            .map_err(AppError::Internal)?;
    } else {
        state
            .share_rules
            .upsert(id, auth_user.user_id, &req.share_mode, &req.category_ids)
            .await
            .map_err(AppError::Internal)?;
    }

    // Invalidate the item list cache so the grid reflects the change
    state.items.invalidate_list_cache(auth_user.user_id).await;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/circles/{id}/items",
    params(("id" = Uuid, Path, description = "Circle ID")),
    responses(
        (status = 200, description = "Paginated response"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn list_circle_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<CircleItemResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let (data, total) = state
        .circles
        .list_circle_items(id, auth_user.user_id, limit, offset)
        .await?;
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    get,
    path = "/circles/{id}/items/{iid}",
    params(
        ("id" = Uuid, Path, description = "Circle ID"),
        ("iid" = Uuid, Path, description = "Item ID"),
    ),
    responses(
        (status = 200, body = CircleItemResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn get_circle_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, iid)): Path<(Uuid, Uuid)>,
) -> Result<Json<CircleItemResponse>, AppError> {
    let item = state
        .circles
        .get_circle_item(id, iid, auth_user.user_id)
        .await?;
    Ok(Json(item))
}

#[utoipa::path(
    delete,
    path = "/circles/{id}/items/{iid}",
    params(
        ("id" = Uuid, Path, description = "Circle ID"),
        ("iid" = Uuid, Path, description = "Item ID"),
    ),
    responses(
        (status = 204, description = "Item unshared"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn unshare_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, iid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .unshare_item(id, iid, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/circles/{id}/feed",
    params(("id" = Uuid, Path, description = "Circle ID")),
    responses(
        (status = 200, description = "Paginated response"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn get_feed(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<FeedQuery>,
) -> Result<Json<PaginatedResponse<CircleEventResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let response = state
        .circles
        .get_feed(id, auth_user.user_id, page, limit, offset)
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/circles/{id}/transfer",
    params(("id" = Uuid, Path, description = "Circle ID")),
    request_body = TransferOwnershipRequest,
    responses(
        (status = 204, description = "Ownership transferred"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn transfer_ownership(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<StatusCode, AppError> {
    state
        .circles
        .transfer_ownership(id, req.user_id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/circles/my-reservations",
    responses(
        (status = 200, description = "Paginated response"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
#[tracing::instrument(skip(state))]
async fn list_reservations(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ReservationResponse>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let (data, total) = state
        .circles
        .list_reservations(auth_user.user_id, limit, offset)
        .await?;
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

#[utoipa::path(
    get,
    path = "/circles/my-share-rules",
    responses(
        (status = 200, description = "Paginated response"),
    ),
    security(("bearer_auth" = [])),
    tag = "Circles"
)]
async fn list_my_share_rules(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<CircleShareRuleSummary>>, AppError> {
    let (page, limit, offset) = normalize_pagination(q.page, q.limit);
    let rules = state
        .share_rules
        .list_by_user(auth_user.user_id)
        .await
        .map_err(AppError::Internal)?;

    let all: Vec<CircleShareRuleSummary> = rules
        .into_iter()
        .map(|r| CircleShareRuleSummary {
            circle_id: r.circle_id,
            share_mode: r.share_mode,
            category_count: r.category_ids.len(),
        })
        .collect();

    let total = all.len() as i64;
    let data = all
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    Ok(Json(PaginatedResponse::new(data, total, page, limit)))
}

/// Public HTML page for circle invite links: GET /join/{token}
pub async fn join_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Response {
    // Look up the invite to get circle info
    let (circle_name, circle_image) = match state.circles.get_invite_circle_info(&token).await {
        Ok(info) => info,
        Err(_) => {
            return Html(render_join_error_html(&headers)).into_response();
        }
    };

    Html(render_join_html(
        &token,
        &circle_name,
        circle_image.as_deref(),
        &headers,
    ))
    .into_response()
}

fn get_lang(headers: &HeaderMap) -> &str {
    headers
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .map(|v| if v.starts_with("en") { "en" } else { "fr" })
        .unwrap_or("fr")
}

fn render_join_html(
    token: &str,
    circle_name: &str,
    circle_image: Option<&str>,
    headers: &HeaderMap,
) -> String {
    let lang = get_lang(headers);
    let (title, subtitle, btn, or_text, dl_text, footer_slogan) = if lang == "en" {
        (
            format!("Join \"{}\"", circle_name),
            "You've been invited to join a circle on Offrii.",
            "Join circle",
            "or",
            "Download Offrii",
            "Give, share, delight",
        )
    } else {
        (
            format!("Rejoindre \"{}\"", circle_name),
            "Vous avez \u{00e9}t\u{00e9} invit\u{00e9} \u{00e0} rejoindre un cercle sur Offrii.",
            "Rejoindre le cercle",
            "ou",
            "T\u{00e9}l\u{00e9}charger Offrii",
            "Offre, partage, fais plaisir",
        )
    };

    let escaped_name = circle_name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    let logo_url = "https://cdn.offrii.com/branding/logo-1024.png";
    let html_lang = if lang == "en" { "en" } else { "fr" };

    let avatar_html = if let Some(img) = circle_image {
        format!("<img class=\"av-img\" src=\"{}\" alt=\"\">", img)
    } else {
        format!(
            "<div class=\"av\">{}</div>",
            escaped_name.chars().next().unwrap_or('?').to_uppercase()
        )
    };

    let mut h = String::with_capacity(4096);
    h.push_str(&format!(
        "<!DOCTYPE html><html lang=\"{html_lang}\"><head><meta charset=\"utf-8\">"
    ));
    h.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    h.push_str(&format!("<title>{title} \u{2014} Offrii</title>"));
    h.push_str(&format!("<meta property=\"og:title\" content=\"{title}\">"));
    h.push_str(&format!(
        "<meta property=\"og:description\" content=\"{subtitle}\">"
    ));
    h.push_str("<meta property=\"og:image\" content=\"https://pub-83ca22acc7354445815c6b4e152ba243.r2.dev/branding/opengraph.png\">");
    h.push_str("<meta property=\"og:type\" content=\"website\">");
    h.push_str("<meta name=\"theme-color\" content=\"#FF6B6B\">");
    h.push_str("<link rel=\"icon\" type=\"image/png\" sizes=\"32x32\" href=\"/favicon.png\">");
    h.push_str("<link rel=\"icon\" type=\"image/x-icon\" href=\"/favicon.ico\">");
    h.push_str(
        "<style>\
         *{margin:0;padding:0;box-sizing:border-box}\
         body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;background:#FFFAF9;display:flex;justify-content:center;align-items:center;min-height:100vh;padding:20px;-webkit-font-smoothing:antialiased;overflow-x:hidden}\
         .blob{position:fixed;border-radius:50%;opacity:0.08;filter:blur(60px);z-index:0;pointer-events:none}\
         .blob-1{width:300px;height:300px;background:#FF6B6B;top:-100px;right:-50px}\
         .blob-2{width:250px;height:250px;background:#FFB347;bottom:-80px;left:-60px}\
         .wrap{position:relative;z-index:1;max-width:420px;width:100%}\
         .brand{display:flex;align-items:center;justify-content:center;gap:10px;margin-bottom:24px}\
         .brand img{width:56px;height:56px;border-radius:14px}\
         .brand span{font-size:20px;font-weight:700;color:#FF6B6B;letter-spacing:-0.02em}\
         .card{text-align:center;padding:40px 32px;background:#fff;border-radius:16px;box-shadow:0 4px 24px rgba(0,0,0,0.06)}\
         .av{width:72px;height:72px;border-radius:50%;background:#FF6B6B;display:flex;align-items:center;justify-content:center;margin:0 auto 16px;font-size:1.5rem;font-weight:600;color:#fff;box-shadow:0 2px 12px rgba(255,107,107,0.25)}\
         .av-img{width:72px;height:72px;border-radius:50%;object-fit:cover;margin:0 auto 16px;box-shadow:0 2px 12px rgba(0,0,0,0.1)}\
         h1{font-size:1.3rem;font-weight:700;margin-bottom:8px;color:#1a1a2e}\
         .sub{color:#6b7280;margin-bottom:28px;font-size:0.95rem;line-height:1.5}\
         .btn{display:block;background:#FF6B6B;color:#fff;text-decoration:none;padding:14px 24px;border-radius:12px;font-weight:600;font-size:1rem;margin-bottom:12px;transition:background 0.2s}\
         .btn:hover{background:#e55b5b}\
         .or{color:#9ca3af;margin:8px 0;font-size:0.9rem}\
         .dl{color:#FF6B6B;text-decoration:none;font-weight:500}\
         .dl:hover{text-decoration:underline}\
         .ft{text-align:center;margin-top:24px;font-size:12px;color:#9ca3af}\
         .ft-brand{color:#FF6B6B;font-weight:600}\
         .ft a{color:#9ca3af;text-decoration:underline}\
         </style>",
    );
    h.push_str("</head><body>");
    h.push_str("<div class=\"blob blob-1\"></div><div class=\"blob blob-2\"></div>");
    h.push_str("<div class=\"wrap\">");
    h.push_str(&format!(
        "<div class=\"brand\"><img src=\"{logo_url}\" alt=\"Offrii\"><span>Offrii</span></div>"
    ));
    h.push_str("<div class=\"card\">");
    h.push_str(&avatar_html);
    h.push_str(&format!("<h1>{title}</h1>"));
    h.push_str(&format!("<p class=\"sub\">{subtitle}</p>"));
    h.push_str(&format!(
        "<a class=\"btn\" href=\"offrii://join/{token}\">{btn}</a>"
    ));
    h.push_str(&format!("<p class=\"or\">{or_text}</p>"));
    h.push_str(&format!(
        "<a class=\"dl\" href=\"https://apps.apple.com/app/offrii\">{dl_text}</a>"
    ));
    h.push_str("</div>");
    h.push_str(&format!(
        "<div class=\"ft\"><p><span class=\"ft-brand\">Offrii</span> \u{2014} {footer_slogan}<br><a href=\"https://offrii.com\">offrii.com</a></p></div>"
    ));
    h.push_str("</div></body></html>");
    h
}

fn render_join_error_html(headers: &HeaderMap) -> String {
    let lang = get_lang(headers);
    let (title, msg, back_text, footer_slogan) = if lang == "en" {
        (
            "Invalid invitation",
            "This invitation link is expired or invalid.",
            "Discover Offrii",
            "Give, share, delight",
        )
    } else {
        (
            "Invitation invalide",
            "Ce lien d\u{2019}invitation est expir\u{00e9} ou invalide.",
            "D\u{00e9}couvrir Offrii",
            "Offre, partage, fais plaisir",
        )
    };
    let logo_url = "https://cdn.offrii.com/branding/logo-1024.png";
    let html_lang = if lang == "en" { "en" } else { "fr" };
    let lock_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M18 8h-1V6A5 5 0 0 0 7 6v2H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2ZM9 6a3 3 0 0 1 6 0v2H9V6Zm9 14H6V10h12v10Zm-6-3a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"/></svg>"#;

    let mut h = String::with_capacity(4096);
    h.push_str(&format!(
        "<!DOCTYPE html><html lang=\"{html_lang}\"><head><meta charset=\"utf-8\">"
    ));
    h.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    h.push_str(&format!("<title>{title} \u{2014} Offrii</title>"));
    h.push_str("<meta name=\"theme-color\" content=\"#FF6B6B\">");
    h.push_str("<link rel=\"icon\" type=\"image/png\" sizes=\"32x32\" href=\"/favicon.png\">");
    h.push_str("<link rel=\"icon\" type=\"image/x-icon\" href=\"/favicon.ico\">");
    h.push_str(
        "<style>\
         *{margin:0;padding:0;box-sizing:border-box}\
         body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;background:#FFFAF9;display:flex;justify-content:center;align-items:center;min-height:100vh;padding:20px;-webkit-font-smoothing:antialiased;overflow-x:hidden}\
         .blob{position:fixed;border-radius:50%;opacity:0.08;filter:blur(60px);z-index:0;pointer-events:none}\
         .blob-1{width:300px;height:300px;background:#FF6B6B;top:-100px;right:-50px}\
         .blob-2{width:250px;height:250px;background:#FFB347;bottom:-80px;left:-60px}\
         .wrap{position:relative;z-index:1;max-width:420px;width:100%}\
         .brand{display:flex;align-items:center;justify-content:center;gap:10px;margin-bottom:24px}\
         .brand img{width:56px;height:56px;border-radius:14px}\
         .brand span{font-size:20px;font-weight:700;color:#FF6B6B;letter-spacing:-0.02em}\
         .card{text-align:center;padding:40px 32px;background:#fff;border-radius:16px;box-shadow:0 4px 24px rgba(0,0,0,0.06)}\
         .icon{width:64px;height:64px;margin:0 auto 20px;border-radius:50%;display:flex;align-items:center;justify-content:center;background:#FFE8E8}\
         .icon svg{width:32px;height:32px;fill:#FF6B6B}\
         h1{font-size:1.3rem;font-weight:700;color:#1a1a2e;margin-bottom:8px}\
         p{color:#6b7280;font-size:.9rem;line-height:1.5;margin-bottom:24px}\
         a.btn{display:inline-block;padding:12px 28px;background:#FF6B6B;color:#fff;border-radius:12px;text-decoration:none;font-size:.9rem;font-weight:600;transition:background 0.2s}\
         a.btn:hover{background:#e55b5b}\
         .ft{text-align:center;margin-top:24px;font-size:12px;color:#9ca3af}\
         .ft-brand{color:#FF6B6B;font-weight:600}\
         .ft a{color:#9ca3af;text-decoration:underline}\
         </style>",
    );
    h.push_str("</head><body>");
    h.push_str("<div class=\"blob blob-1\"></div><div class=\"blob blob-2\"></div>");
    h.push_str("<div class=\"wrap\">");
    h.push_str(&format!(
        "<div class=\"brand\"><img src=\"{logo_url}\" alt=\"Offrii\"><span>Offrii</span></div>"
    ));
    h.push_str("<div class=\"card\">");
    h.push_str(&format!("<div class=\"icon\">{lock_svg}</div>"));
    h.push_str(&format!("<h1>{title}</h1>"));
    h.push_str(&format!("<p>{msg}</p>"));
    h.push_str(&format!(
        "<a class=\"btn\" href=\"https://offrii.com\">{back_text}</a>"
    ));
    h.push_str("</div>");
    h.push_str(&format!(
        "<div class=\"ft\"><p><span class=\"ft-brand\">Offrii</span> \u{2014} {footer_slogan}<br><a href=\"https://offrii.com\">offrii.com</a></p></div>"
    ));
    h.push_str("</div></body></html>");
    h
}
