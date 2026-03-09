use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::items::ItemResponse;
use crate::models::ShareLink;

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLinkResponse {
    pub id: Uuid,
    pub token: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ShareLinkResponse {
    pub fn from_model(link: ShareLink, base_url: &str) -> Self {
        let url = format!("{base_url}/shared/{}", link.token);
        Self {
            id: link.id,
            token: link.token,
            url,
            created_at: link.created_at,
            expires_at: link.expires_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLinkListItem {
    pub id: Uuid,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<ShareLink> for ShareLinkListItem {
    fn from(link: ShareLink) -> Self {
        Self {
            id: link.id,
            token: link.token,
            created_at: link.created_at,
            expires_at: link.expires_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedViewResponse {
    pub user_display_name: Option<String>,
    pub items: Vec<ItemResponse>,
}

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateShareLinkRequest {
    pub expires_at: Option<DateTime<Utc>>,
}
