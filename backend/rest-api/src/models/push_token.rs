use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PushToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub platform: String,
    pub created_at: DateTime<Utc>,
}
