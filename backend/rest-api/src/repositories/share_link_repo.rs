use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::ShareLink;
use crate::traits;

const SHARE_LINK_COLS: &str = "id, user_id, token, created_at, expires_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgShareLinkRepo {
    pool: PgPool,
}

impl PgShareLinkRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::ShareLinkRepo for PgShareLinkRepo {
    async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ShareLink> {
        create(&self.pool, user_id, token, expires_at).await
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<ShareLink>> {
        list_by_user(&self.pool, user_id).await
    }

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<ShareLink>> {
        find_by_id(&self.pool, id, user_id).await
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<ShareLink>> {
        find_by_token(&self.pool, token).await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        delete(&self.pool, id, user_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    token: &str,
    expires_at: Option<DateTime<Utc>>,
) -> Result<ShareLink> {
    let sql = format!(
        "INSERT INTO share_links (user_id, token, expires_at) \
         VALUES ($1, $2, $3) \
         RETURNING {SHARE_LINK_COLS}"
    );
    let link = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(exec)
        .await?;

    Ok(link)
}

pub(crate) async fn list_by_user(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<ShareLink>> {
    let sql = format!(
        "SELECT {SHARE_LINK_COLS} FROM share_links \
         WHERE user_id = $1 \
         ORDER BY created_at DESC"
    );
    let links = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(user_id)
        .fetch_all(exec)
        .await?;

    Ok(links)
}

pub(crate) async fn find_by_id(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<ShareLink>> {
    let sql = format!("SELECT {SHARE_LINK_COLS} FROM share_links WHERE id = $1 AND user_id = $2");
    let link = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(exec)
        .await?;

    Ok(link)
}

pub(crate) async fn find_by_token(
    exec: impl PgExecutor<'_>,
    token: &str,
) -> Result<Option<ShareLink>> {
    let sql = format!("SELECT {SHARE_LINK_COLS} FROM share_links WHERE token = $1");
    let link = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(token)
        .fetch_optional(exec)
        .await?;

    Ok(link)
}

pub(crate) async fn delete(exec: impl PgExecutor<'_>, id: Uuid, user_id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM share_links WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}
