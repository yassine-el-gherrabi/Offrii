use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, FromRow)]
pub struct FriendRequest {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub status: String,
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
