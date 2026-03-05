use anyhow::Result;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::User;

pub async fn create_user(
    exec: impl PgExecutor<'_>,
    email: &str,
    password_hash: &str,
    display_name: Option<&str>,
) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(exec)
    .await?;

    Ok(user)
}

pub async fn find_by_email(exec: impl PgExecutor<'_>, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}

pub async fn find_by_id(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}
