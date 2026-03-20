use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct CreateCircleRequest {
    #[validate(length(min = 1, max = 100, message = "name must be 1-100 characters"))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct UpdateCircleRequest {
    #[validate(length(min = 1, max = 100, message = "name must be 1-100 characters"))]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::dto::nullable::deserialize")]
    pub image_url: Option<Option<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ShareItemRequest {
    pub item_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct BatchShareRequest {
    pub item_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct SetShareRuleRequest {
    pub share_mode: String,
    #[serde(default)]
    pub category_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ShareRuleResponse {
    pub share_mode: String,
    pub category_ids: Vec<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CircleShareRuleSummary {
    pub circle_id: Uuid,
    pub share_mode: String,
    pub category_count: usize,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct FeedQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CircleResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub is_direct: bool,
    pub owner_id: Uuid,
    pub image_url: Option<String>,
    pub member_count: i64,
    pub unreserved_item_count: i64,
    pub last_activity: Option<String>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub member_names: Vec<String>,
    pub member_ids: Vec<Uuid>,
    pub member_avatars: Vec<Option<String>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CircleDetailResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub is_direct: bool,
    pub owner_id: Uuid,
    pub image_url: Option<String>,
    pub members: Vec<CircleMemberResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CircleMemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct InviteResponse {
    pub id: Uuid,
    pub token: String,
    pub url: String,
    pub circle_id: Uuid,
    pub created_by: Uuid,
    pub created_by_name: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub max_uses: i32,
    pub use_count: i32,
    pub created_at: DateTime<Utc>,
}

impl InviteResponse {
    pub fn with_url(mut self, base_url: &str) -> Self {
        self.url = format!("{base_url}/join/{}", self.token);
        self
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CircleItemResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,

    pub estimated_price: Option<rust_decimal::Decimal>,
    pub priority: i16,
    pub category_id: Option<Uuid>,
    pub category_icon: Option<String>,
    pub status: String,
    pub is_claimed: bool,
    pub claimed_by: Option<ClaimedByInfo>,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub og_image_url: Option<String>,
    pub og_title: Option<String>,
    pub og_site_name: Option<String>,
    pub shared_at: DateTime<Utc>,
    pub shared_by: Uuid,
    pub shared_by_name: Option<String>,
    pub shared_by_avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ClaimedByInfo {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
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

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct JoinResponse {
    pub circle_id: Uuid,
    pub circle_name: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TransferOwnershipRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ReservationResponse {
    pub item_id: Uuid,
    pub item_name: String,
    pub item_image_url: Option<String>,

    pub item_estimated_price: Option<Decimal>,
    pub item_status: String,
    pub owner_name: String,
    pub owner_avatar_url: Option<String>,
    pub circle_id: Uuid,
    pub circle_name: Option<String>,
    pub claimed_at: DateTime<Utc>,
}
