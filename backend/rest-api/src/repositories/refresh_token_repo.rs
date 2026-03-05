use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::RefreshToken;
use crate::traits;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgRefreshTokenRepo {
    pool: PgPool,
}

impl PgRefreshTokenRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::RefreshTokenRepo for PgRefreshTokenRepo {
    async fn insert(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken> {
        insert(&self.pool, user_id, token_hash, expires_at).await
    }

    async fn revoke_by_hash(&self, token_hash: &str) -> Result<bool> {
        revoke_by_hash(&self.pool, token_hash).await
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<()> {
        revoke_all_for_user(&self.pool, user_id).await
    }

    async fn revoke_excess_for_user(&self, user_id: Uuid, keep: i64) -> Result<()> {
        revoke_excess_for_user(&self.pool, user_id, keep).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn insert(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken> {
    let rt = sqlx::query_as::<_, RefreshToken>(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, token_hash, expires_at, revoked_at, created_at
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(exec)
    .await?;

    Ok(rt)
}

#[allow(dead_code)]
pub(crate) async fn find_active_by_hash(
    exec: impl PgExecutor<'_>,
    token_hash: &str,
) -> Result<Option<RefreshToken>> {
    let rt = sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT id, user_id, token_hash, expires_at, revoked_at, created_at
        FROM refresh_tokens
        WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(exec)
    .await?;

    Ok(rt)
}

pub(crate) async fn revoke_by_hash(exec: impl PgExecutor<'_>, token_hash: &str) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(exec)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Atomically revoke a specific refresh token, verifying ownership.
///
/// Returns `true` if a row was revoked, `false` if the token was already
/// revoked, expired, or doesn't belong to the given user.
pub(crate) async fn revoke_by_hash_for_user(
    exec: impl PgExecutor<'_>,
    token_hash: &str,
    user_id: Uuid,
) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() \
         WHERE token_hash = $1 AND user_id = $2 \
         AND revoked_at IS NULL AND expires_at > NOW()",
    )
    .bind(token_hash)
    .bind(user_id)
    .execute(exec)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Revoke active refresh tokens beyond the `keep` limit (oldest first).
pub(crate) async fn revoke_excess_for_user(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    keep: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() \
         WHERE user_id = $1 AND revoked_at IS NULL \
         AND id NOT IN ( \
             SELECT id FROM refresh_tokens \
             WHERE user_id = $1 AND revoked_at IS NULL \
             ORDER BY created_at DESC LIMIT $2 \
         )",
    )
    .bind(user_id)
    .bind(keep)
    .execute(exec)
    .await?;

    Ok(())
}

/// Revoke all active refresh tokens for a user.
pub(crate) async fn revoke_all_for_user(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(exec)
    .await?;

    Ok(())
}
