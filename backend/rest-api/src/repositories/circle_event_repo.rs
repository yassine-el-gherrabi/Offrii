use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, Row};
use uuid::Uuid;

use crate::models::CircleEvent;
use crate::traits;

const EVENT_COLS: &str =
    "id, circle_id, actor_id, event_type, target_item_id, target_user_id, created_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCircleEventRepo {
    pool: PgPool,
}

impl PgCircleEventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::CircleEventRepo for PgCircleEventRepo {
    async fn insert(
        &self,
        circle_id: Uuid,
        actor_id: Uuid,
        event_type: &str,
        target_item_id: Option<Uuid>,
        target_user_id: Option<Uuid>,
    ) -> Result<CircleEvent> {
        insert(
            &self.pool,
            circle_id,
            actor_id,
            event_type,
            target_item_id,
            target_user_id,
        )
        .await
    }

    async fn list_by_circle(
        &self,
        circle_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CircleEvent>> {
        list_by_circle(&self.pool, circle_id, limit, offset).await
    }

    async fn count_by_circle(&self, circle_id: Uuid) -> Result<i64> {
        count_by_circle(&self.pool, circle_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn insert(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    actor_id: Uuid,
    event_type: &str,
    target_item_id: Option<Uuid>,
    target_user_id: Option<Uuid>,
) -> Result<CircleEvent> {
    let sql = format!(
        "INSERT INTO circle_events (circle_id, actor_id, event_type, target_item_id, target_user_id) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING {EVENT_COLS}"
    );
    let event = sqlx::query_as::<_, CircleEvent>(&sql)
        .bind(circle_id)
        .bind(actor_id)
        .bind(event_type)
        .bind(target_item_id)
        .bind(target_user_id)
        .fetch_one(exec)
        .await?;

    Ok(event)
}

pub(crate) async fn list_by_circle(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<CircleEvent>> {
    let sql = format!(
        "SELECT {EVENT_COLS} FROM circle_events \
         WHERE circle_id = $1 \
         ORDER BY created_at DESC \
         LIMIT $2 OFFSET $3"
    );
    let events = sqlx::query_as::<_, CircleEvent>(&sql)
        .bind(circle_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(exec)
        .await?;

    Ok(events)
}

pub(crate) async fn count_by_circle(exec: impl PgExecutor<'_>, circle_id: Uuid) -> Result<i64> {
    let row = sqlx::query("SELECT COUNT(*) FROM circle_events WHERE circle_id = $1")
        .bind(circle_id)
        .fetch_one(exec)
        .await?;
    let count: i64 = row.get(0);

    Ok(count)
}
