use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCircleRequest {
    #[validate(length(min = 1, max = 100, message = "name must be 1-100 characters"))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCircleRequest {
    #[validate(length(min = 1, max = 100, message = "name must be 1-100 characters"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ShareItemRequest {
    pub item_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CircleResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub is_direct: bool,
    pub owner_id: Uuid,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CircleDetailResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub is_direct: bool,
    pub owner_id: Uuid,
    pub members: Vec<CircleMemberResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CircleMemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InviteResponse {
    pub id: Uuid,
    pub token: String,
    pub circle_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub max_uses: i32,
    pub use_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CircleItemResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub estimated_price: Option<rust_decimal::Decimal>,
    pub priority: i16,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub is_claimed: bool,
    pub claimed_by: Option<ClaimedByInfo>,
    pub shared_at: DateTime<Utc>,
    pub shared_by: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaimedByInfo {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CircleEventResponse {
    pub id: Uuid,
    pub event_type: String,
    pub actor_id: Option<Uuid>,
    pub actor_username: Option<String>,
    pub target_item_id: Option<Uuid>,
    pub target_item_name: Option<String>,
    pub target_user_id: Option<Uuid>,
    pub target_username: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JoinResponse {
    pub circle_id: Uuid,
    pub circle_name: Option<String>,
}
