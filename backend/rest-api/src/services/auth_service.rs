use std::sync::Arc;

use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{User, UserResponse};
use crate::repositories::{category_repo, refresh_token_repo, user_repo};
use crate::services::token_cache;
use crate::utils::hash;
use crate::utils::jwt::{JwtKeys, REFRESH_TOKEN_TTL_SECS};
use crate::utils::token_hash::sha256_hex;

#[derive(serde::Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

#[derive(serde::Serialize)]
pub struct AuthResponse {
    pub tokens: TokenPair,
    pub user: UserResponse,
}

#[derive(serde::Serialize)]
pub struct RefreshResponse {
    pub tokens: TokenPair,
}

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

pub async fn register(
    db: &PgPool,
    redis: &redis::Client,
    jwt: &Arc<JwtKeys>,
    email: &str,
    password: &str,
    display_name: Option<&str>,
) -> Result<AuthResponse, AppError> {
    // Check email uniqueness
    if user_repo::find_by_email(db, email).await?.is_some() {
        return Err(AppError::Conflict("email already registered".into()));
    }

    // Hash password on blocking thread (CPU-bound)
    let password_owned = password.to_string();
    let password_hash = tokio::task::spawn_blocking(move || hash::hash_password(&password_owned))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
        .map_err(AppError::Internal)?;

    // Transaction: create user + copy categories + insert refresh token
    let mut tx = db.begin().await.map_err(|e| AppError::Internal(e.into()))?;

    let user = user_repo::create_user(&mut *tx, email, &password_hash, display_name)
        .await
        .map_err(AppError::Internal)?;

    category_repo::copy_defaults_for_user(&mut *tx, user.id)
        .await
        .map_err(AppError::Internal)?;

    let tokens = generate_token_pair(jwt, user.id).map_err(AppError::Internal)?;
    let refresh_hash = sha256_hex(&tokens.refresh_token);
    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS as i64);

    refresh_token_repo::insert(&mut *tx, user.id, &refresh_hash, expires_at)
        .await
        .map_err(AppError::Internal)?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // Redis cache (best-effort)
    token_cache::store(redis, &refresh_hash, user.id).await;

    Ok(AuthResponse {
        tokens,
        user: UserResponse::from(&user),
    })
}

pub async fn login(
    db: &PgPool,
    redis: &redis::Client,
    jwt: &Arc<JwtKeys>,
    email: &str,
    password: &str,
) -> Result<AuthResponse, AppError> {
    let invalid_credentials = || AppError::Unauthorized("invalid email or password".into());

    let user: User = user_repo::find_by_email(db, email)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(invalid_credentials)?;

    // Verify password on blocking thread
    let password_owned = password.to_string();
    let hash_owned = user.password_hash.clone();
    let valid =
        tokio::task::spawn_blocking(move || hash::verify_password(&password_owned, &hash_owned))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
            .map_err(AppError::Internal)?;

    if !valid {
        return Err(invalid_credentials());
    }

    let tokens = generate_token_pair(jwt, user.id).map_err(AppError::Internal)?;
    let refresh_hash = sha256_hex(&tokens.refresh_token);
    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS as i64);

    refresh_token_repo::insert(db, user.id, &refresh_hash, expires_at)
        .await
        .map_err(AppError::Internal)?;

    // Redis cache (best-effort)
    token_cache::store(redis, &refresh_hash, user.id).await;

    Ok(AuthResponse {
        tokens,
        user: UserResponse::from(&user),
    })
}

pub async fn refresh(
    db: &PgPool,
    redis: &redis::Client,
    jwt: &Arc<JwtKeys>,
    raw_refresh_token: &str,
) -> Result<RefreshResponse, AppError> {
    // Validate the JWT structure of the refresh token
    let claims = jwt
        .validate_refresh_token(raw_refresh_token)
        .map_err(|_| AppError::Unauthorized("invalid or expired refresh token".into()))?;

    let user_id: Uuid = claims
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("invalid token subject".into()))?;

    let old_hash = sha256_hex(raw_refresh_token);

    // Check Redis first, fallback to DB
    let cached_user_id = token_cache::get(redis, &old_hash).await;
    if cached_user_id.is_none() {
        // Fallback to DB
        let db_token = refresh_token_repo::find_active_by_hash(db, &old_hash)
            .await
            .map_err(AppError::Internal)?;

        if db_token.is_none() {
            return Err(AppError::Unauthorized(
                "refresh token revoked or not found".into(),
            ));
        }
    }

    // Revoke old token
    refresh_token_repo::revoke_by_hash(db, &old_hash)
        .await
        .map_err(AppError::Internal)?;
    token_cache::delete(redis, &old_hash).await;

    // Generate new pair
    let tokens = generate_token_pair(jwt, user_id).map_err(AppError::Internal)?;
    let new_hash = sha256_hex(&tokens.refresh_token);
    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS as i64);

    refresh_token_repo::insert(db, user_id, &new_hash, expires_at)
        .await
        .map_err(AppError::Internal)?;

    token_cache::store(redis, &new_hash, user_id).await;

    Ok(RefreshResponse { tokens })
}

pub async fn logout(db: &PgPool, redis: &redis::Client, user_id: Uuid) -> Result<(), AppError> {
    let revoked_hashes = refresh_token_repo::revoke_all_for_user(db, user_id)
        .await
        .map_err(AppError::Internal)?;

    token_cache::delete_many(redis, &revoked_hashes).await;

    Ok(())
}
