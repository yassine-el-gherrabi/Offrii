use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;

/// Extracted from the `Authorization: Bearer <access_token>` header.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing authorization header".into()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("invalid authorization header format".into()))?;

        let claims = state
            .jwt
            .validate_access_token(token)
            .map_err(|_| AppError::Unauthorized("invalid or expired access token".into()))?;

        let user_id: Uuid = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("invalid token subject".into()))?;

        Ok(AuthUser { user_id })
    }
}
