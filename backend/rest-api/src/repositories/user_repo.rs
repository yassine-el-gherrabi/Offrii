use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use sqlx::{PgExecutor, PgPool, QueryBuilder};
use uuid::Uuid;

use crate::models::User;
use crate::traits;

/// Shared column list for all user queries (avoids duplication).
const USER_COLS: &str = "id, email, username, password_hash, display_name, \
                         oauth_provider, oauth_provider_id, \
                         reminder_freq, reminder_time, timezone, \
                         utc_reminder_hour, locale, token_version, \
                         created_at, updated_at";

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
        username: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<User> {
        create_user(&self.pool, email, username, password_hash, display_name).await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        find_by_email(&self.pool, email).await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        find_by_id(&self.pool, id).await
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>> {
        find_by_ids(&self.pool, ids).await
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        find_by_username(&self.pool, username).await
    }

    async fn is_username_taken(
        &self,
        username: &str,
        exclude_user_id: Option<Uuid>,
    ) -> Result<bool> {
        is_username_taken(&self.pool, username, exclude_user_id).await
    }

    async fn update_profile(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        username: Option<&str>,
        reminder_freq: Option<&str>,
        reminder_time: Option<NaiveTime>,
        timezone: Option<&str>,
        utc_reminder_hour: Option<i16>,
        locale: Option<&str>,
    ) -> Result<Option<User>> {
        update_profile(
            &self.pool,
            id,
            display_name,
            username,
            reminder_freq,
            reminder_time,
            timezone,
            utc_reminder_hour,
            locale,
        )
        .await
    }

    async fn delete_user(&self, id: Uuid) -> Result<bool> {
        delete_user(&self.pool, id).await
    }

    async fn find_eligible_for_reminder(&self, utc_hour: i16) -> Result<Vec<User>> {
        find_eligible_for_reminder(&self.pool, utc_hour).await
    }

    async fn update_password_hash(&self, id: Uuid, password_hash: &str) -> Result<bool> {
        update_password_hash(&self.pool, id, password_hash).await
    }

    async fn increment_token_version(&self, id: Uuid) -> Result<i32> {
        increment_token_version(&self.pool, id).await
    }

    async fn get_user_created_at(&self, user_id: Uuid) -> Result<Option<DateTime<Utc>>> {
        get_user_created_at(&self.pool, user_id).await
    }

    async fn create_oauth_user(
        &self,
        email: &str,
        username: &str,
        display_name: Option<&str>,
        oauth_provider: &str,
        oauth_provider_id: &str,
    ) -> Result<User> {
        create_oauth_user(
            &self.pool,
            email,
            username,
            display_name,
            oauth_provider,
            oauth_provider_id,
        )
        .await
    }

    async fn find_by_oauth(&self, provider: &str, provider_id: &str) -> Result<Option<User>> {
        find_by_oauth(&self.pool, provider, provider_id).await
    }

    async fn link_oauth(&self, user_id: Uuid, provider: &str, provider_id: &str) -> Result<bool> {
        link_oauth(&self.pool, user_id, provider, provider_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn create_user(
    exec: impl PgExecutor<'_>,
    email: &str,
    username: &str,
    password_hash: &str,
    display_name: Option<&str>,
) -> Result<User> {
    let sql = format!(
        "INSERT INTO users (email, username, password_hash, display_name) \
         VALUES ($1, $2, $3, $4) \
         RETURNING {USER_COLS}"
    );
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(email)
        .bind(username)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(exec)
        .await?;

    Ok(user)
}

pub(crate) async fn find_by_username(
    exec: impl PgExecutor<'_>,
    username: &str,
) -> Result<Option<User>> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE username = $1");
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(username)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}

pub(crate) async fn is_username_taken(
    exec: impl PgExecutor<'_>,
    username: &str,
    exclude_user_id: Option<Uuid>,
) -> Result<bool> {
    let exists: bool = match exclude_user_id {
        Some(uid) => {
            sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND id != $2)",
            )
            .bind(username)
            .bind(uid)
            .fetch_one(exec)
            .await?
        }
        None => {
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                .bind(username)
                .fetch_one(exec)
                .await?
        }
    };

    Ok(exists)
}

pub(crate) async fn find_by_email(exec: impl PgExecutor<'_>, email: &str) -> Result<Option<User>> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE email = $1");
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(email)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}

pub(crate) async fn find_by_id(exec: impl PgExecutor<'_>, id: Uuid) -> Result<Option<User>> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(id)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}

pub(crate) async fn find_by_ids(exec: impl PgExecutor<'_>, ids: &[Uuid]) -> Result<Vec<User>> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE id = ANY($1)");
    let users = sqlx::query_as::<_, User>(&sql)
        .bind(ids)
        .fetch_all(exec)
        .await?;

    Ok(users)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn update_profile(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    display_name: Option<&str>,
    username: Option<&str>,
    reminder_freq: Option<&str>,
    reminder_time: Option<NaiveTime>,
    timezone: Option<&str>,
    utc_reminder_hour: Option<i16>,
    locale: Option<&str>,
) -> Result<Option<User>> {
    // If nothing to update, short-circuit with a SELECT instead of invalid SQL
    if display_name.is_none()
        && username.is_none()
        && reminder_freq.is_none()
        && reminder_time.is_none()
        && timezone.is_none()
        && utc_reminder_hour.is_none()
        && locale.is_none()
    {
        return find_by_id(exec, id).await;
    }

    let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("UPDATE users SET ");
    let mut separated = qb.separated(", ");

    if let Some(v) = display_name {
        separated.push("display_name = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = username {
        separated.push("username = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = reminder_freq {
        separated.push("reminder_freq = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = reminder_time {
        separated.push("reminder_time = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = timezone {
        separated.push("timezone = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = utc_reminder_hour {
        separated.push("utc_reminder_hour = ");
        separated.push_bind_unseparated(v);
    }
    if let Some(v) = locale {
        separated.push("locale = ");
        separated.push_bind_unseparated(v);
    }

    qb.push(" WHERE id = ");
    qb.push_bind(id);
    qb.push(format!(" RETURNING {USER_COLS}"));

    let user = qb.build_query_as::<User>().fetch_optional(exec).await?;

    Ok(user)
}

pub(crate) async fn update_password_hash(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    password_hash: &str,
) -> Result<bool> {
    let result =
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(exec)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn delete_user(exec: impl PgExecutor<'_>, id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn find_eligible_for_reminder(
    exec: impl PgExecutor<'_>,
    utc_hour: i16,
) -> Result<Vec<User>> {
    let sql = format!(
        "SELECT {USER_COLS} FROM users u \
         WHERE u.utc_reminder_hour = $1 \
           AND u.reminder_freq != 'never' \
           AND EXISTS (SELECT 1 FROM push_tokens pt WHERE pt.user_id = u.id)"
    );
    let users = sqlx::query_as::<_, User>(&sql)
        .bind(utc_hour)
        .fetch_all(exec)
        .await?;

    Ok(users)
}

pub(crate) async fn increment_token_version(exec: impl PgExecutor<'_>, id: Uuid) -> Result<i32> {
    let row: (i32,) = sqlx::query_as(
        "UPDATE users SET token_version = token_version + 1 \
         WHERE id = $1 RETURNING token_version",
    )
    .bind(id)
    .fetch_one(exec)
    .await?;

    Ok(row.0)
}

pub(crate) async fn get_user_created_at(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<DateTime<Utc>>> {
    let row: Option<(DateTime<Utc>,)> =
        sqlx::query_as("SELECT created_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(exec)
            .await?;
    Ok(row.map(|r| r.0))
}

pub(crate) async fn create_oauth_user(
    exec: impl PgExecutor<'_>,
    email: &str,
    username: &str,
    display_name: Option<&str>,
    oauth_provider: &str,
    oauth_provider_id: &str,
) -> Result<User> {
    let sql = format!(
        "INSERT INTO users (email, username, display_name, oauth_provider, oauth_provider_id) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING {USER_COLS}"
    );
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(email)
        .bind(username)
        .bind(display_name)
        .bind(oauth_provider)
        .bind(oauth_provider_id)
        .fetch_one(exec)
        .await?;

    Ok(user)
}

pub(crate) async fn find_by_oauth(
    exec: impl PgExecutor<'_>,
    provider: &str,
    provider_id: &str,
) -> Result<Option<User>> {
    let sql = format!(
        "SELECT {USER_COLS} FROM users WHERE oauth_provider = $1 AND oauth_provider_id = $2"
    );
    let user = sqlx::query_as::<_, User>(&sql)
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(exec)
        .await?;

    Ok(user)
}

pub(crate) async fn link_oauth(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    provider: &str,
    provider_id: &str,
) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE users SET oauth_provider = $1, oauth_provider_id = $2, updated_at = NOW() \
         WHERE id = $3",
    )
    .bind(provider)
    .bind(provider_id)
    .bind(user_id)
    .execute(exec)
    .await?;

    Ok(result.rows_affected() > 0)
}
