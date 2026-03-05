use anyhow::Result;
use sqlx::PgExecutor;
use uuid::Uuid;

/// Copy the default categories (user_id IS NULL) to the given user.
pub async fn copy_defaults_for_user(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<u64> {
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
