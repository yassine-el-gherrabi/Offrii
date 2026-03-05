use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::User;
use crate::traits;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::UserRepo for PgUserRepo {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<User> {
        create_user(&self.pool, email, password_hash, display_name).await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        find_by_email(&self.pool, email).await
    }

    async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        find_by_id(&self.pool, user_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn create_user(
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

pub(crate) async fn find_by_email(exec: impl PgExecutor<'_>, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}

pub(crate) async fn find_by_id(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}
