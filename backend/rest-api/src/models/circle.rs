use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Circle {
    pub id: Uuid,
    pub name: Option<String>,
    pub owner_id: Uuid,
    pub is_direct: bool,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CircleMember {
    pub circle_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CircleInvite {
    pub id: Uuid,
    pub circle_id: Uuid,
    pub token: String,
    pub created_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub max_uses: i32,
    pub use_count: i32,
    pub created_at: DateTime<Utc>,
}
