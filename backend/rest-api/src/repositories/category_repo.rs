use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, QueryBuilder, Row};
use uuid::Uuid;

use crate::models::Category;
use crate::traits;

/// Shared column list for all category queries.
const CAT_COLS: &str = "id, user_id, name, icon, is_default, position, created_at";

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

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Category>> {
        list_by_user(&self.pool, user_id).await
    }

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Category>> {
        find_by_id(&self.pool, id, user_id).await
    }

    async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        icon: Option<&str>,
        position: i32,
    ) -> Result<Category> {
        create(&self.pool, user_id, name, icon, position).await
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        icon: Option<&str>,
        position: Option<i32>,
    ) -> Result<Option<Category>> {
        update(&self.pool, id, user_id, name, icon, position).await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        delete(&self.pool, id, user_id).await
    }

    async fn next_position(&self, user_id: Uuid) -> Result<i32> {
        next_position(&self.pool, user_id).await
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

pub(crate) async fn list_by_user(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<Category>> {
    let sql = format!(
        "SELECT {CAT_COLS} FROM categories WHERE user_id = $1 ORDER BY position ASC, created_at ASC"
    );
    let cats = sqlx::query_as::<_, Category>(&sql)
        .bind(user_id)
        .fetch_all(exec)
        .await?;

    Ok(cats)
}

pub(crate) async fn find_by_id(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<Category>> {
    let sql = format!("SELECT {CAT_COLS} FROM categories WHERE id = $1 AND user_id = $2");
    let cat = sqlx::query_as::<_, Category>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(exec)
        .await?;

    Ok(cat)
}

pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    name: &str,
    icon: Option<&str>,
    position: i32,
) -> Result<Category> {
    let sql = format!(
        "INSERT INTO categories (user_id, name, icon, position) \
         VALUES ($1, $2, $3, $4) \
         RETURNING {CAT_COLS}"
    );
    let cat = sqlx::query_as::<_, Category>(&sql)
        .bind(user_id)
        .bind(name)
        .bind(icon)
        .bind(position)
        .fetch_one(exec)
        .await?;

    Ok(cat)
}

pub(crate) async fn update(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    name: Option<&str>,
    icon: Option<&str>,
    position: Option<i32>,
) -> Result<Option<Category>> {
    let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("UPDATE categories SET ");
    let mut separated = qb.separated(", ");

    if let Some(n) = name {
        separated.push("name = ");
        separated.push_bind_unseparated(n);
    }
    if let Some(i) = icon {
        separated.push("icon = ");
        separated.push_bind_unseparated(i);
    }
    if let Some(p) = position {
        separated.push("position = ");
        separated.push_bind_unseparated(p);
    }

    qb.push(" WHERE id = ");
    qb.push_bind(id);
    qb.push(" AND user_id = ");
    qb.push_bind(user_id);
    qb.push(format!(" RETURNING {CAT_COLS}"));

    let cat = qb.build_query_as::<Category>().fetch_optional(exec).await?;

    Ok(cat)
}

pub(crate) async fn delete(exec: impl PgExecutor<'_>, id: Uuid, user_id: Uuid) -> Result<bool> {
    let result =
        sqlx::query("DELETE FROM categories WHERE id = $1 AND user_id = $2 AND is_default = FALSE")
            .bind(id)
            .bind(user_id)
            .execute(exec)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn next_position(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<i32> {
    let row =
        sqlx::query("SELECT COALESCE(MAX(position), 0) + 1 FROM categories WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(exec)
            .await?;

    Ok(row.get(0))
}
