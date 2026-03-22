use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, max = 500, message = "message must be 1-500 characters"))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ListMessagesQuery {
    #[validate(range(min = 1, max = 100, message = "limit must be 1-100"))]
    pub limit: Option<i64>,
    #[validate(range(min = 1, message = "page must be >= 1"))]
    pub page: Option<i64>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MessageResponse {
    pub id: Uuid,
    pub sender_display_name: String,
    pub is_mine: bool,
    pub body: String,
    pub created_at: DateTime<Utc>,
}
