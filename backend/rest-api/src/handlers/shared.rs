use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::AppState;
use crate::dto::share_links::SharedViewResponse;
use crate::errors::AppError;
use crate::middleware::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{token}", get(get_shared_view))
        .route(
            "/{token}/items/{item_id}/claim",
            axum::routing::post(claim_via_share).delete(unclaim_via_share),
        )
        .route(
            "/{token}/items/{item_id}/web-claim",
            axum::routing::post(web_claim_via_share).delete(web_unclaim_via_share),
        )
}

fn wants_html(headers: &HeaderMap) -> bool {
    if let Some(accept) = headers.get(ACCEPT)
        && let Ok(val) = accept.to_str()
    {
        return val.contains("text/html");
    }
    false
}

fn get_lang(headers: &HeaderMap) -> &'static str {
    if let Some(accept) = headers.get(ACCEPT_LANGUAGE)
        && let Ok(val) = accept.to_str()
        && val.starts_with("en")
    {
        return "en";
    }
    "fr"
}

fn t<'a>(translations: &'a HashMap<&str, HashMap<&str, &str>>, lang: &str, key: &str) -> &'a str {
    translations
        .get(key)
        .and_then(|m| m.get(lang))
        .unwrap_or(&"")
}

fn build_translations() -> HashMap<&'static str, HashMap<&'static str, &'static str>> {
    let mut m: HashMap<&str, HashMap<&str, &str>> = HashMap::new();

    let mut add = |key: &'static str, fr: &'static str, en: &'static str| {
        let mut inner = HashMap::new();
        inner.insert("fr", fr);
        inner.insert("en", en);
        m.insert(key, inner);
    };

    add("wishes_of", "Les envies de", "Wishes of");
    add("wishes", "envie(s)", "wish(es)");
    add("available", "disponible(s)", "available");
    add("reserved", "r\u{00e9}serv\u{00e9}(s)", "reserved");
    add("available_badge", "Disponible", "Available");
    add("reserved_badge", "R\u{00e9}serv\u{00e9}", "Reserved");
    add("claim_btn", "Je m'en occupe", "I'll get this");
    add(
        "claim_placeholder",
        "Votre pr\u{00e9}nom",
        "Your first name",
    );
    add("claim_confirm", "Confirmer", "Confirm");
    add("claim_cancel", "Annuler", "Cancel");
    add(
        "download_title",
        "T\u{00e9}l\u{00e9}chargez Offrii pour r\u{00e9}server un cadeau et g\u{00e9}rer vos propres envies",
        "Download Offrii to reserve gifts and manage your own wishes",
    );
    add("download_cta", "Ouvrir dans l'app", "Open in app");
    add("footer", "Partag\u{00e9} avec", "Shared with");
    add(
        "footer_slogan",
        "Offre, partage, fais plaisir",
        "Give, share, delight",
    );
    add(
        "expired_title",
        "Ce lien a expir\u{00e9}",
        "This link has expired",
    );
    add(
        "expired_msg",
        "Le propri\u{00e9}taire de cette liste peut g\u{00e9}n\u{00e9}rer un nouveau lien de partage.",
        "The owner of this list can generate a new share link.",
    );
    add(
        "disabled_title",
        "Ce lien n'est plus actif",
        "This link is no longer active",
    );
    add(
        "disabled_msg",
        "Le propri\u{00e9}taire a d\u{00e9}sactiv\u{00e9} ce lien de partage.",
        "The owner has deactivated this share link.",
    );
    add("discover_cta", "D\u{00e9}couvrir Offrii", "Discover Offrii");
    add(
        "view_only_banner",
        "D\u{00e9}couvrez ce qui ferait plaisir, pour ne jamais manquer d'inspiration.",
        "Discover what would make them happy, so you never run out of gift ideas.",
    );
    add(
        "claim_banner",
        "R\u{00e9}servez une envie pour \u{00e9}viter les doublons. C'est anonyme, seul votre pr\u{00e9}nom appara\u{00ee}t.",
        "Reserve a wish to avoid duplicates. It's anonymous, only your first name appears.",
    );
    add("not_found_title", "Lien introuvable", "Link not found");
    add(
        "not_found_msg",
        "Ce lien de partage n'existe pas ou a \u{00e9}t\u{00e9} supprim\u{00e9}.",
        "This share link does not exist or has been deleted.",
    );
    add("reserved_by", "R\u{00e9}serv\u{00e9} par", "Reserved by");
    add(
        "unclaim_btn",
        "Annuler ma r\u{00e9}servation",
        "Cancel my reservation",
    );

    m
}

// ── Handlers ──────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, headers))]
async fn get_shared_view(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<Response, AppError> {
    let lang = get_lang(&headers);

    let result = state.share_links.get_shared_view(&token).await;

    match result {
        Ok(response) => {
            if wants_html(&headers) {
                // Fetch category icons for gradient mapping
                let cat_ids: Vec<Uuid> = response
                    .items
                    .iter()
                    .filter_map(|i| i.category_id)
                    .collect();
                let cat_icons = if !cat_ids.is_empty() {
                    fetch_category_icons(&state.db, &cat_ids).await
                } else {
                    HashMap::new()
                };
                let html = render_shared_view_html(&response, lang, &cat_icons);
                Ok(Html(html).into_response())
            } else {
                Ok(Json(response).into_response())
            }
        }
        Err(err) => {
            if wants_html(&headers) {
                let html = render_error_html(&err, lang);
                Ok(Html(html).into_response())
            } else {
                Err(err)
            }
        }
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

#[derive(Debug, Deserialize)]
struct WebClaimRequest {
    name: String,
}

#[tracing::instrument(skip(state))]
async fn web_claim_via_share(
    State(state): State<AppState>,
    Path((token, item_id)): Path<(String, Uuid)>,
    Json(body): Json<WebClaimRequest>,
) -> Result<Response, AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::BadRequest(
            "name must be between 1 and 100 characters".into(),
        ));
    }

    let web_claim_token = state
        .share_links
        .web_claim_via_share(&token, item_id, &name)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "web_claim_token": web_claim_token })),
    )
        .into_response())
}

#[derive(Debug, Deserialize)]
struct WebUnclaimRequest {
    web_claim_token: Uuid,
}

#[tracing::instrument(skip(state))]
async fn web_unclaim_via_share(
    State(state): State<AppState>,
    Path((token, item_id)): Path<(String, Uuid)>,
    Json(body): Json<WebUnclaimRequest>,
) -> Result<StatusCode, AppError> {
    state
        .share_links
        .web_unclaim_via_share(&token, item_id, body.web_claim_token)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Category icon lookup ──────────────────────────────────────────────

async fn fetch_category_icons(pool: &sqlx::PgPool, ids: &[Uuid]) -> HashMap<Uuid, String> {
    let rows = sqlx::query("SELECT id, icon FROM categories WHERE id = ANY($1)")
        .bind(ids)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    rows.into_iter()
        .filter_map(|r| {
            let id: Uuid = r.get("id");
            let icon: Option<String> = r.get("icon");
            icon.map(|i| (id, i))
        })
        .collect()
}

// ── HTML rendering ────────────────────────────────────────────────────

const PAGE_CSS: &str = r#"
*{margin:0;padding:0;box-sizing:border-box}
:root{--primary:#FF6B6B;--primary-light:#FFE8E8;--accent:#FFB347;--success:#4CAF50;--success-light:#E8F5E9;--text:#1a1a2e;--text-sec:#6b7280;--text-muted:#9ca3af;--surface:#f9fafb;--card:#fff;--radius:16px;--radius-sm:10px}
body{font-family:-apple-system,BlinkMacSystemFont,'SF Pro Display','Segoe UI',Roboto,sans-serif;background:var(--surface);color:var(--text);line-height:1.5;-webkit-font-smoothing:antialiased}
.c{max-width:480px;margin:0 auto;padding:0 16px 32px}
.hd{text-align:center;padding:32px 0 24px}
.hd .lg{font-size:1.1rem;font-weight:700;color:var(--primary);letter-spacing:-0.02em;margin-bottom:16px}
.hd .av{width:56px;height:56px;border-radius:50%;background:var(--primary);display:flex;align-items:center;justify-content:center;margin:0 auto 12px;font-size:1.2rem;font-weight:600;color:#fff}
.hd .av-img{width:56px;height:56px;border-radius:50%;object-fit:cover;margin:0 auto 12px}
.hd h1{font-size:1.3rem;font-weight:700;letter-spacing:-0.01em}
.st{display:flex;justify-content:center;gap:16px;margin-top:12px;font-size:.8rem;color:var(--text-muted)}
.st strong{color:var(--text);font-weight:600}
.perm-banner{text-align:center;margin:0 0 16px;padding:12px 20px;font-size:.82rem;font-weight:400;line-height:1.5;color:var(--text-sec)}
.il{display:flex;flex-direction:column;gap:10px}
.it{background:var(--card);border-radius:var(--radius);overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,.04),0 1px 2px rgba(0,0,0,.06);transition:transform .15s,box-shadow .15s}
.it:hover{transform:translateY(-1px);box-shadow:0 4px 12px rgba(0,0,0,.08)}
.it.cl{opacity:1}
.it-img{width:100%;height:140px;object-fit:cover;display:block}
.it-grad{width:100%;height:140px;display:flex;align-items:center;justify-content:center;font-size:2.5rem}
.it-body{padding:14px 16px}
.tp{display:flex;align-items:flex-start;justify-content:space-between;gap:12px}
.inf{flex:1;min-width:0}
.nm{font-size:.95rem;font-weight:600;line-height:1.3}
.pr{display:inline-block;margin-top:4px;font-size:.85rem;font-weight:600;color:var(--primary)}
.ds{color:var(--text-sec);font-size:.8rem;margin-top:8px;line-height:1.4}
.claim-row{display:flex;align-items:center;justify-content:space-between;margin-top:8px;gap:8px}
.bc{font-size:.7rem;font-weight:500;padding:3px 10px;border-radius:20px;white-space:nowrap;background:var(--surface);color:var(--text-muted);margin-left:auto}
.ba{flex-shrink:0;font-size:.75rem;font-weight:600;padding:6px 14px;border-radius:20px;white-space:nowrap;background:var(--primary);color:#fff;border:none;cursor:pointer;transition:background .15s}
.ba:hover{background:#e55b5b}
.dt{font-size:.6rem;letter-spacing:1px;margin-right:2px}
.dh{color:#ef4444}
.dm{color:var(--accent)}
.claim-form{margin-top:10px;display:flex;flex-wrap:wrap;gap:8px;align-items:center}
.claim-form input{flex:1 1 100%;padding:8px 12px;border:1px solid #e5e7eb;border-radius:var(--radius-sm);font-size:.85rem;outline:none;transition:border .15s}
.claim-form input:focus{border-color:var(--primary)}
.claim-form .btn-confirm{flex:1;padding:8px 16px;background:var(--primary);color:#fff;border:none;border-radius:var(--radius-sm);font-size:.85rem;font-weight:600;cursor:pointer}
.claim-form .btn-cancel{flex:1;padding:8px 12px;background:#f3f4f6;color:var(--text-sec);border:none;border-radius:var(--radius-sm);font-size:.85rem;cursor:pointer}
.it-img-wrap{position:relative}
.claimed-overlay{position:absolute;top:0;left:0;right:0;bottom:0;display:flex;align-items:center;justify-content:center;font-size:1rem;font-weight:700;letter-spacing:.12em;text-transform:uppercase;color:#fff;background:rgba(0,0,0,.35);backdrop-filter:blur(4px);-webkit-backdrop-filter:blur(4px)}
.claimed-name{font-size:.75rem;font-weight:600;color:var(--primary);margin-top:6px;padding:4px 12px;background:var(--primary-light);border-radius:20px;display:inline-block}
.btn-unclaim{font-size:.75rem;font-weight:500;color:var(--text-sec);background:none;border:1px solid #e5e7eb;border-radius:20px;padding:4px 12px;cursor:pointer;transition:all .15s;white-space:nowrap}
.btn-unclaim:hover{color:var(--primary);border-color:var(--primary)}
.ct{text-align:center;margin-top:24px;padding:20px;background:var(--card);border-radius:var(--radius);box-shadow:0 1px 3px rgba(0,0,0,.04)}
.ct p{font-size:.85rem;color:var(--text-sec);margin-bottom:12px}
.ct a{display:inline-block;padding:10px 24px;background:var(--primary);color:#fff;border-radius:var(--radius-sm);text-decoration:none;font-size:.9rem;font-weight:600;transition:background .15s}
.ct a:hover{background:#e55b5b}
.ft{text-align:center;padding:24px 0;color:var(--text-muted);font-size:.75rem}
.ft a{color:var(--primary);text-decoration:none;font-weight:500}
.err-page{text-align:center;padding:80px 16px}
.err-page .icon{width:64px;height:64px;margin:0 auto 20px;border-radius:50%;display:flex;align-items:center;justify-content:center}
.err-page .icon svg{width:32px;height:32px}
.err-page .icon.expired{background:var(--primary-light)}
.err-page .icon.expired svg{fill:var(--primary)}
.err-page .icon.disabled{background:#f3f4f6}
.err-page .icon.disabled svg{fill:var(--text-muted)}
.err-page .icon.notfound{background:var(--primary-light)}
.err-page .icon.notfound svg{fill:var(--primary)}
.err-page h1{font-size:1.3rem;font-weight:700;margin-bottom:8px;color:var(--text)}
.err-page p{font-size:.9rem;color:var(--text-sec);margin-bottom:24px;line-height:1.5}
.err-page a{display:inline-block;padding:10px 24px;background:var(--primary);color:#fff;border-radius:var(--radius-sm);text-decoration:none;font-size:.9rem;font-weight:600}
.cat-laptop{background:linear-gradient(135deg,#3380E6,#66B3FF)}
.cat-tshirt{background:linear-gradient(135deg,#994DCC,#CC80FF)}
.cat-home{background:linear-gradient(135deg,#E68033,#FFB366)}
.cat-gamepad{background:linear-gradient(135deg,#4DB399,#80E6CC)}
.cat-heart{background:linear-gradient(135deg,#4DB366,#80E699)}
.cat-tag{background:linear-gradient(135deg,#808099,#B3B3CC)}
.cat-default{background:linear-gradient(135deg,#FF6B6B,#FFB347)}
.it-grad svg{width:48px;height:48px;fill:rgba(255,255,255,0.4)}
"#;

fn category_class(icon: Option<&str>) -> &'static str {
    match icon {
        Some("laptop") => "cat-laptop",
        Some("tshirt") => "cat-tshirt",
        Some("home") => "cat-home",
        Some("gamepad") => "cat-gamepad",
        Some("heart") => "cat-heart",
        Some("tag") => "cat-tag",
        _ => "cat-default",
    }
}

/// Minimal SVG icons for category gradients — clean at small sizes.
fn category_svg(icon: Option<&str>) -> &'static str {
    match icon {
        // Laptop — simple rectangle with stand
        Some("laptop") => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="12" rx="2"/><path d="M2 18h20v1a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1v-1Z"/></svg>"#
        }
        // T-shirt — simplified silhouette
        Some("tshirt") => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M16 3h3l3 4-3 2v11H5V9L2 7l3-4h3a4 4 0 0 0 8 0Z"/></svg>"#
        }
        // Home — simple house
        Some("home") => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 3 2 12h3v8h5v-5h4v5h5v-8h3L12 3Z"/></svg>"#
        }
        // Gamepad — play triangle (universal gaming symbol)
        Some("gamepad") => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M8 5v14l11-7L8 5Z"/></svg>"#
        }
        // Heart — simple filled heart
        Some("heart") => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 21C6 16.5 2 13 2 8.5 2 5.4 4.4 3 7.5 3c1.7 0 3.4.8 4.5 2.1C13.1 3.8 14.8 3 16.5 3 19.6 3 22 5.4 22 8.5 22 13 18 16.5 12 21Z"/></svg>"#
        }
        // Tag — simple price tag
        Some("tag") => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M21 12 12 3H5a2 2 0 0 0-2 2v7l9 9 9-9ZM7.5 8.5a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"/></svg>"#
        }
        // Gift — simple present box
        _ => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M3 10h18v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V10Zm8 0V6m2 4V6m-8 0h16a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1Zm4-3a2 2 0 0 1 2-2h0a2 2 0 0 1 2 2v3H7V3Zm4 0a2 2 0 0 1 2-2h0a2 2 0 0 1 2 2v3h-4V3Z"/></svg>"#
        }
    }
}

fn render_shared_view_html(
    view: &SharedViewResponse,
    lang: &str,
    cat_icons: &HashMap<Uuid, String>,
) -> String {
    let tr = build_translations();
    let mut items_html = String::new();
    for item in &view.items {
        let price = item
            .estimated_price
            .as_ref()
            .map(|p| {
                let formatted = format_price(&p.to_string(), lang);
                format!("<span class=\"pr\">{}</span>", html_escape(&formatted))
            })
            .unwrap_or_default();

        let desc = item
            .description
            .as_deref()
            .filter(|d| !d.is_empty())
            .map(|d| {
                let truncated: String = d.chars().take(120).collect();
                let suffix = if d.chars().count() > 120 {
                    "\u{2026}"
                } else {
                    ""
                };
                format!("<p class=\"ds\">{}{}</p>", html_escape(&truncated), suffix)
            })
            .unwrap_or_default();

        // Image section with optional "Réservé" overlay
        let icon = item
            .category_id
            .and_then(|cid| cat_icons.get(&cid).map(|s| s.as_str()));
        let reserved_overlay = if item.is_claimed {
            format!(
                "<div class=\"claimed-overlay\">{}</div>",
                t(&tr, lang, "reserved_badge")
            )
        } else {
            String::new()
        };
        let img_html = if let Some(ref img_url) = item.image_url {
            format!(
                "<div class=\"it-img-wrap\"><img class=\"it-img\" src=\"{}\" alt=\"\" loading=\"lazy\">{reserved_overlay}</div>",
                html_escape(img_url)
            )
        } else if let Some(ref og_url) = item.og_image_url {
            format!(
                "<div class=\"it-img-wrap\"><img class=\"it-img\" src=\"{}\" alt=\"\" loading=\"lazy\">{reserved_overlay}</div>",
                html_escape(og_url)
            )
        } else {
            let cat_cls = category_class(icon);
            let svg = category_svg(icon);
            format!(
                "<div class=\"it-img-wrap\"><div class=\"it-grad {cat_cls}\">{svg}</div>{reserved_overlay}</div>"
            )
        };

        let item_id = item.id;
        let badge = if item.is_claimed {
            if let Some(ref name) = item.claimed_name {
                format!(
                    "<div class=\"claim-row\">\
                       <button class=\"btn-unclaim\" id=\"uc-{item_id}\" style=\"display:none\" onclick=\"unclaim('{item_id}')\">{}</button>\
                       <span class=\"bc\">{} {}</span>\
                     </div>",
                    t(&tr, lang, "unclaim_btn"),
                    t(&tr, lang, "reserved_by"),
                    html_escape(name),
                )
            } else {
                String::new()
            }
        } else if view.permissions == "view_and_claim" {
            format!(
                "<button class=\"ba\" id=\"cb-{item_id}\" onclick=\"showClaim('{item_id}')\">{}</button>\
                 <div id=\"cf-{item_id}\" class=\"claim-form\" style=\"display:none\">\
                   <input type=\"text\" id=\"cn-{item_id}\" placeholder=\"{}\" maxlength=\"100\">\
                   <button class=\"btn-confirm\" onclick=\"claim('{item_id}')\">{}</button>\
                   <button class=\"btn-cancel\" onclick=\"hideClaim('{item_id}')\">{}</button>\
                 </div>",
                t(&tr, lang, "claim_btn"),
                t(&tr, lang, "claim_placeholder"),
                t(&tr, lang, "claim_confirm"),
                t(&tr, lang, "claim_cancel"),
            )
        } else {
            String::new()
        };

        let dots = match item.priority {
            3 => "<span class=\"dt dh\">\u{1F525}\u{1F525}</span> ",
            2 => "<span class=\"dt dm\">\u{1F525}</span> ",
            _ => "",
        };

        let cl = if item.is_claimed { " cl" } else { "" };
        let name = html_escape(&item.name);

        items_html.push_str(&format!(
            "<div class=\"it{cl}\">{img_html}<div class=\"it-body\"><div class=\"tp\"><div class=\"inf\"><div class=\"nm\">{dots}{name}</div>{price}</div></div>{desc}{badge}</div></div>"
        ));
    }

    let display = view
        .user_display_name
        .as_deref()
        .filter(|n| !n.is_empty())
        .unwrap_or(&view.user_username);
    let username = html_escape(&view.user_username);
    let n = view.items.len();
    let c = view.items.iter().filter(|i| i.is_claimed).count();
    let a = n - c;
    let ini = get_initials(display);

    let perm_key = if view.permissions == "view_and_claim" {
        "claim_banner"
    } else {
        "view_only_banner"
    };
    let perm_banner = format!(
        "<div class=\"perm-banner\">{}</div>",
        t(&tr, lang, perm_key)
    );

    let html_lang = if lang == "en" { "en" } else { "fr" };

    let mut h = String::with_capacity(8192);
    h.push_str(&format!(
        "<!DOCTYPE html><html lang=\"{html_lang}\"><head><meta charset=\"utf-8\">"
    ));
    h.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    h.push_str(&format!(
        "<title>{} @{username} \u{2014} Offrii</title>",
        t(&tr, lang, "wishes_of")
    ));
    h.push_str(&format!(
        "<meta property=\"og:title\" content=\"{} @{username}\">",
        t(&tr, lang, "wishes_of")
    ));
    h.push_str(&format!(
        "<meta property=\"og:description\" content=\"{n} {} \u{2014} {a} {}\">",
        t(&tr, lang, "wishes"),
        t(&tr, lang, "available")
    ));
    // og:image — use first item's image if available
    let og_image = view
        .items
        .iter()
        .find_map(|i| i.image_url.as_ref().or(i.og_image_url.as_ref()));
    if let Some(img) = og_image {
        h.push_str(&format!(
            "<meta property=\"og:image\" content=\"{}\">",
            html_escape(img)
        ));
    }
    h.push_str("<meta property=\"og:type\" content=\"website\"><meta name=\"theme-color\" content=\"#FF6B6B\">");
    // Favicon: gift icon matching the iOS ShinyIcon(systemName:"gift.fill")
    h.push_str("<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect x='2' y='14' width='28' height='15' rx='2' fill='%23FF6B6B'/%3E%3Crect x='14' y='14' width='4' height='15' fill='%23E55B5B'/%3E%3Crect x='1' y='10' width='30' height='6' rx='2' fill='%23FF6B6B'/%3E%3Crect x='14' y='10' width='4' height='6' fill='%23E55B5B'/%3E%3Cpath d='M16 10c-2-4-7-5-7-2s5 2 7 2Z' fill='%23FFB347'/%3E%3Cpath d='M16 10c2-4 7-5 7-2s-5 2-7 2Z' fill='%23FFB347'/%3E%3C/svg%3E\">");
    h.push_str("<style>");
    h.push_str(PAGE_CSS);
    h.push_str("</style></head><body><div class=\"c\">");
    h.push_str("<div class=\"hd\">");
    h.push_str("<div class=\"lg\">offrii</div>");
    if let Some(ref avatar_url) = view.user_avatar_url {
        h.push_str(&format!(
            "<img class=\"av-img\" src=\"{}\" alt=\"\">",
            html_escape(avatar_url)
        ));
    } else {
        h.push_str(&format!("<div class=\"av\">{ini}</div>"));
    }
    h.push_str(&format!(
        "<h1>{} @{username}</h1>",
        t(&tr, lang, "wishes_of")
    ));
    h.push_str(&format!(
        "<div class=\"st\"><span><strong>{n}</strong> {}</span><span><strong>{a}</strong> {}</span><span><strong>{c}</strong> {}</span></div>",
        t(&tr, lang, "wishes"),
        t(&tr, lang, "available"),
        t(&tr, lang, "reserved"),
    ));
    h.push_str("</div>");
    h.push_str(&perm_banner);
    h.push_str("<div class=\"il\">");
    h.push_str(&items_html);
    h.push_str("</div>");
    h.push_str(&format!(
        "<div class=\"ct\"><p>{}</p><a href=\"https://apps.apple.com/app/offrii/id0000000000\">{}</a></div>",
        t(&tr, lang, "download_title"),
        t(&tr, lang, "download_cta"),
    ));
    h.push_str(&format!(
        "</div><footer class=\"ft\"><p>{} <a href=\"https://offrii.com\">Offrii</a> \u{2014} {}</p></footer>",
        t(&tr, lang, "footer"),
        t(&tr, lang, "footer_slogan"),
    ));

    // JavaScript for web claim flow
    h.push_str(
        r#"<script>
function showClaim(id){
  document.getElementById('cb-'+id).style.display='none';
  var f=document.getElementById('cf-'+id);
  f.style.display='flex';
  setTimeout(function(){f.scrollIntoView({behavior:'smooth',block:'center'});},50);
  document.getElementById('cn-'+id).focus();
}
function hideClaim(id){
  document.getElementById('cf-'+id).style.display='none';
  document.getElementById('cb-'+id).style.display='';
}
async function claim(itemId){
  var name=document.getElementById('cn-'+itemId).value.trim();
  if(!name)return;
  var token=location.pathname.split('/')[2];
  try{
    var res=await fetch('/shared/'+token+'/items/'+itemId+'/web-claim',{
      method:'POST',headers:{'Content-Type':'application/json'},
      body:JSON.stringify({name:name})
    });
    if(res.ok){
      var data=await res.json();
      localStorage.setItem('claim-'+itemId,data.web_claim_token);
      location.reload();
    }else{
      var err=await res.json().catch(function(){return{error:{message:'Error'}};});
      alert(err.error&&err.error.message||'Error');
    }
  }catch(e){alert('Network error');}
}
async function unclaim(itemId){
  var claimToken=localStorage.getItem('claim-'+itemId);
  if(!claimToken)return;
  var token=location.pathname.split('/')[2];
  try{
    var res=await fetch('/shared/'+token+'/items/'+itemId+'/web-claim',{
      method:'DELETE',headers:{'Content-Type':'application/json'},
      body:JSON.stringify({web_claim_token:claimToken})
    });
    if(res.ok){
      localStorage.removeItem('claim-'+itemId);
      location.reload();
    }else{
      var err=await res.json().catch(function(){return{error:{message:'Error'}};});
      alert(err.error&&err.error.message||'Error');
    }
  }catch(e){alert('Network error');}
}
// Show unclaim buttons for items the current visitor has claimed
document.addEventListener('DOMContentLoaded',function(){
  document.querySelectorAll('[id^="uc-"]').forEach(function(btn){
    var itemId=btn.id.substring(3);
    if(localStorage.getItem('claim-'+itemId)){
      btn.style.display='';
    }
  });
});
</script>"#,
    );
    h.push_str("</body></html>");

    h
}

fn render_error_html(err: &AppError, lang: &str) -> String {
    let tr = build_translations();

    // SVG icons for error pages — clean, no emoji
    let svg_clock = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20Zm0 18a8 8 0 1 1 0-16 8 8 0 0 1 0 16Zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67V7Z"/></svg>"#;
    let svg_lock = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M18 8h-1V6A5 5 0 0 0 7 6v2H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2ZM9 6a3 3 0 0 1 6 0v2H9V6Zm9 14H6V10h12v10Zm-6-3a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"/></svg>"#;
    let svg_search = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5a6.5 6.5 0 1 0-6.5 6.5c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5Zm-6 0A4.5 4.5 0 1 1 14 9.5 4.49 4.49 0 0 1 9.5 14Z"/></svg>"#;

    let (icon_svg, icon_class, title, message) = match err {
        AppError::Gone(msg) if msg.contains("expired") => (
            svg_clock,
            "expired",
            t(&tr, lang, "expired_title"),
            t(&tr, lang, "expired_msg"),
        ),
        AppError::Gone(_) => (
            svg_lock,
            "disabled",
            t(&tr, lang, "disabled_title"),
            t(&tr, lang, "disabled_msg"),
        ),
        AppError::NotFound(_) => (
            svg_search,
            "notfound",
            t(&tr, lang, "not_found_title"),
            t(&tr, lang, "not_found_msg"),
        ),
        _ => (
            svg_search,
            "notfound",
            t(&tr, lang, "not_found_title"),
            t(&tr, lang, "not_found_msg"),
        ),
    };

    let html_lang = if lang == "en" { "en" } else { "fr" };

    format!(
        "<!DOCTYPE html><html lang=\"{html_lang}\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{title} \u{2014} Offrii</title>\
         <meta name=\"theme-color\" content=\"#FF6B6B\">\
         <style>{PAGE_CSS}</style></head><body>\
         <div class=\"c\">\
         <div class=\"hd\"><div class=\"lg\">offrii</div></div>\
         <div class=\"err-page\">\
         <div class=\"icon {icon_class}\">{icon_svg}</div>\
         <h1>{title}</h1>\
         <p>{message}</p>\
         <a href=\"https://offrii.com\">{}</a>\
         </div></div>\
         <footer class=\"ft\"><p>{} <a href=\"https://offrii.com\">Offrii</a> \u{2014} {}</p></footer>\
         </body></html>",
        t(&tr, lang, "discover_cta"),
        t(&tr, lang, "footer"),
        t(&tr, lang, "footer_slogan"),
    )
}

fn get_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.len() {
        0 => "?".to_string(),
        1 => parts[0].chars().take(2).collect::<String>().to_uppercase(),
        _ => format!(
            "{}{}",
            parts[0].chars().next().unwrap_or('?'),
            parts[1].chars().next().unwrap_or('?')
        )
        .to_uppercase(),
    }
}

/// Format a decimal price string with locale-aware separators.
/// "2999.00" → "2 999,00 €" (FR) or "2,999.00 €" (EN)
fn format_price(raw: &str, lang: &str) -> String {
    let parts: Vec<&str> = raw.split('.').collect();
    let integer = parts[0];
    let decimals = parts.get(1).unwrap_or(&"00");

    // Add thousand separators
    let digits: Vec<char> = integer.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut formatted = String::new();
    for (i, ch) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            formatted.push(if lang == "fr" { '\u{202F}' } else { ',' });
        }
        formatted.push(*ch);
    }

    if lang == "fr" {
        format!("{},{}\u{00A0}\u{20AC}", formatted, decimals)
    } else {
        format!("{}.{}\u{00A0}\u{20AC}", formatted, decimals)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use axum::http::header::ACCEPT_LANGUAGE;

    #[test]
    fn format_price_fr_thousands() {
        assert_eq!(
            format_price("2999.00", "fr"),
            "2\u{202F}999,00\u{00A0}\u{20AC}"
        );
    }

    #[test]
    fn format_price_en_thousands() {
        assert_eq!(format_price("2999.00", "en"), "2,999.00\u{00A0}\u{20AC}");
    }

    #[test]
    fn format_price_small() {
        assert_eq!(format_price("9.99", "fr"), "9,99\u{00A0}\u{20AC}");
    }

    #[test]
    fn format_price_integer() {
        assert_eq!(format_price("100", "fr"), "100,00\u{00A0}\u{20AC}");
    }

    #[test]
    fn get_initials_single_name() {
        assert_eq!(get_initials("Yassine"), "YA");
    }

    #[test]
    fn get_initials_two_names() {
        assert_eq!(get_initials("Marie Dupont"), "MD");
    }

    #[test]
    fn get_initials_empty() {
        assert_eq!(get_initials(""), "?");
    }

    #[test]
    fn html_escape_special_chars() {
        assert_eq!(
            html_escape("<script>alert('xss');</script>"),
            "&lt;script&gt;alert(&#39;xss&#39;);&lt;/script&gt;"
        );
    }

    #[test]
    fn get_lang_default_french() {
        let h = HeaderMap::new();
        assert_eq!(get_lang(&h), "fr");
    }

    #[test]
    fn get_lang_english() {
        let mut h = HeaderMap::new();
        h.insert(ACCEPT_LANGUAGE, "en-US".parse().unwrap());
        assert_eq!(get_lang(&h), "en");
    }
}
