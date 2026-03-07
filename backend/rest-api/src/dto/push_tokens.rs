use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::PushToken;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterPushTokenRequest {
    #[validate(length(min = 1, message = "token is required"))]
    pub token: String,
    #[validate(custom(function = "validate_platform"))]
    pub platform: String,
}

fn validate_platform(platform: &str) -> Result<(), validator::ValidationError> {
    match platform {
        "ios" | "android" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_platform");
            err.message = Some("platform must be 'ios' or 'android'".into());
            Err(err)
        }
    }
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PushTokenResponse {
    pub id: Uuid,
    pub token: String,
    pub platform: String,
    pub created_at: DateTime<Utc>,
}

impl From<&PushToken> for PushTokenResponse {
    fn from(pt: &PushToken) -> Self {
        Self {
            id: pt.id,
            token: pt.token.clone(),
            platform: pt.platform.clone(),
            created_at: pt.created_at,
        }
    }
}
