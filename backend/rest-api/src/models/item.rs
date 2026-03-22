use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typed status for wishlist items. Serializes to/from lowercase strings
/// (e.g. `"active"`) for JSON and maps to VARCHAR columns in PostgreSQL.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Active,
    Purchased,
    Deleted,
}

impl ItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Purchased => "purchased",
            Self::Deleted => "deleted",
        }
    }
}

impl std::fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Item {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub estimated_price: Option<Decimal>,
    pub priority: i16,
    pub category_id: Option<Uuid>,
    pub status: ItemStatus,
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
