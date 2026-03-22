use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, Row};
use uuid::Uuid;

use crate::models::CircleItem;
use crate::traits;

const CIRCLE_ITEM_COLS: &str = "circle_id, item_id, shared_by, shared_at";

/// (circle_id, name, is_direct, image_url)
pub type CircleInfoTuple = (Uuid, String, bool, Option<String>);
pub type CircleInfoMap = std::collections::HashMap<Uuid, Vec<CircleInfoTuple>>;

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
    ) -> Result<Option<CircleItem>> {
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

    async fn list_circle_names_for_items(&self, item_ids: &[Uuid]) -> Result<CircleInfoMap> {
        list_circle_names_for_items(&self.pool, item_ids).await
    }

    async fn delete_by_circle_and_user(&self, circle_id: Uuid, user_id: Uuid) -> Result<u64> {
        delete_by_circle_and_user(&self.pool, circle_id, user_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn share_item(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    item_id: Uuid,
    shared_by: Uuid,
) -> Result<Option<CircleItem>> {
    let sql = format!(
        "INSERT INTO circle_items (circle_id, item_id, shared_by) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (circle_id, item_id) DO NOTHING \
         RETURNING {CIRCLE_ITEM_COLS}"
    );
    let item = sqlx::query_as::<_, CircleItem>(&sql)
        .bind(circle_id)
        .bind(item_id)
        .bind(shared_by)
        .fetch_optional(exec)
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

/// Batch fetch circle names for multiple items in a single query.
/// Returns a map: item_id → Vec<(circle_id, circle_name)>
pub(crate) async fn list_circle_names_for_items(
    exec: impl PgExecutor<'_>,
    item_ids: &[Uuid],
) -> Result<CircleInfoMap> {
    // For direct circles (name IS NULL), use the other member's display_name or username
    let rows = sqlx::query(
        "SELECT ci.item_id, c.id, c.is_direct, c.image_url, \
           CASE WHEN c.name IS NOT NULL THEN c.name \
                ELSE COALESCE(u.display_name, u.username, '') \
           END AS name \
         FROM circle_items ci \
         JOIN circles c ON c.id = ci.circle_id \
         LEFT JOIN circle_members cm ON cm.circle_id = c.id AND cm.user_id != ci.shared_by AND c.is_direct = true \
         LEFT JOIN users u ON u.id = cm.user_id \
         WHERE ci.item_id = ANY($1) \
         ORDER BY ci.shared_at ASC",
    )
    .bind(item_ids)
    .fetch_all(exec)
    .await?;

    let mut map: CircleInfoMap = std::collections::HashMap::new();
    for row in rows {
        let item_id: Uuid = row.get("item_id");
        let circle_id: Uuid = row.get("id");
        let name: String = row.get("name");
        let is_direct: bool = row.get("is_direct");
        let image_url: Option<String> = row.get("image_url");
        map.entry(item_id)
            .or_default()
            .push((circle_id, name, is_direct, image_url));
    }

    Ok(map)
}

pub(crate) async fn delete_by_circle_and_user(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    user_id: Uuid,
) -> Result<u64> {
    let result = sqlx::query("DELETE FROM circle_items WHERE circle_id = $1 AND shared_by = $2")
        .bind(circle_id)
        .bind(user_id)
        .execute(exec)
        .await?;
    Ok(result.rows_affected())
}
