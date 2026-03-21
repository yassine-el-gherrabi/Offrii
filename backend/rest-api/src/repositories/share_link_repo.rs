use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool, QueryBuilder};
use uuid::Uuid;

use crate::models::ShareLink;
use crate::traits;

const SHARE_LINK_COLS: &str =
    "id, user_id, token, label, permissions, scope, scope_data, created_at, expires_at, is_active";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgShareLinkRepo {
    pool: PgPool,
}

impl PgShareLinkRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::ShareLinkRepo for PgShareLinkRepo {
    async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: Option<DateTime<Utc>>,
        label: Option<&str>,
        permissions: &str,
        scope: &str,
        scope_data: Option<&serde_json::Value>,
    ) -> Result<ShareLink> {
        create(
            &self.pool,
            user_id,
            token,
            expires_at,
            label,
            permissions,
            scope,
            scope_data,
        )
        .await
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<ShareLink>> {
        list_by_user(&self.pool, user_id).await
    }

    async fn list_by_user_paginated(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ShareLink>> {
        list_by_user_paginated(&self.pool, user_id, limit, offset).await
    }

    async fn count_by_user(&self, user_id: Uuid) -> Result<i64> {
        count_by_user(&self.pool, user_id).await
    }

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<ShareLink>> {
        find_by_id(&self.pool, id, user_id).await
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<ShareLink>> {
        find_by_token(&self.pool, token).await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        delete(&self.pool, id, user_id).await
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        label: Option<&str>,
        is_active: Option<bool>,
        permissions: Option<&str>,
        expires_at: Option<Option<DateTime<Utc>>>,
    ) -> Result<Option<ShareLink>> {
        update(
            &self.pool,
            id,
            user_id,
            label,
            is_active,
            permissions,
            expires_at,
        )
        .await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

#[allow(clippy::too_many_arguments)]
pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    token: &str,
    expires_at: Option<DateTime<Utc>>,
    label: Option<&str>,
    permissions: &str,
    scope: &str,
    scope_data: Option<&serde_json::Value>,
) -> Result<ShareLink> {
    let sql = format!(
        "INSERT INTO share_links (user_id, token, expires_at, label, permissions, scope, scope_data) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING {SHARE_LINK_COLS}"
    );
    let link = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .bind(label)
        .bind(permissions)
        .bind(scope)
        .bind(scope_data)
        .fetch_one(exec)
        .await?;

    Ok(link)
}

pub(crate) async fn list_by_user(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<ShareLink>> {
    let sql = format!(
        "SELECT {SHARE_LINK_COLS} FROM share_links \
         WHERE user_id = $1 \
         ORDER BY created_at DESC"
    );
    let links = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(user_id)
        .fetch_all(exec)
        .await?;

    Ok(links)
}

pub(crate) async fn list_by_user_paginated(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ShareLink>> {
    let sql = format!(
        "SELECT {SHARE_LINK_COLS} FROM share_links \
         WHERE user_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    );
    let links = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(exec)
        .await?;
    Ok(links)
}

pub(crate) async fn count_by_user(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<i64> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM share_links WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(exec)
        .await?;
    Ok(count)
}

pub(crate) async fn find_by_id(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<ShareLink>> {
    let sql = format!("SELECT {SHARE_LINK_COLS} FROM share_links WHERE id = $1 AND user_id = $2");
    let link = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(exec)
        .await?;

    Ok(link)
}

pub(crate) async fn find_by_token(
    exec: impl PgExecutor<'_>,
    token: &str,
) -> Result<Option<ShareLink>> {
    let sql = format!("SELECT {SHARE_LINK_COLS} FROM share_links WHERE token = $1");
    let link = sqlx::query_as::<_, ShareLink>(&sql)
        .bind(token)
        .fetch_optional(exec)
        .await?;

    Ok(link)
}

pub(crate) async fn delete(exec: impl PgExecutor<'_>, id: Uuid, user_id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM share_links WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn update(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    label: Option<&str>,
    is_active: Option<bool>,
    permissions: Option<&str>,
    expires_at: Option<Option<DateTime<Utc>>>,
) -> Result<Option<ShareLink>> {
    let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("UPDATE share_links SET ");
    let mut separated = qb.separated(", ");

    if let Some(l) = label {
        separated.push("label = ");
        separated.push_bind_unseparated(l);
    }
    if let Some(a) = is_active {
        separated.push("is_active = ");
        separated.push_bind_unseparated(a);
    }
    if let Some(p) = permissions {
        separated.push("permissions = ");
        separated.push_bind_unseparated(p);
    }
    if let Some(e) = expires_at {
        separated.push("expires_at = ");
        separated.push_bind_unseparated(e);
    }

    qb.push(" WHERE id = ");
    qb.push_bind(id);
    qb.push(" AND user_id = ");
    qb.push_bind(user_id);

    qb.push(format!(" RETURNING {SHARE_LINK_COLS}"));

    let link = qb
        .build_query_as::<ShareLink>()
        .fetch_optional(exec)
        .await?;

    Ok(link)
}
