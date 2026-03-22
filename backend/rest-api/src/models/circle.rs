use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// Typed role for circle members. Serializes to/from lowercase strings
/// and maps to VARCHAR columns in PostgreSQL.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CircleMemberRole {
    Owner,
    Member,
}

impl CircleMemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Member => "member",
        }
    }
}

impl std::fmt::Display for CircleMemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct CircleMember {
    pub circle_id: Uuid,
    pub user_id: Uuid,
    pub role: CircleMemberRole,
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
