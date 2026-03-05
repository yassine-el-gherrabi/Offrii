use chrono::{DateTime, NaiveTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub reminder_freq: String,
    pub reminder_time: NaiveTime,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
