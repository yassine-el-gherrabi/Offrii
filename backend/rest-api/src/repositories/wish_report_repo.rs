use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::community_wish::WishReport;
use crate::traits;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgWishReportRepo {
    pool: PgPool,
}

impl PgWishReportRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::WishReportRepo for PgWishReportRepo {
    async fn create(
        &self,
        wish_id: Uuid,
        reporter_id: Uuid,
        reason: &str,
        details: Option<&str>,
    ) -> Result<WishReport> {
        create(&self.pool, wish_id, reporter_id, reason, details).await
    }

    async fn has_reported(&self, wish_id: Uuid, reporter_id: Uuid) -> Result<bool> {
        has_reported(&self.pool, wish_id, reporter_id).await
    }

    async fn count_by_reporter_today(&self, reporter_id: Uuid) -> Result<i64> {
        count_by_reporter_today(&self.pool, reporter_id).await
    }

    async fn delete_by_wish(&self, wish_id: Uuid) -> Result<u64> {
        delete_by_wish(&self.pool, wish_id).await
    }
}

// ── Free functions ───────────────────────────────────────────────────

pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    wish_id: Uuid,
    reporter_id: Uuid,
    reason: &str,
    details: Option<&str>,
) -> Result<WishReport> {
    let report = sqlx::query_as::<_, WishReport>(
        "INSERT INTO wish_reports (wish_id, reporter_id, reason, details) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, wish_id, reporter_id, reason, details, created_at",
    )
    .bind(wish_id)
    .bind(reporter_id)
    .bind(reason)
    .bind(details)
    .fetch_one(exec)
    .await?;
    Ok(report)
}

pub(crate) async fn has_reported(
    exec: impl PgExecutor<'_>,
    wish_id: Uuid,
    reporter_id: Uuid,
) -> Result<bool> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM wish_reports WHERE wish_id = $1 AND reporter_id = $2)",
    )
    .bind(wish_id)
    .bind(reporter_id)
    .fetch_one(exec)
    .await?;
    Ok(row.0)
}

pub(crate) async fn count_by_reporter_today(
    exec: impl PgExecutor<'_>,
    reporter_id: Uuid,
) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wish_reports \
         WHERE reporter_id = $1 AND created_at >= CURRENT_DATE",
    )
    .bind(reporter_id)
    .fetch_one(exec)
    .await?;
    Ok(row.0)
}

pub(crate) async fn delete_by_wish(exec: impl PgExecutor<'_>, wish_id: Uuid) -> Result<u64> {
    let rows = sqlx::query("DELETE FROM wish_reports WHERE wish_id = $1")
        .bind(wish_id)
        .execute(exec)
        .await?
        .rows_affected();
    Ok(rows)
}
