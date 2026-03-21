use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Item {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub estimated_price: Option<Decimal>,
    pub priority: i16,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub purchased_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub og_image_url: Option<String>,
    pub og_title: Option<String>,
    pub og_site_name: Option<String>,
    pub is_private: bool,
    pub claimed_via: Option<String>,
    pub claimed_name: Option<String>,
    pub claimed_via_link_id: Option<Uuid>,
    pub web_claim_token: Option<Uuid>,
}
