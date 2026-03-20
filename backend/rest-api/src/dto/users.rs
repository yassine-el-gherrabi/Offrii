use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::dto::categories::CategoryResponse;
use crate::dto::items::ItemResponse;
use crate::models::User;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(max = 100, message = "display name must be at most 100 characters"))]
    pub display_name: Option<String>,
    #[validate(length(
        min = 3,
        max = 30,
        message = "username must be between 3 and 30 characters"
    ))]
    pub username: Option<String>,
    pub reminder_freq: Option<String>,
    pub reminder_time: Option<NaiveTime>,
    pub timezone: Option<String>,
    #[validate(length(max = 10, message = "locale must be at most 10 characters"))]
    pub locale: Option<String>,
    #[serde(default, deserialize_with = "crate::dto::nullable::deserialize")]
    pub avatar_url: Option<Option<String>>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email_verified: bool,
    pub reminder_freq: String,
    pub reminder_time: NaiveTime,
    pub timezone: String,
    pub locale: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserDataExport {
    pub profile: UserProfileResponse,
    pub items: Vec<ItemResponse>,
    pub categories: Vec<CategoryResponse>,
    pub exported_at: DateTime<Utc>,
}

impl From<&User> for UserProfileResponse {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            email_verified: u.email_verified,
            reminder_freq: u.reminder_freq.clone(),
            reminder_time: u.reminder_time,
            timezone: u.timezone.clone(),
            locale: u.locale.clone(),
            avatar_url: u.avatar_url.clone(),
            created_at: u.created_at,
        }
    }
}
