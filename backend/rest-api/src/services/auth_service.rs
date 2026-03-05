use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse, TokenPair};
use crate::errors::AppError;
use crate::models::{User, UserResponse};
use crate::repositories::{category_repo, refresh_token_repo, user_repo};
use crate::traits;
use crate::utils::hash;
use crate::utils::jwt::{JwtKeys, REFRESH_TOKEN_TTL_SECS};
use crate::utils::token_hash::sha256_hex;

fn generate_token_pair(jwt: &JwtKeys, user_id: Uuid) -> Result<TokenPair> {
    let access_token = jwt.generate_access_token(user_id)?;
    let refresh_token = jwt.generate_refresh_token(user_id)?;
    Ok(TokenPair {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: crate::utils::jwt::ACCESS_TOKEN_TTL_SECS,
    })
}

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgAuthService {
    /// Held for `register()`'s multi-step transaction only.
    /// Non-transactional DB access goes through the injected trait objects.
    pool: PgPool,
    user_repo: Arc<dyn traits::UserRepo>,
    refresh_token_repo: Arc<dyn traits::RefreshTokenRepo>,
    jwt: Arc<JwtKeys>,
}

/// Check if a sqlx database error is a unique-constraint violation (PG code 23505).
fn is_unique_violation(err: &anyhow::Error) -> bool {
    err.downcast_ref::<sqlx::Error>()
        .and_then(|e| match e {
            sqlx::Error::Database(db_err) => db_err.code().map(|c| c == "23505"),
            _ => None,
        })
        .unwrap_or(false)
}

impl PgAuthService {
    pub fn new(
        pool: PgPool,
        user_repo: Arc<dyn traits::UserRepo>,
        refresh_token_repo: Arc<dyn traits::RefreshTokenRepo>,
        jwt: Arc<JwtKeys>,
    ) -> Self {
        Self {
            pool,
            user_repo,
            refresh_token_repo,
            jwt,
        }
    }
}

#[async_trait]
impl traits::AuthService for PgAuthService {
    async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> Result<AuthResponse, AppError> {
        // Hash password on blocking thread (CPU-bound)
        let password_owned = password.to_string();
        let password_hash =
            tokio::task::spawn_blocking(move || hash::hash_password(&password_owned))
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
                .map_err(AppError::Internal)?;

        // Transaction: create user + copy categories + insert refresh token
        // Uses free functions directly on &mut *tx for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // The DB unique constraint on `email` is the real guard against duplicates.
        // No TOCTOU pre-check needed — we catch the violation directly.
        let user = user_repo::create_user(&mut *tx, email, &password_hash, display_name)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    AppError::Conflict("email already registered".into())
                } else {
                    AppError::Internal(e)
                }
            })?;

        category_repo::copy_defaults_for_user(&mut *tx, user.id)
            .await
            .map_err(AppError::Internal)?;

        let tokens = generate_token_pair(&self.jwt, user.id).map_err(AppError::Internal)?;
        let refresh_hash = sha256_hex(&tokens.refresh_token);
        let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS as i64);

        refresh_token_repo::insert(&mut *tx, user.id, &refresh_hash, expires_at)
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(AuthResponse {
            tokens,
            user: UserResponse::from(&user),
        })
    }

    async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, AppError> {
        let invalid_credentials = || AppError::Unauthorized("invalid email or password".into());

        let user: User = self
            .user_repo
            .find_by_email(email)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(invalid_credentials)?;

        // Verify password on blocking thread
        let password_owned = password.to_string();
        let hash_owned = user.password_hash.clone();
        let valid = tokio::task::spawn_blocking(move || {
            hash::verify_password(&password_owned, &hash_owned)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
        .map_err(AppError::Internal)?;

        if !valid {
            return Err(invalid_credentials());
        }

        let tokens = generate_token_pair(&self.jwt, user.id).map_err(AppError::Internal)?;
        let refresh_hash = sha256_hex(&tokens.refresh_token);
        let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS as i64);

        self.refresh_token_repo
            .insert(user.id, &refresh_hash, expires_at)
            .await
            .map_err(AppError::Internal)?;

        Ok(AuthResponse {
            tokens,
            user: UserResponse::from(&user),
        })
    }

    async fn refresh(&self, raw_refresh_token: &str) -> Result<RefreshResponse, AppError> {
        // Validate the JWT structure of the refresh token
        let claims = self
            .jwt
            .validate_refresh_token(raw_refresh_token)
            .map_err(|_| AppError::Unauthorized("invalid or expired refresh token".into()))?;

        let user_id: Uuid = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("invalid token subject".into()))?;

        let old_hash = sha256_hex(raw_refresh_token);

        // Verify token exists and is active in DB (source of truth)
        let db_token = self
            .refresh_token_repo
            .find_active_by_hash(&old_hash)
            .await
            .map_err(AppError::Internal)?;

        if db_token.is_none() {
            return Err(AppError::Unauthorized(
                "refresh token revoked or not found".into(),
            ));
        }

        // Revoke old token; if nothing was revoked (race condition), reject
        let revoked = self
            .refresh_token_repo
            .revoke_by_hash(&old_hash)
            .await
            .map_err(AppError::Internal)?;

        if !revoked {
            return Err(AppError::Unauthorized(
                "refresh token revoked or not found".into(),
            ));
        }

        // Generate new pair
        let tokens = generate_token_pair(&self.jwt, user_id).map_err(AppError::Internal)?;
        let new_hash = sha256_hex(&tokens.refresh_token);
        let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS as i64);

        self.refresh_token_repo
            .insert(user_id, &new_hash, expires_at)
            .await
            .map_err(AppError::Internal)?;

        Ok(RefreshResponse { tokens })
    }

    async fn logout(&self, user_id: Uuid) -> Result<(), AppError> {
        self.refresh_token_repo
            .revoke_all_for_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(())
    }
}
