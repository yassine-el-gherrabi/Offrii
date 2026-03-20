use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Category;

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub is_default: bool,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            name: c.name,
            icon: c.icon,
            is_default: c.is_default,
            position: c.position,
            created_at: c.created_at,
        }
    }
}
