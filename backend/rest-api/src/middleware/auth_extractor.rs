use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;

/// Extracted from the `Authorization: Bearer <access_token>` header.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub jti: String,
    pub exp: usize,
}

/// Parse a Bearer token from the Authorization header value (case-insensitive scheme).
fn parse_bearer_token(header_value: &str) -> Result<&str, AppError> {
    header_value
        .split_once(' ')
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
        .map(|(_, token)| token)
        .ok_or_else(|| AppError::Unauthorized("invalid authorization header format".into()))
}

impl AuthUser {
    /// Shared logic for validating a Bearer token header value.
    async fn from_header(header: &str, state: &AppState) -> Result<Self, AppError> {
        let token = parse_bearer_token(header)?;

        let claims = state
            .jwt
            .validate_access_token(token)
            .map_err(|_| AppError::Unauthorized("invalid or expired access token".into()))?;

        let user_id: Uuid = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("invalid token subject".into()))?;

        // Redis-based revocation checks (fail-closed if Redis unavailable)
        let mut conn = state
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Redis unavailable - cannot verify token revocation");
                AppError::Internal(anyhow::anyhow!("authentication service unavailable"))
            })?;

        // Pipeline: check JTI blacklist + token version in one round-trip
        let blacklist_key = format!("blacklist:{}", claims.jti);
        let tkver_key = format!("tkver:{user_id}");

        let (blacklisted, cached_version): (i64, Option<i32>) = redis::pipe()
            .cmd("EXISTS")
            .arg(&blacklist_key)
            .cmd("GET")
            .arg(&tkver_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Redis query failed during token validation");
                AppError::Internal(anyhow::anyhow!("authentication service unavailable"))
            })?;

        if blacklisted == 1 {
            return Err(AppError::Unauthorized("token has been revoked".into()));
        }
        if let Some(ver) = cached_version
            && claims.token_version < ver
        {
            return Err(AppError::Unauthorized("token version revoked".into()));
        }

        Ok(AuthUser {
            user_id,
            jti: claims.jti,
            exp: claims.exp,
        })
    }
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

        Self::from_header(header, state).await
    }
}

/// Extracted from the `Authorization: Bearer <access_token>` header.
/// Returns `None` if no token is present (public access), or `Some(AuthUser)` if valid.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let Some(header) = header else {
            return Ok(OptionalAuthUser(None));
        };

        // If a header is present, it must be valid
        let auth_user = AuthUser::from_header(header, state).await?;
        Ok(OptionalAuthUser(Some(auth_user)))
    }
}

/// Admin-only extractor. Validates that the authenticated user has is_admin = true.
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        // Check admin status via a quick DB query
        let is_admin: Option<(bool,)> = sqlx::query_as("SELECT is_admin FROM users WHERE id = $1")
            .bind(auth_user.user_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

        match is_admin {
            Some((true,)) => Ok(AdminUser {
                user_id: auth_user.user_id,
            }),
            _ => Err(AppError::Forbidden("admin access required".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bearer_valid() {
        assert_eq!(parse_bearer_token("Bearer abc").unwrap(), "abc");
    }

    #[test]
    fn parse_bearer_lowercase() {
        assert_eq!(parse_bearer_token("bearer abc").unwrap(), "abc");
    }

    #[test]
    fn parse_bearer_uppercase() {
        assert_eq!(parse_bearer_token("BEARER abc").unwrap(), "abc");
    }

    #[test]
    fn parse_bearer_mixed_case() {
        assert_eq!(parse_bearer_token("BeArEr abc").unwrap(), "abc");
    }

    #[test]
    fn parse_bearer_missing_scheme() {
        assert!(parse_bearer_token("abc").is_err());
    }

    #[test]
    fn parse_bearer_wrong_scheme() {
        assert!(parse_bearer_token("Basic abc").is_err());
    }

    #[test]
    fn parse_bearer_empty() {
        assert!(parse_bearer_token("").is_err());
    }

    #[test]
    fn parse_bearer_preserves_token_content() {
        let token = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.sig";
        assert_eq!(
            parse_bearer_token(&format!("Bearer {token}")).unwrap(),
            token
        );
    }
}
