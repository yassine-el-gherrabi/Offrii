use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::items::ItemResponse;
use crate::models::ShareLink;

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ShareLinkResponse {
    pub id: Uuid,
    pub token: String,
    pub url: String,
    pub label: Option<String>,
    pub permissions: String,
    pub scope: String,
    #[schema(value_type = Option<Object>)]
    pub scope_data: Option<serde_json::Value>,
    pub is_active: bool,
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
            label: link.label,
            permissions: link.permissions,
            scope: link.scope,
            scope_data: link.scope_data,
            is_active: link.is_active,
            created_at: link.created_at,
            expires_at: link.expires_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ShareLinkListItem {
    pub id: Uuid,
    pub token: String,
    pub url: String,
    pub label: Option<String>,
    pub permissions: String,
    pub scope: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ShareLinkListItem {
    pub fn from_model(link: ShareLink, base_url: &str) -> Self {
        let url = format!("{base_url}/shared/{}", link.token);
        Self {
            id: link.id,
            token: link.token,
            url,
            label: link.label,
            permissions: link.permissions,
            scope: link.scope,
            is_active: link.is_active,
            created_at: link.created_at,
            expires_at: link.expires_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SharedViewResponse {
    pub user_username: String,
    pub user_display_name: Option<String>,
    pub user_avatar_url: Option<String>,
    pub permissions: String,
    pub items: Vec<ItemResponse>,
}

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateShareLinkRequest {
    pub expires_at: Option<DateTime<Utc>>,
    pub label: Option<String>,
    pub permissions: Option<String>,
    pub scope: Option<String>,
    #[schema(value_type = Option<Object>)]
    pub scope_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateShareLinkRequest {
    pub label: Option<String>,
    pub is_active: Option<bool>,
    pub permissions: Option<String>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
}
