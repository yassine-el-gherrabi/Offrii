use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::Item;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateItemRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: String,
    #[validate(length(max = 5000, message = "description must be at most 5000 characters"))]
    pub description: Option<String>,
    #[validate(length(max = 2048, message = "url must be at most 2048 characters"))]
    pub url: Option<String>,
    pub estimated_price: Option<Decimal>,
    pub priority: Option<i16>,
    pub category_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateItemRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,
    #[validate(length(max = 5000, message = "description must be at most 5000 characters"))]
    pub description: Option<String>,
    #[validate(length(max = 2048, message = "url must be at most 2048 characters"))]
    pub url: Option<String>,
    pub estimated_price: Option<Decimal>,
    pub priority: Option<i16>,
    pub category_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListItemsQuery {
    pub status: Option<String>,
    pub category_id: Option<Uuid>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub estimated_price: Option<Decimal>,
    pub priority: i16,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub purchased_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Item> for ItemResponse {
    fn from(i: Item) -> Self {
        Self {
            id: i.id,
            name: i.name,
            description: i.description,
            url: i.url,
            estimated_price: i.estimated_price,
            priority: i.priority,
            category_id: i.category_id,
            status: i.status,
            purchased_at: i.purchased_at,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsListResponse {
    pub items: Vec<ItemResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
