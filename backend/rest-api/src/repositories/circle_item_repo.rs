use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::CircleItem;
use crate::traits;

const CIRCLE_ITEM_COLS: &str = "circle_id, item_id, shared_by, shared_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCircleItemRepo {
    pool: PgPool,
}

impl PgCircleItemRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::CircleItemRepo for PgCircleItemRepo {
    async fn share_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        shared_by: Uuid,
    ) -> Result<CircleItem> {
        share_item(&self.pool, circle_id, item_id, shared_by).await
    }

    async fn unshare_item(&self, circle_id: Uuid, item_id: Uuid) -> Result<bool> {
        unshare_item(&self.pool, circle_id, item_id).await
    }

    async fn list_by_circle(&self, circle_id: Uuid) -> Result<Vec<CircleItem>> {
        list_by_circle(&self.pool, circle_id).await
    }

    async fn find(&self, circle_id: Uuid, item_id: Uuid) -> Result<Option<CircleItem>> {
        find(&self.pool, circle_id, item_id).await
    }

    async fn list_circles_for_item(&self, item_id: Uuid) -> Result<Vec<Uuid>> {
        list_circles_for_item(&self.pool, item_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn share_item(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    item_id: Uuid,
    shared_by: Uuid,
) -> Result<CircleItem> {
    let sql = format!(
        "INSERT INTO circle_items (circle_id, item_id, shared_by) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (circle_id, item_id) DO UPDATE SET shared_by = EXCLUDED.shared_by \
         RETURNING {CIRCLE_ITEM_COLS}"
    );
    let item = sqlx::query_as::<_, CircleItem>(&sql)
        .bind(circle_id)
        .bind(item_id)
        .bind(shared_by)
        .fetch_one(exec)
        .await?;

    Ok(item)
}

pub(crate) async fn unshare_item(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    item_id: Uuid,
) -> Result<bool> {
    let result = sqlx::query("DELETE FROM circle_items WHERE circle_id = $1 AND item_id = $2")
        .bind(circle_id)
        .bind(item_id)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn list_by_circle(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
) -> Result<Vec<CircleItem>> {
    let sql = format!(
        "SELECT {CIRCLE_ITEM_COLS} FROM circle_items \
         WHERE circle_id = $1 \
         ORDER BY shared_at DESC"
    );
    let items = sqlx::query_as::<_, CircleItem>(&sql)
        .bind(circle_id)
        .fetch_all(exec)
        .await?;

    Ok(items)
}

pub(crate) async fn find(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    item_id: Uuid,
) -> Result<Option<CircleItem>> {
    let sql = format!(
        "SELECT {CIRCLE_ITEM_COLS} FROM circle_items WHERE circle_id = $1 AND item_id = $2"
    );
    let item = sqlx::query_as::<_, CircleItem>(&sql)
        .bind(circle_id)
        .bind(item_id)
        .fetch_optional(exec)
        .await?;

    Ok(item)
}

pub(crate) async fn list_circles_for_item(
    exec: impl PgExecutor<'_>,
    item_id: Uuid,
) -> Result<Vec<Uuid>> {
    let ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT circle_id FROM circle_items WHERE item_id = $1",
    )
    .bind(item_id)
    .fetch_all(exec)
    .await?;

    Ok(ids)
}
