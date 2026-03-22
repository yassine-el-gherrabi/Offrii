use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct CircleEvent {
    pub id: Uuid,
    pub circle_id: Uuid,
    pub actor_id: Uuid,
    pub event_type: String,
    pub target_item_id: Option<Uuid>,
    pub target_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
