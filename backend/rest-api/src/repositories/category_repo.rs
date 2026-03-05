use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::traits;

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
    async fn copy_defaults_for_user(&self, user_id: Uuid) -> Result<u64> {
        copy_defaults_for_user(&self.pool, user_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

/// Copy the default categories (user_id IS NULL) to the given user.
pub(crate) async fn copy_defaults_for_user(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<u64> {
    let result = sqlx::query(
        r#"
        INSERT INTO categories (user_id, name, icon, is_default, position)
        SELECT $1, name, icon, is_default, position
        FROM categories
        WHERE user_id IS NULL
        "#,
    )
    .bind(user_id)
    .execute(exec)
    .await?;

    Ok(result.rows_affected())
}
