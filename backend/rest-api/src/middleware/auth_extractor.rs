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

        let token = parse_bearer_token(header)?;

        let claims = state
            .jwt
            .validate_access_token(token)
            .map_err(|_| AppError::Unauthorized("invalid or expired access token".into()))?;

        let user_id: Uuid = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("invalid token subject".into()))?;

        // Redis-based revocation checks (fail-open if Redis unavailable)
        if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
            // Pipeline: check JTI blacklist + token version in one round-trip
            let blacklist_key = format!("blacklist:{}", claims.jti);
            let tkver_key = format!("tkver:{user_id}");

            let result: Result<(i64, Option<i32>), _> = redis::pipe()
                .cmd("EXISTS")
                .arg(&blacklist_key)
                .cmd("GET")
                .arg(&tkver_key)
                .query_async(&mut conn)
                .await;

            if let Ok((blacklisted, cached_version)) = result {
                if blacklisted == 1 {
                    return Err(AppError::Unauthorized("token has been revoked".into()));
                }
                if let Some(ver) = cached_version
                    && claims.token_version < ver
                {
                    return Err(AppError::Unauthorized("token version revoked".into()));
                }
            }
        }

        Ok(AuthUser {
            user_id,
            jti: claims.jti,
            exp: claims.exp,
        })
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
