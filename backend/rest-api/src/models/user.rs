use chrono::{DateTime, Utc};
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
    pub email_verified: bool,
    pub token_version: i32,
    pub is_admin: bool,
    pub username_customized: bool,
    pub avatar_url: Option<String>,
    pub terms_accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
