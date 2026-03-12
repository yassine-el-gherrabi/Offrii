use chrono::{DateTime, NaiveTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_provider_id: Option<String>,
    pub reminder_freq: String,
    pub reminder_time: NaiveTime,
    pub timezone: String,
    pub utc_reminder_hour: i16,
    pub locale: String,
    pub token_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
