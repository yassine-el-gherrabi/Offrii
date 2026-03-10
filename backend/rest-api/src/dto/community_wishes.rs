use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWishRequest {
    #[validate(length(min = 1, max = 255, message = "title must be 1-255 characters"))]
    pub title: String,
    #[validate(length(max = 2000, message = "description must be at most 2000 characters"))]
    pub description: Option<String>,
    #[validate(custom(function = "validate_wish_category"))]
    pub category: String,
    #[serde(default)]
    pub is_anonymous: bool,
    #[validate(length(max = 2048, message = "image_url must be at most 2048 characters"))]
    pub image_url: Option<String>,
    #[validate(custom(function = "validate_links"))]
    pub links: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateWishRequest {
    #[validate(length(min = 1, max = 255, message = "title must be 1-255 characters"))]
    pub title: Option<String>,
    #[validate(length(max = 2000, message = "description must be at most 2000 characters"))]
    pub description: Option<String>,
    #[validate(custom(function = "validate_wish_category"))]
    pub category: Option<String>,
    #[validate(length(max = 2048, message = "image_url must be at most 2048 characters"))]
    pub image_url: Option<String>,
    #[validate(custom(function = "validate_links"))]
    pub links: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListWishesQuery {
    pub category: Option<String>,
    #[validate(range(min = 1, max = 100, message = "limit must be 1-100"))]
    pub limit: Option<i64>,
    #[validate(range(min = 1, message = "page must be >= 1"))]
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReportWishRequest {
    #[validate(custom(function = "validate_report_reason"))]
    pub reason: Option<String>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishResponse {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub status: String,
    pub is_mine: bool,
    pub is_matched_by_me: bool,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WishDetailResponse {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub status: String,
    pub is_mine: bool,
    pub is_matched_by_me: bool,
    pub matched_with_display_name: Option<String>,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub matched_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MyWishResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub status: String,
    pub is_anonymous: bool,
    pub matched_with_display_name: Option<String>,
    pub report_count: i32,
    pub reopen_count: i32,
    pub moderation_note: Option<String>,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub matched_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminWishResponse {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub status: String,
    pub moderation_note: Option<String>,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub report_count: i32,
    pub created_at: DateTime<Utc>,
}

// ── Validators ───────────────────────────────────────────────────────

const VALID_CATEGORIES: &[&str] = &[
    "education",
    "clothing",
    "health",
    "religion",
    "home",
    "children",
    "other",
];

fn validate_wish_category(category: &str) -> Result<(), validator::ValidationError> {
    if VALID_CATEGORIES.contains(&category) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_category");
        err.message = Some(
            "category must be one of: education, clothing, health, religion, home, children, other"
                .into(),
        );
        Err(err)
    }
}

fn validate_links(links: &Vec<String>) -> Result<(), validator::ValidationError> {
    if links.len() > 10 {
        let mut err = validator::ValidationError::new("too_many_links");
        err.message = Some("at most 10 links are allowed".into());
        return Err(err);
    }
    for link in links {
        if link.len() > 2048 {
            let mut err = validator::ValidationError::new("link_too_long");
            err.message = Some("each link must be at most 2048 characters".into());
            return Err(err);
        }
    }
    Ok(())
}

const VALID_REPORT_REASONS: &[&str] = &["inappropriate", "spam", "scam", "other"];

fn validate_report_reason(reason: &str) -> Result<(), validator::ValidationError> {
    if VALID_REPORT_REASONS.contains(&reason) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_reason");
        err.message = Some("reason must be one of: inappropriate, spam, scam, other".into());
        Err(err)
    }
}
