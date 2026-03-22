use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::Category;
use crate::traits;

/// Shared column list for all category queries.
const CAT_COLS: &str = "id, name, icon, is_default, position, created_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCategoryRepo {
    pool: PgPool,
}

impl PgCategoryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::CategoryRepo for PgCategoryRepo {
    async fn list_all(&self) -> Result<Vec<Category>> {
        let sql =
            format!("SELECT {CAT_COLS} FROM categories ORDER BY position ASC, created_at ASC");
        let cats = sqlx::query_as::<_, Category>(&sql)
            .fetch_all(&self.pool)
            .await?;

        Ok(cats)
    }
}
