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

    async fn find_active_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>> {
        find_active_by_hash(&self.pool, token_hash).await
    }

    async fn revoke_by_hash(&self, token_hash: &str) -> Result<bool> {
        revoke_by_hash(&self.pool, token_hash).await
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<()> {
        revoke_all_for_user(&self.pool, user_id).await
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
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(exec)
    .await?;

    Ok(rt)
}

pub(crate) async fn find_active_by_hash(
    exec: impl PgExecutor<'_>,
    token_hash: &str,
) -> Result<Option<RefreshToken>> {
    let rt = sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT * FROM refresh_tokens
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
