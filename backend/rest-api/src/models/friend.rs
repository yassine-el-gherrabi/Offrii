use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct FriendRequest {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct Friendship {
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
    pub created_at: DateTime<Utc>,
}
