use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::Item;
use crate::models::item::ItemStatus;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct CreateItemRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: String,
    #[validate(length(max = 5000, message = "description must be at most 5000 characters"))]
    pub description: Option<String>,

    pub estimated_price: Option<Decimal>,
    pub priority: Option<i16>,
    pub category_id: Option<Uuid>,
    #[validate(length(max = 2048, message = "image_url must be at most 2048 characters"))]
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub is_private: Option<bool>,
}

impl CreateItemRequest {
    /// Resolve links: use `links` if provided and non-empty.
    pub fn resolved_links(&self) -> Option<Vec<String>> {
        if let Some(ref links) = self.links {
            if links.is_empty() {
                None
            } else {
                Some(links.clone())
            }
        } else {
            None
        }
    }

    /// Validate links constraints (called manually after Validate derive).
    pub fn validate_links(&self) -> Result<(), String> {
        if let Some(ref links) = self.links {
            if links.len() > 10 {
                return Err("links must contain at most 10 entries".into());
            }
            for (i, link) in links.iter().enumerate() {
                if link.is_empty() {
                    continue;
                }
                if link.len() > 2048 {
                    return Err(format!("links[{i}] must be at most 2048 characters"));
                }
                if !crate::utils::link_validation::is_valid_link(link) {
                    return Err(format!(
                        "links[{i}] is not a valid URL (must be http(s) with a valid domain)"
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct UpdateItemRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,
    #[validate(length(max = 5000, message = "description must be at most 5000 characters"))]
    pub description: Option<String>,

    pub estimated_price: Option<Decimal>,
    pub priority: Option<i16>,
    pub category_id: Option<Uuid>,
    pub status: Option<ItemStatus>,
    #[serde(default, deserialize_with = "crate::dto::nullable::deserialize")]
    pub image_url: Option<Option<String>>,
    pub links: Option<Vec<String>>,
    pub is_private: Option<bool>,
}

impl UpdateItemRequest {
    /// Resolve links: use `links` if provided.
    pub fn resolved_links(&self) -> Option<Vec<String>> {
        self.links.clone()
    }

    pub fn validate_links(&self) -> Result<(), String> {
        if let Some(ref links) = self.links {
            if links.len() > 10 {
                return Err("links must contain at most 10 entries".into());
            }
            for (i, link) in links.iter().enumerate() {
                if link.is_empty() {
                    continue;
                }
                if link.len() > 2048 {
                    return Err(format!("links[{i}] must be at most 2048 characters"));
                }
                if !crate::utils::link_validation::is_valid_link(link) {
                    return Err(format!(
                        "links[{i}] is not a valid URL (must be http(s) with a valid domain)"
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ListItemsQuery {
    pub status: Option<String>,
    pub category_id: Option<Uuid>,
    /// Comma-separated UUIDs for multi-category filter. Takes precedence over category_id.
    pub category_ids: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl ListItemsQuery {
    /// Resolve category filter: category_ids takes precedence, else wrap category_id.
    pub fn resolved_category_ids(&self) -> Option<Vec<Uuid>> {
        if let Some(ref ids_str) = self.category_ids {
            let ids: Vec<Uuid> = ids_str
                .split(',')
                .filter_map(|s| Uuid::parse_str(s.trim()).ok())
                .collect();
            if !ids.is_empty() {
                return Some(ids);
            }
        }
        self.category_id.map(|id| vec![id])
    }
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct BatchDeleteRequest {
    pub ids: Vec<Uuid>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ItemResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,

    pub estimated_price: Option<Decimal>,
    pub priority: i16,
    pub category_id: Option<Uuid>,
    pub status: ItemStatus,
    pub purchased_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_claimed: bool,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub og_image_url: Option<String>,
    pub og_title: Option<String>,
    pub og_site_name: Option<String>,
    pub is_private: bool,
    #[serde(default)]
    pub shared_circles: Vec<SharedCircleInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed_via: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SharedCircleInfo {
    pub id: Uuid,
    pub name: String,
    pub is_direct: bool,
    pub image_url: Option<String>,
}

impl From<Item> for ItemResponse {
    fn from(i: Item) -> Self {
        Self {
            id: i.id,
            name: i.name,
            description: i.description,
            estimated_price: i.estimated_price,
            priority: i.priority,
            category_id: i.category_id,
            status: i.status,
            purchased_at: i.purchased_at,
            created_at: i.created_at,
            updated_at: i.updated_at,
            is_claimed: i.claimed_by.is_some() || i.claimed_via.is_some(),
            image_url: i.image_url,
            links: i.links,
            og_image_url: i.og_image_url,
            og_title: i.og_title,
            og_site_name: i.og_site_name,
            is_private: i.is_private,
            shared_circles: vec![], // Set by service layer
            claimed_via: i.claimed_via,
            claimed_name: i.claimed_name,
        }
    }
}
