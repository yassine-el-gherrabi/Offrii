use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, max = 2000, message = "message must be 1-2000 characters"))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListMessagesQuery {
    #[validate(range(min = 1, max = 50, message = "limit must be 1-50"))]
    pub limit: Option<i64>,
    #[validate(range(min = 0, message = "offset must be >= 0"))]
    pub offset: Option<i64>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub sender_display_name: String,
    pub is_mine: bool,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub total: i64,
}
