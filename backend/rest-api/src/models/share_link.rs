use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ShareLink {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub label: Option<String>,
    pub permissions: String,
    pub scope: String,
    pub scope_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}
