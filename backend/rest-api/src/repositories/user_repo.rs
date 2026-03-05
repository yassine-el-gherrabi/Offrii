use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};

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
        RETURNING id, email, password_hash, display_name,
                  reminder_freq, reminder_time, created_at, updated_at
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
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, display_name, \
         reminder_freq, reminder_time, created_at, updated_at \
         FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(exec)
    .await?;

    Ok(user)
}
