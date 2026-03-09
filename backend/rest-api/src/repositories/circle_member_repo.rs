use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, Row};
use uuid::Uuid;

use crate::models::CircleMember;
use crate::traits;

const MEMBER_COLS: &str = "circle_id, user_id, role, joined_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCircleMemberRepo {
    pool: PgPool,
}

impl PgCircleMemberRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::CircleMemberRepo for PgCircleMemberRepo {
    async fn add_member(&self, circle_id: Uuid, user_id: Uuid, role: &str) -> Result<CircleMember> {
        add_member(&self.pool, circle_id, user_id, role).await
    }

    async fn remove_member(&self, circle_id: Uuid, user_id: Uuid) -> Result<bool> {
        remove_member(&self.pool, circle_id, user_id).await
    }

    async fn find_member(&self, circle_id: Uuid, user_id: Uuid) -> Result<Option<CircleMember>> {
        find_member(&self.pool, circle_id, user_id).await
    }

    async fn list_members(&self, circle_id: Uuid) -> Result<Vec<CircleMember>> {
        list_members(&self.pool, circle_id).await
    }

    async fn count_members(&self, circle_id: Uuid) -> Result<i64> {
        count_members(&self.pool, circle_id).await
    }

    async fn find_direct_circle_between(&self, user_a: Uuid, user_b: Uuid) -> Result<Option<Uuid>> {
        find_direct_circle_between(&self.pool, user_a, user_b).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn add_member(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<CircleMember> {
    let sql = format!(
        "INSERT INTO circle_members (circle_id, user_id, role) \
         VALUES ($1, $2, $3) \
         RETURNING {MEMBER_COLS}"
    );
    let member = sqlx::query_as::<_, CircleMember>(&sql)
        .bind(circle_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(exec)
        .await?;

    Ok(member)
}

pub(crate) async fn remove_member(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let result = sqlx::query("DELETE FROM circle_members WHERE circle_id = $1 AND user_id = $2")
        .bind(circle_id)
        .bind(user_id)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn find_member(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    user_id: Uuid,
) -> Result<Option<CircleMember>> {
    let sql =
        format!("SELECT {MEMBER_COLS} FROM circle_members WHERE circle_id = $1 AND user_id = $2");
    let member = sqlx::query_as::<_, CircleMember>(&sql)
        .bind(circle_id)
        .bind(user_id)
        .fetch_optional(exec)
        .await?;

    Ok(member)
}

pub(crate) async fn list_members(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
) -> Result<Vec<CircleMember>> {
    let sql =
        format!("SELECT {MEMBER_COLS} FROM circle_members WHERE circle_id = $1 ORDER BY joined_at");
    let members = sqlx::query_as::<_, CircleMember>(&sql)
        .bind(circle_id)
        .fetch_all(exec)
        .await?;

    Ok(members)
}

pub(crate) async fn count_members(exec: impl PgExecutor<'_>, circle_id: Uuid) -> Result<i64> {
    let row = sqlx::query("SELECT COUNT(*) FROM circle_members WHERE circle_id = $1")
        .bind(circle_id)
        .fetch_one(exec)
        .await?;
    let count: i64 = row.get(0);

    Ok(count)
}

/// Find an existing direct circle where both users are members.
pub(crate) async fn find_direct_circle_between(
    exec: impl PgExecutor<'_>,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<Option<Uuid>> {
    let row = sqlx::query(
        "SELECT c.id FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true \
         LIMIT 1",
    )
    .bind(user_a)
    .bind(user_b)
    .fetch_optional(exec)
    .await?;

    Ok(row.map(|r| r.get("id")))
}
