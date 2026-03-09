use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct SendFriendRequestBody {
    #[validate(length(min = 1, max = 50, message = "username must be 1-50 characters"))]
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct UserSearchQuery {
    pub q: String,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FriendRequestResponse {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub from_username: String,
    pub from_display_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SentFriendRequestResponse {
    pub id: Uuid,
    pub to_user_id: Uuid,
    pub to_username: String,
    pub to_display_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FriendResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub since: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserSearchResult {
    pub username: String,
    pub display_name: Option<String>,
}
