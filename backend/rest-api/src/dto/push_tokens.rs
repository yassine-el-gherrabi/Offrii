use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::PushToken;

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RegisterPushTokenRequest {
    #[validate(custom(function = "validate_apns_token"))]
    pub token: String,
    #[validate(custom(function = "validate_platform"))]
    pub platform: String,
}

fn validate_apns_token(token: &str) -> Result<(), validator::ValidationError> {
    if token.len() != 64 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut err = validator::ValidationError::new("invalid_token");
        err.message = Some("token must be exactly 64 hex characters".into());
        return Err(err);
    }
    Ok(())
}

fn validate_platform(platform: &str) -> Result<(), validator::ValidationError> {
    match platform {
        "ios" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_platform");
            err.message = Some("platform must be 'ios'".into());
            Err(err)
        }
    }
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn valid_token() -> String {
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd".to_string()
    }

    fn req(token: &str, platform: &str) -> RegisterPushTokenRequest {
        RegisterPushTokenRequest {
            token: token.to_string(),
            platform: platform.to_string(),
        }
    }

    // ── Token validation ────────────────────────────────────────────

    #[test]
    fn valid_64_hex_lowercase_accepted() {
        assert!(req(&valid_token(), "ios").validate().is_ok());
    }

    #[test]
    fn valid_64_hex_uppercase_accepted() {
        let token = "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6ABCD";
        assert!(req(token, "ios").validate().is_ok());
    }

    #[test]
    fn token_63_chars_rejected() {
        let token = "a".repeat(63);
        assert!(req(&token, "ios").validate().is_err());
    }

    #[test]
    fn token_65_chars_rejected() {
        let token = "a".repeat(65);
        assert!(req(&token, "ios").validate().is_err());
    }

    #[test]
    fn token_empty_rejected() {
        assert!(req("", "ios").validate().is_err());
    }

    #[test]
    fn token_non_hex_chars_rejected() {
        let token = "g".repeat(64);
        assert!(req(&token, "ios").validate().is_err());
    }

    #[test]
    fn token_with_spaces_rejected() {
        let token = format!("{: <64}", "a1b2c3");
        assert!(req(&token, "ios").validate().is_err());
    }

    // ── Platform validation ─────────────────────────────────────────

    #[test]
    fn platform_ios_accepted() {
        assert!(req(&valid_token(), "ios").validate().is_ok());
    }

    #[test]
    fn platform_android_rejected() {
        assert!(req(&valid_token(), "android").validate().is_err());
    }

    #[test]
    fn platform_ios_uppercase_rejected() {
        assert!(req(&valid_token(), "IOS").validate().is_err());
    }

    #[test]
    fn platform_ios_apple_case_rejected() {
        assert!(req(&valid_token(), "iOS").validate().is_err());
    }

    #[test]
    fn platform_empty_rejected() {
        assert!(req(&valid_token(), "").validate().is_err());
    }

    #[test]
    fn platform_with_whitespace_rejected() {
        assert!(req(&valid_token(), " ios ").validate().is_err());
    }
}
