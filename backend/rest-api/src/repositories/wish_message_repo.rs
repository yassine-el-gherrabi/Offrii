use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::community_wish::WishMessage;
use crate::traits;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgWishMessageRepo {
    pool: PgPool,
}

impl PgWishMessageRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::WishMessageRepo for PgWishMessageRepo {
    async fn create(&self, wish_id: Uuid, sender_id: Uuid, body: &str) -> Result<WishMessage> {
        create(&self.pool, wish_id, sender_id, body).await
    }

    async fn list_by_wish(
        &self,
        wish_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WishMessage>> {
        list_by_wish(&self.pool, wish_id, limit, offset).await
    }

    async fn count_by_wish(&self, wish_id: Uuid) -> Result<i64> {
        count_by_wish(&self.pool, wish_id).await
    }
}

// ── Free functions ───────────────────────────────────────────────────

pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    wish_id: Uuid,
    sender_id: Uuid,
    body: &str,
) -> Result<WishMessage> {
    let msg = sqlx::query_as::<_, WishMessage>(
        "INSERT INTO wish_messages (wish_id, sender_id, body) \
         VALUES ($1, $2, $3) \
         RETURNING id, wish_id, sender_id, body, created_at",
    )
    .bind(wish_id)
    .bind(sender_id)
    .bind(body)
    .fetch_one(exec)
    .await?;
    Ok(msg)
}

pub(crate) async fn list_by_wish(
    exec: impl PgExecutor<'_>,
    wish_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<WishMessage>> {
    let msgs = sqlx::query_as::<_, WishMessage>(
        "SELECT id, wish_id, sender_id, body, created_at \
         FROM wish_messages \
         WHERE wish_id = $1 \
         ORDER BY created_at ASC \
         LIMIT $2 OFFSET $3",
    )
    .bind(wish_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(exec)
    .await?;
    Ok(msgs)
}

pub(crate) async fn count_by_wish(exec: impl PgExecutor<'_>, wish_id: Uuid) -> Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wish_messages WHERE wish_id = $1")
        .bind(wish_id)
        .fetch_one(exec)
        .await?;
    Ok(row.0)
}
