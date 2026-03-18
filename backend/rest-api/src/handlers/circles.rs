use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::circles::{
    AddMemberRequest, CircleDetailResponse, CircleEventResponse, CircleItemResponse,
    CircleResponse, CreateCircleRequest, CreateInviteRequest, FeedQuery, InviteResponse,
    JoinResponse, ReservationResponse, ShareItemRequest, TransferOwnershipRequest,
    UpdateCircleRequest,
};
use crate::dto::pagination::{PaginatedResponse, normalize_pagination};
use crate::errors::AppError;
use crate::middleware::AuthUser;

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
}

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

#[tracing::instrument(skip(state))]
async fn list_circles(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<CircleResponse>>, AppError> {
    let response = state.circles.list_circles(auth_user.user_id).await?;
    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn get_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CircleDetailResponse>, AppError> {
    let response = state.circles.get_circle(id, auth_user.user_id).await?;
    Ok(Json(response))
}

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

#[tracing::instrument(skip(state))]
async fn delete_circle(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.circles.delete_circle(id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

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

#[tracing::instrument(skip(state))]
async fn list_invites(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<InviteResponse>>, AppError> {
    let base_url = &state.app_base_url;
    let response: Vec<InviteResponse> = state
        .circles
        .list_invites(id, auth_user.user_id)
        .await?
        .into_iter()
        .map(|r| r.with_url(base_url))
        .collect();
    Ok(Json(response))
}

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

#[tracing::instrument(skip(state))]
async fn get_share_rule(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::dto::circles::ShareRuleResponse>, AppError> {
    // Membership check
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM circle_members WHERE circle_id = $1 AND user_id = $2)",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if !is_member {
        return Err(AppError::Forbidden("not a member of this circle".into()));
    }

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

#[tracing::instrument(skip(state))]
async fn set_share_rule(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<crate::dto::circles::SetShareRuleRequest>,
) -> Result<StatusCode, AppError> {
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM circle_members WHERE circle_id = $1 AND user_id = $2)",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if !is_member {
        return Err(AppError::Forbidden("not a member of this circle".into()));
    }

    // Validate share_mode
    let valid_modes = ["none", "all", "categories", "selection"];
    if !valid_modes.contains(&req.share_mode.as_str()) {
        return Err(AppError::BadRequest(
            "share_mode must be one of: none, all, categories, selection".into(),
        ));
    }

    // Clean up circle_items when switching away from selection mode
    // (or stopping sharing entirely)
    if req.share_mode != "selection" {
        sqlx::query("DELETE FROM circle_items WHERE circle_id = $1 AND shared_by = $2")
            .bind(id)
            .bind(auth_user.user_id)
            .execute(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
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

    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn list_circle_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::dto::circles::CircleItemResponse>>, AppError> {
    let response = state
        .circles
        .list_circle_items(id, auth_user.user_id)
        .await?;
    Ok(Json(response))
}

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

#[tracing::instrument(skip(state))]
async fn list_reservations(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<ReservationResponse>>, AppError> {
    let reservations = state.circles.list_reservations(auth_user.user_id).await?;
    Ok(Json(reservations))
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
    let (title, subtitle, btn, or_text, dl_text) = if lang == "en" {
        (
            format!("Join \"{}\"", circle_name),
            "You've been invited to join a circle on Offrii.",
            "Join circle",
            "or",
            "Download Offrii",
        )
    } else {
        (
            format!("Rejoindre \"{}\"", circle_name),
            "Vous avez été invité à rejoindre un cercle sur Offrii.",
            "Rejoindre le cercle",
            "ou",
            "Télécharger Offrii",
        )
    };

    let escaped_name = circle_name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    format!(
        r#"<!DOCTYPE html><html lang="{lang}"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — Offrii</title>
<meta property="og:title" content="{title}">
<meta property="og:description" content="{subtitle}">
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'><circle cx='16' cy='16' r='16' fill='%23FF6B6B'/><text x='16' y='22' text-anchor='middle' fill='white' font-size='18' font-weight='bold' font-family='system-ui'>O</text></svg>">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:-apple-system,system-ui,sans-serif;background:#FFFAF9;display:flex;justify-content:center;align-items:center;min-height:100vh;padding:20px}}
.c{{max-width:400px;text-align:center}}
.logo{{font-size:1.5rem;font-weight:800;color:#FF6B6B;letter-spacing:-0.02em;margin-bottom:24px}}
.av{{width:72px;height:72px;border-radius:50%;background:#FF6B6B;display:flex;align-items:center;justify-content:center;margin:0 auto 16px;font-size:1.5rem;font-weight:600;color:#fff}}
.av-img{{width:72px;height:72px;border-radius:50%;object-fit:cover;margin:0 auto 16px}}
h1{{font-size:1.3rem;font-weight:700;margin-bottom:8px;color:#1a1a1a}}
.sub{{color:#666;margin-bottom:24px;font-size:0.95rem}}
.btn{{display:block;background:#FF6B6B;color:#fff;text-decoration:none;padding:14px 24px;border-radius:12px;font-weight:600;font-size:1rem;margin-bottom:12px}}
.btn:hover{{background:#e55a5a}}
.or{{color:#999;margin:8px 0;font-size:0.9rem}}
.dl{{color:#FF6B6B;text-decoration:none;font-weight:500}}
</style></head><body>
<div class="c">
<div class="logo">offrii</div>
{avatar_html}
<h1>{title}</h1>
<p class="sub">{subtitle}</p>
<a class="btn" href="offrii://join/{token}">{btn}</a>
<p class="or">{or_text}</p>
<a class="dl" href="https://apps.apple.com/app/offrii">{dl_text}</a>
</div></body></html>"#,
        lang = lang,
        title = title,
        subtitle = subtitle,
        btn = btn,
        or_text = or_text,
        dl_text = dl_text,
        token = token,
        avatar_html = if let Some(img) = circle_image {
            format!("<img class=\"av-img\" src=\"{}\" alt=\"\">", img)
        } else {
            format!(
                "<div class=\"av\">{}</div>",
                escaped_name.chars().next().unwrap_or('?').to_uppercase()
            )
        },
    )
}

fn render_join_error_html(headers: &HeaderMap) -> String {
    let lang = get_lang(headers);
    let (title, msg) = if lang == "en" {
        (
            "Invalid invitation",
            "This invitation link is expired or invalid.",
        )
    } else {
        (
            "Invitation invalide",
            "Ce lien d'invitation est expiré ou invalide.",
        )
    };
    format!(
        r#"<!DOCTYPE html><html><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — Offrii</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:-apple-system,system-ui,sans-serif;background:#FFFAF9;display:flex;justify-content:center;align-items:center;min-height:100vh;padding:20px}}
.c{{max-width:400px;text-align:center}}
.logo{{font-size:1.5rem;font-weight:800;color:#FF6B6B;margin-bottom:24px}}
h1{{font-size:1.2rem;color:#1a1a1a;margin-bottom:8px}}
p{{color:#666}}
</style></head><body>
<div class="c"><div class="logo">offrii</div><h1>{title}</h1><p>{msg}</p></div>
</body></html>"#,
        title = title,
        msg = msg
    )
}
