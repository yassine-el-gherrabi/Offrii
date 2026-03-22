use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::dto::categories::CategoryResponse;
use crate::dto::circles::CircleResponse;
use crate::dto::community_wishes::MyWishResponse;
use crate::dto::friends::FriendResponse;
use crate::dto::items::ItemResponse;
use crate::dto::wish_messages::MessageResponse;
use crate::models::User;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
#[validate(schema(function = "validate_avatar_url_length"))]
pub struct UpdateProfileRequest {
    #[validate(length(max = 100, message = "display name must be at most 100 characters"))]
    pub display_name: Option<String>,
    #[validate(length(
        min = 3,
        max = 30,
        message = "username must be between 3 and 30 characters"
    ))]
    pub username: Option<String>,
    #[serde(default, deserialize_with = "crate::dto::nullable::deserialize")]
    pub avatar_url: Option<Option<String>>,
}

fn validate_avatar_url_length(
    req: &UpdateProfileRequest,
) -> Result<(), validator::ValidationError> {
    if let Some(Some(ref url)) = req.avatar_url
        && url.len() > 2048
    {
        let mut err = validator::ValidationError::new("avatar_url_too_long");
        err.message = Some("avatar URL must be at most 2048 characters".into());
        return Err(err);
    }
    Ok(())
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub username_customized: bool,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UserDataExport {
    pub profile: UserProfileResponse,
    pub items: Vec<ItemResponse>,
    pub categories: Vec<CategoryResponse>,
    pub circles: Vec<CircleResponse>,
    pub friends: Vec<FriendResponse>,
    pub community_wishes: Vec<MyWishResponse>,
    pub wish_messages: Vec<MessageResponse>,
    pub exported_at: DateTime<Utc>,
}

impl From<&User> for UserProfileResponse {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            username_customized: u.username_customized,
            email_verified: u.email_verified,
            avatar_url: u.avatar_url.clone(),
            created_at: u.created_at,
        }
    }
}
