use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct CircleItem {
    pub circle_id: Uuid,
    pub item_id: Uuid,
    pub shared_by: Uuid,
    pub shared_at: DateTime<Utc>,
}
