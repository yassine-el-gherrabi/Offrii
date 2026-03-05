use std::sync::{Arc, LazyLock};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse, TokenPair, UserResponse};
use crate::errors::AppError;
use crate::models::User;
use crate::repositories::{category_repo, refresh_token_repo, user_repo};
use crate::traits;
use crate::utils::hash;
use crate::utils::jwt::{JwtKeys, REFRESH_TOKEN_TTL_SECS};
use crate::utils::token_hash::sha256_hex;

/// Maximum number of active refresh tokens kept per user.
const MAX_REFRESH_TOKENS_PER_USER: i64 = 5;

/// Pre-computed Argon2id hash used as a timing side-channel countermeasure.
/// When a login attempt targets a non-existent email, we still run Argon2
/// verification against this dummy hash so the response time is
/// indistinguishable from a "wrong password" attempt.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash::hash_password("timing-safe-dummy").expect("failed to generate dummy hash")
});

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

fn refresh_expires_at() -> chrono::DateTime<Utc> {
    Utc::now() + TimeDelta::try_seconds(REFRESH_TOKEN_TTL_SECS as i64).expect("valid refresh TTL")
}

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgAuthService {
    /// Held for transactional operations (`register`, `refresh`).
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
    #[tracing::instrument(skip(self, password))]
    async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> Result<AuthResponse, AppError> {
        let email = email.trim().to_lowercase();

        // Hash password on blocking thread (CPU-bound)
        let password_owned = password.to_string();
        let password_hash =
            tokio::task::spawn_blocking(move || hash::hash_password(&password_owned))
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
                .map_err(AppError::Internal)?;

        // Transaction: create user + copy categories + insert refresh token
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let user = user_repo::create_user(&mut *tx, &email, &password_hash, display_name)
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

        refresh_token_repo::insert(&mut *tx, user.id, &refresh_hash, refresh_expires_at())
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

    #[tracing::instrument(skip(self, password))]
    async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, AppError> {
        let email = email.trim().to_lowercase();
        let invalid_credentials = || AppError::Unauthorized("invalid email or password".into());

        let user: Option<User> = self
            .user_repo
            .find_by_email(&email)
            .await
            .map_err(AppError::Internal)?;

        // Always run Argon2 verify — even when user doesn't exist — to prevent
        // timing side-channel attacks that reveal whether an email is registered.
        let password_hash_to_check = match &user {
            Some(u) => u.password_hash.clone(),
            None => DUMMY_HASH.clone(),
        };

        let password_owned = password.to_string();
        let valid = tokio::task::spawn_blocking(move || {
            hash::verify_password(&password_owned, &password_hash_to_check)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
        .map_err(AppError::Internal)?;

        let user = match (valid, user) {
            (true, Some(u)) => u,
            _ => return Err(invalid_credentials()),
        };

        let tokens = generate_token_pair(&self.jwt, user.id).map_err(AppError::Internal)?;
        let refresh_hash = sha256_hex(&tokens.refresh_token);

        self.refresh_token_repo
            .insert(user.id, &refresh_hash, refresh_expires_at())
            .await
            .map_err(AppError::Internal)?;

        // Enforce maximum active refresh tokens per user
        self.refresh_token_repo
            .revoke_excess_for_user(user.id, MAX_REFRESH_TOKENS_PER_USER)
            .await
            .map_err(AppError::Internal)?;

        Ok(AuthResponse {
            tokens,
            user: UserResponse::from(&user),
        })
    }

    #[tracing::instrument(skip(self, raw_refresh_token))]
    async fn refresh(&self, raw_refresh_token: &str) -> Result<RefreshResponse, AppError> {
        let claims = self
            .jwt
            .validate_refresh_token(raw_refresh_token)
            .map_err(|_| AppError::Unauthorized("invalid or expired refresh token".into()))?;

        let user_id: Uuid = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("invalid token subject".into()))?;

        let old_hash = sha256_hex(raw_refresh_token);

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Atomic revoke (UPDATE acquires row lock, prevents concurrent refresh)
        let revoked = refresh_token_repo::revoke_by_hash_for_user(&mut *tx, &old_hash, user_id)
            .await
            .map_err(AppError::Internal)?;

        if !revoked {
            return Err(AppError::Unauthorized(
                "refresh token revoked or not found".into(),
            ));
        }

        let tokens = generate_token_pair(&self.jwt, user_id).map_err(AppError::Internal)?;
        let new_hash = sha256_hex(&tokens.refresh_token);

        refresh_token_repo::insert(&mut *tx, user_id, &new_hash, refresh_expires_at())
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(RefreshResponse { tokens })
    }

    #[tracing::instrument(skip(self))]
    async fn logout(&self, user_id: Uuid) -> Result<(), AppError> {
        self.refresh_token_repo
            .revoke_all_for_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(())
    }
}
