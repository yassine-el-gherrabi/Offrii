use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{ACCEPT, HeaderMap};
use axum::response::{Html, IntoResponse, Response};
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

/// Check if the Accept header explicitly requests HTML.
/// Returns JSON by default (for API clients, `*/*`, or missing Accept header).
fn wants_html(headers: &HeaderMap) -> bool {
    if let Some(accept) = headers.get(ACCEPT)
        && let Ok(val) = accept.to_str()
    {
        return val.contains("text/html");
    }
    false
}

#[tracing::instrument(skip(state, headers))]
async fn get_shared_view(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<Response, AppError> {
    let response = state.share_links.get_shared_view(&token).await?;

    if wants_html(&headers) {
        let html = render_shared_view_html(&response, &token);
        Ok(Html(html).into_response())
    } else {
        Ok(Json(response).into_response())
    }
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

fn render_shared_view_html(view: &SharedViewResponse, token: &str) -> String {
    let mut items_html = String::new();

    for item in &view.items {
        let claimed_class = if item.is_claimed { " claimed" } else { "" };
        let claimed_badge = if item.is_claimed {
            r#"<span class="badge">Réservé</span>"#
        } else {
            ""
        };

        let price_html = item
            .estimated_price
            .map(|p| {
                let escaped = html_escape(&p.to_string());
                format!(r#"<span class="price">{escaped} €</span>"#)
            })
            .unwrap_or_default();

        let desc_html = item
            .description
            .as_deref()
            .map(|d| {
                let escaped = html_escape(d);
                format!(r#"<p class="desc">{escaped}</p>"#)
            })
            .unwrap_or_default();

        let category_html = String::new();

        let claim_button = if view.permissions == "view_and_claim" && !item.is_claimed {
            format!(
                r#"<a class="btn" href="https://apps.apple.com/app/offrii/id0000000000" onclick="window.location='offrii://claim/{token}/{item_id}';return false;">Je m'en occupe</a>"#,
                item_id = item.id
            )
        } else {
            String::new()
        };

        let escaped_name = html_escape(&item.name);

        items_html.push_str(&format!(
            r#"<div class="item{claimed_class}">
  <div class="item-header">
    <h3>{escaped_name}</h3>
    {claimed_badge}
    {price_html}
  </div>
  {desc_html}
  <div class="item-meta">{category_html}</div>
  {claim_button}
</div>
"#
        ));
    }

    let escaped_username = html_escape(&view.user_username);
    let item_count = view.items.len();

    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>La wishlist de @{escaped_username} — Offrii</title>
<meta property="og:title" content="La wishlist de @{escaped_username}">
<meta property="og:description" content="{item_count} idées cadeaux sur Offrii">
<meta property="og:type" content="website">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#f8f9fa;color:#1a1a2e;line-height:1.5}}
.container{{max-width:600px;margin:0 auto;padding:16px}}
header{{text-align:center;padding:24px 0;border-bottom:1px solid #e0e0e0;margin-bottom:24px}}
header h1{{font-size:1.5rem;color:#1a1a2e}}
header p{{color:#666;font-size:0.9rem;margin-top:4px}}
.item{{background:#fff;border-radius:12px;padding:16px;margin-bottom:12px;box-shadow:0 1px 3px rgba(0,0,0,0.08)}}
.item.claimed{{opacity:0.6}}
.item.claimed h3{{text-decoration:line-through}}
.item-header{{display:flex;align-items:center;gap:8px;flex-wrap:wrap}}
.item-header h3{{font-size:1rem;flex:1}}
.badge{{background:#e8f5e9;color:#2e7d32;font-size:0.75rem;padding:2px 8px;border-radius:12px;font-weight:600}}
.price{{color:#1a73e8;font-weight:600;font-size:0.9rem}}
.desc{{color:#555;font-size:0.875rem;margin-top:6px}}
.item-meta{{margin-top:6px;display:flex;gap:8px}}
.category{{background:#f0f0f0;color:#555;font-size:0.75rem;padding:2px 8px;border-radius:8px}}
.btn{{display:inline-block;margin-top:10px;padding:8px 16px;background:#1a73e8;color:#fff;border-radius:8px;text-decoration:none;font-size:0.875rem;font-weight:500}}
.btn:hover{{background:#1557b0}}
footer{{text-align:center;padding:32px 0 16px;color:#999;font-size:0.8rem}}
footer a{{color:#1a73e8;text-decoration:none}}
</style>
</head>
<body>
<div class="container">
<header>
  <h1>La wishlist de @{escaped_username}</h1>
  <p>{item_count} idée(s) cadeau</p>
</header>
{items_html}
<footer>
  <p>Partagé via <a href="https://offrii.com">Offrii</a></p>
</footer>
</div>
</body>
</html>"#
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
