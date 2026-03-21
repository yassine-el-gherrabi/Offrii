use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub circle_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub wish_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
