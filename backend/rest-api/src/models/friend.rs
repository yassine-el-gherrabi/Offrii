use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typed status for friend requests. Serializes to/from lowercase strings
/// and maps to VARCHAR columns in PostgreSQL.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Declined,
    Cancelled,
}

impl FriendRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Declined => "declined",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FriendRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct FriendRequest {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub status: FriendRequestStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Friendship {
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct FriendWithSince {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub since: DateTime<Utc>,
}
