use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::RefreshToken;

pub async fn insert(
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

pub async fn find_active_by_hash(
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

pub async fn revoke_by_hash(exec: impl PgExecutor<'_>, token_hash: &str) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(exec)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Revoke all active refresh tokens for a user. Returns the hashes that were revoked.
pub async fn revoke_all_for_user(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<Vec<String>> {
    let hashes: Vec<(String,)> = sqlx::query_as(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = NOW()
        WHERE user_id = $1 AND revoked_at IS NULL
        RETURNING token_hash
        "#,
    )
    .bind(user_id)
    .fetch_all(exec)
    .await?;

    Ok(hashes.into_iter().map(|(h,)| h).collect())
}
