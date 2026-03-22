use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::CircleInvite;
use crate::traits;

const INVITE_COLS: &str =
    "id, circle_id, token, created_by, expires_at, max_uses, use_count, created_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCircleInviteRepo {
    pool: PgPool,
}

impl PgCircleInviteRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::CircleInviteRepo for PgCircleInviteRepo {
    async fn create(
        &self,
        circle_id: Uuid,
        token: &str,
        created_by: Uuid,
        expires_at: DateTime<Utc>,
        max_uses: i32,
    ) -> Result<CircleInvite> {
        create(
            &self.pool, circle_id, token, created_by, expires_at, max_uses,
        )
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CircleInvite>> {
        find_by_id(&self.pool, id).await
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<CircleInvite>> {
        find_by_token(&self.pool, token).await
    }

    async fn increment_use_count(&self, id: Uuid) -> Result<bool> {
        increment_use_count(&self.pool, id).await
    }

    async fn list_active_by_circle(&self, circle_id: Uuid) -> Result<Vec<CircleInvite>> {
        list_active_by_circle(&self.pool, circle_id).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        delete(&self.pool, id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    token: &str,
    created_by: Uuid,
    expires_at: DateTime<Utc>,
    max_uses: i32,
) -> Result<CircleInvite> {
    let sql = format!(
        "INSERT INTO circle_invites (circle_id, token, created_by, expires_at, max_uses) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING {INVITE_COLS}"
    );
    let invite = sqlx::query_as::<_, CircleInvite>(&sql)
        .bind(circle_id)
        .bind(token)
        .bind(created_by)
        .bind(expires_at)
        .bind(max_uses)
        .fetch_one(exec)
        .await?;

    Ok(invite)
}

pub(crate) async fn find_by_id(
    exec: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<CircleInvite>> {
    let sql = format!("SELECT {INVITE_COLS} FROM circle_invites WHERE id = $1");
    let invite = sqlx::query_as::<_, CircleInvite>(&sql)
        .bind(id)
        .fetch_optional(exec)
        .await?;
    Ok(invite)
}

pub(crate) async fn find_by_token(
    exec: impl PgExecutor<'_>,
    token: &str,
) -> Result<Option<CircleInvite>> {
    let sql = format!("SELECT {INVITE_COLS} FROM circle_invites WHERE token = $1");
    let invite = sqlx::query_as::<_, CircleInvite>(&sql)
        .bind(token)
        .fetch_optional(exec)
        .await?;

    Ok(invite)
}

/// Atomically increment use_count only if the invite is still valid.
pub(crate) async fn increment_use_count(exec: impl PgExecutor<'_>, id: Uuid) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE circle_invites \
         SET use_count = use_count + 1 \
         WHERE id = $1 AND use_count < max_uses AND expires_at > NOW()",
    )
    .bind(id)
    .execute(exec)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Return invites that haven't expired and haven't been maxed out.
pub(crate) async fn list_active_by_circle(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
) -> Result<Vec<CircleInvite>> {
    let sql = format!(
        "SELECT {INVITE_COLS} FROM circle_invites \
         WHERE circle_id = $1 AND expires_at > NOW() AND use_count < max_uses \
         ORDER BY created_at DESC"
    );
    let invites = sqlx::query_as::<_, CircleInvite>(&sql)
        .bind(circle_id)
        .fetch_all(exec)
        .await?;

    Ok(invites)
}

pub(crate) async fn delete(exec: impl PgExecutor<'_>, id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM circle_invites WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}
