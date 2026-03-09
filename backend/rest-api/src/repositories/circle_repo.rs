use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, Row};
use uuid::Uuid;

use crate::models::Circle;
use crate::traits;

const CIRCLE_COLS: &str = "id, name, owner_id, is_direct, created_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCircleRepo {
    pool: PgPool,
}

impl PgCircleRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::CircleRepo for PgCircleRepo {
    async fn create(&self, name: Option<&str>, owner_id: Uuid, is_direct: bool) -> Result<Circle> {
        create(&self.pool, name, owner_id, is_direct).await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Circle>> {
        find_by_id(&self.pool, id).await
    }

    async fn update_name(&self, id: Uuid, name: &str) -> Result<Option<Circle>> {
        update_name(&self.pool, id, name).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        delete(&self.pool, id).await
    }

    async fn list_by_member(&self, user_id: Uuid) -> Result<Vec<(Circle, i64, Option<String>)>> {
        list_by_member(&self.pool, user_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    name: Option<&str>,
    owner_id: Uuid,
    is_direct: bool,
) -> Result<Circle> {
    let sql = format!(
        "INSERT INTO circles (name, owner_id, is_direct) \
         VALUES ($1, $2, $3) \
         RETURNING {CIRCLE_COLS}"
    );
    let circle = sqlx::query_as::<_, Circle>(&sql)
        .bind(name)
        .bind(owner_id)
        .bind(is_direct)
        .fetch_one(exec)
        .await?;

    Ok(circle)
}

pub(crate) async fn find_by_id(exec: impl PgExecutor<'_>, id: Uuid) -> Result<Option<Circle>> {
    let sql = format!("SELECT {CIRCLE_COLS} FROM circles WHERE id = $1");
    let circle = sqlx::query_as::<_, Circle>(&sql)
        .bind(id)
        .fetch_optional(exec)
        .await?;

    Ok(circle)
}

pub(crate) async fn update_name(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    name: &str,
) -> Result<Option<Circle>> {
    let sql = format!("UPDATE circles SET name = $2 WHERE id = $1 RETURNING {CIRCLE_COLS}");
    let circle = sqlx::query_as::<_, Circle>(&sql)
        .bind(id)
        .bind(name)
        .fetch_optional(exec)
        .await?;

    Ok(circle)
}

pub(crate) async fn delete(exec: impl PgExecutor<'_>, id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM circles WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Returns circles the user is a member of, with member counts and
/// (for direct circles) the other member's username.
pub(crate) async fn list_by_member(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<(Circle, i64, Option<String>)>> {
    let rows = sqlx::query(
        "SELECT c.id, c.name, c.owner_id, c.is_direct, c.created_at, \
                COUNT(cm2.user_id) AS member_count, \
                ( \
                    SELECT u.username FROM circle_members cm3 \
                    JOIN users u ON u.id = cm3.user_id \
                    WHERE cm3.circle_id = c.id AND cm3.user_id != $1 \
                    LIMIT 1 \
                ) AS other_username \
         FROM circles c \
         JOIN circle_members cm ON cm.circle_id = c.id AND cm.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         GROUP BY c.id \
         ORDER BY c.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(exec)
    .await?;

    let results = rows
        .into_iter()
        .map(|row| {
            let circle = Circle {
                id: row.get("id"),
                name: row.get("name"),
                owner_id: row.get("owner_id"),
                is_direct: row.get("is_direct"),
                created_at: row.get("created_at"),
            };
            let count: i64 = row.get("member_count");
            let other_username: Option<String> = row.get("other_username");
            (circle, count, other_username)
        })
        .collect();

    Ok(results)
}
