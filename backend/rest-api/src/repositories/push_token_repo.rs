use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::PushToken;
use crate::traits;

const PT_COLS: &str = "id, user_id, token, platform, created_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgPushTokenRepo {
    pool: PgPool,
}

impl PgPushTokenRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::PushTokenRepo for PgPushTokenRepo {
    async fn upsert(&self, user_id: Uuid, token: &str, platform: &str) -> Result<PushToken> {
        upsert(&self.pool, user_id, token, platform).await
    }

    async fn delete_by_token(&self, user_id: Uuid, token: &str) -> Result<bool> {
        delete_by_token(&self.pool, user_id, token).await
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<PushToken>> {
        find_by_user(&self.pool, user_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn upsert(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    token: &str,
    platform: &str,
) -> Result<PushToken> {
    let sql = format!(
        "INSERT INTO push_tokens (user_id, token, platform) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, token) DO UPDATE SET platform = EXCLUDED.platform \
         RETURNING {PT_COLS}"
    );
    let pt = sqlx::query_as::<_, PushToken>(&sql)
        .bind(user_id)
        .bind(token)
        .bind(platform)
        .fetch_one(exec)
        .await?;

    Ok(pt)
}

pub(crate) async fn delete_by_token(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    token: &str,
) -> Result<bool> {
    let result = sqlx::query("DELETE FROM push_tokens WHERE user_id = $1 AND token = $2")
        .bind(user_id)
        .bind(token)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn find_by_user(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<PushToken>> {
    let sql = format!("SELECT {PT_COLS} FROM push_tokens WHERE user_id = $1");
    let tokens = sqlx::query_as::<_, PushToken>(&sql)
        .bind(user_id)
        .fetch_all(exec)
        .await?;

    Ok(tokens)
}
