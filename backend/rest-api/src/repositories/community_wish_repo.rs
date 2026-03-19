use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::community_wish::{CommunityWish, WishStatus};
use crate::traits;

const WISH_COLS: &str = "\
    id, owner_id, title, description, category, status, is_anonymous, \
    matched_with, matched_at, fulfilled_at, closed_at, \
    report_count, reopen_count, last_reopen_at, moderation_note, \
    image_url, links, \
    created_at, updated_at";

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCommunityWishRepo {
    pool: PgPool,
}

impl PgCommunityWishRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::CommunityWishRepo for PgCommunityWishRepo {
    async fn create(
        &self,
        owner_id: Uuid,
        title: &str,
        description: Option<&str>,
        category: &str,
        is_anonymous: bool,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<CommunityWish> {
        create(
            &self.pool,
            owner_id,
            title,
            description,
            category,
            is_anonymous,
            image_url,
            links,
        )
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CommunityWish>> {
        find_by_id(&self.pool, id).await
    }

    async fn list_open(
        &self,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommunityWish>> {
        list_open(&self.pool, category, limit, offset).await
    }

    async fn count_open(&self, category: Option<&str>) -> Result<i64> {
        count_open(&self.pool, category).await
    }

    async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<CommunityWish>> {
        list_by_owner(&self.pool, owner_id).await
    }

    async fn list_by_donor(&self, donor_id: Uuid) -> Result<Vec<CommunityWish>> {
        list_by_donor(&self.pool, donor_id).await
    }

    async fn count_active_by_owner(&self, owner_id: Uuid) -> Result<i64> {
        count_active_by_owner(&self.pool, owner_id).await
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: WishStatus,
        moderation_note: Option<&str>,
    ) -> Result<bool> {
        update_status(&self.pool, id, status.as_str(), moderation_note).await
    }

    async fn set_matched(
        &self,
        id: Uuid,
        donor_id: Uuid,
        matched_at: DateTime<Utc>,
    ) -> Result<bool> {
        set_matched(&self.pool, id, donor_id, matched_at).await
    }

    async fn clear_match(&self, id: Uuid) -> Result<bool> {
        clear_match(&self.pool, id).await
    }

    async fn set_fulfilled(&self, id: Uuid, fulfilled_at: DateTime<Utc>) -> Result<bool> {
        set_fulfilled(&self.pool, id, fulfilled_at).await
    }

    async fn set_closed(&self, id: Uuid, closed_at: DateTime<Utc>) -> Result<bool> {
        set_closed(&self.pool, id, closed_at).await
    }

    async fn increment_report_count(&self, id: Uuid) -> Result<i32> {
        increment_report_count(&self.pool, id).await
    }

    async fn reset_reports(&self, id: Uuid) -> Result<bool> {
        reset_reports(&self.pool, id).await
    }

    async fn increment_reopen_count(&self, id: Uuid, now: DateTime<Utc>) -> Result<i32> {
        increment_reopen_count(&self.pool, id, now).await
    }

    async fn update_content(
        &self,
        id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        category: Option<&str>,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<Option<CommunityWish>> {
        update_content(
            &self.pool,
            id,
            title,
            description,
            category,
            image_url,
            links,
        )
        .await
    }

    async fn list_flagged(&self, limit: i64, offset: i64) -> Result<Vec<CommunityWish>> {
        list_flagged(&self.pool, limit, offset).await
    }

    async fn count_flagged(&self) -> Result<i64> {
        count_flagged(&self.pool).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM community_wishes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ── Free functions (for transaction support) ─────────────────────────

#[allow(clippy::too_many_arguments)]
pub(crate) async fn create(
    exec: impl PgExecutor<'_>,
    owner_id: Uuid,
    title: &str,
    description: Option<&str>,
    category: &str,
    is_anonymous: bool,
    image_url: Option<&str>,
    links: Option<&[String]>,
) -> Result<CommunityWish> {
    let sql = format!(
        "INSERT INTO community_wishes (owner_id, title, description, category, is_anonymous, image_url, links) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING {WISH_COLS}"
    );
    let wish = sqlx::query_as::<_, CommunityWish>(&sql)
        .bind(owner_id)
        .bind(title)
        .bind(description)
        .bind(category)
        .bind(is_anonymous)
        .bind(image_url)
        .bind(links)
        .fetch_one(exec)
        .await?;
    Ok(wish)
}

pub(crate) async fn find_by_id(
    exec: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<CommunityWish>> {
    let sql = format!("SELECT {WISH_COLS} FROM community_wishes WHERE id = $1");
    let wish = sqlx::query_as::<_, CommunityWish>(&sql)
        .bind(id)
        .fetch_optional(exec)
        .await?;
    Ok(wish)
}

pub(crate) async fn list_open(
    exec: impl PgExecutor<'_>,
    category: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<CommunityWish>> {
    let sql = if category.is_some() {
        format!(
            "SELECT {WISH_COLS} FROM community_wishes \
             WHERE status = 'open' AND category = $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
    } else {
        format!(
            "SELECT {WISH_COLS} FROM community_wishes \
             WHERE status = 'open' \
             ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
    };

    let wishes = if let Some(cat) = category {
        sqlx::query_as::<_, CommunityWish>(&sql)
            .bind(cat)
            .bind(limit)
            .bind(offset)
            .fetch_all(exec)
            .await?
    } else {
        sqlx::query_as::<_, CommunityWish>(&sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(exec)
            .await?
    };
    Ok(wishes)
}

pub(crate) async fn count_open(exec: impl PgExecutor<'_>, category: Option<&str>) -> Result<i64> {
    let count: (i64,) = if let Some(cat) = category {
        sqlx::query_as(
            "SELECT COUNT(*) FROM community_wishes WHERE status = 'open' AND category = $1",
        )
        .bind(cat)
        .fetch_one(exec)
        .await?
    } else {
        sqlx::query_as("SELECT COUNT(*) FROM community_wishes WHERE status = 'open'")
            .fetch_one(exec)
            .await?
    };
    Ok(count.0)
}

pub(crate) async fn list_by_owner(
    exec: impl PgExecutor<'_>,
    owner_id: Uuid,
) -> Result<Vec<CommunityWish>> {
    let sql = format!(
        "SELECT {WISH_COLS} FROM community_wishes \
         WHERE owner_id = $1 \
         ORDER BY created_at DESC"
    );
    let wishes = sqlx::query_as::<_, CommunityWish>(&sql)
        .bind(owner_id)
        .fetch_all(exec)
        .await?;
    Ok(wishes)
}

pub(crate) async fn list_by_donor(
    exec: impl PgExecutor<'_>,
    donor_id: Uuid,
) -> Result<Vec<CommunityWish>> {
    let sql = format!(
        "SELECT {WISH_COLS} FROM community_wishes \
         WHERE matched_with = $1 \
         ORDER BY created_at DESC"
    );
    let wishes = sqlx::query_as::<_, CommunityWish>(&sql)
        .bind(donor_id)
        .fetch_all(exec)
        .await?;
    Ok(wishes)
}

pub(crate) async fn count_active_by_owner(
    exec: impl PgExecutor<'_>,
    owner_id: Uuid,
) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM community_wishes \
         WHERE owner_id = $1 AND status IN ('open', 'matched', 'pending')",
    )
    .bind(owner_id)
    .fetch_one(exec)
    .await?;
    Ok(count.0)
}

pub(crate) async fn update_status(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    status: &str,
    moderation_note: Option<&str>,
) -> Result<bool> {
    let rows = if let Some(note) = moderation_note {
        sqlx::query("UPDATE community_wishes SET status = $1, moderation_note = $2 WHERE id = $3")
            .bind(status)
            .bind(note)
            .bind(id)
            .execute(exec)
            .await?
            .rows_affected()
    } else {
        sqlx::query("UPDATE community_wishes SET status = $1 WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(exec)
            .await?
            .rows_affected()
    };
    Ok(rows > 0)
}

pub(crate) async fn set_matched(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    donor_id: Uuid,
    matched_at: DateTime<Utc>,
) -> Result<bool> {
    let rows = sqlx::query(
        "UPDATE community_wishes \
         SET status = 'matched', matched_with = $1, matched_at = $2 \
         WHERE id = $3 AND status = 'open'",
    )
    .bind(donor_id)
    .bind(matched_at)
    .bind(id)
    .execute(exec)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub(crate) async fn clear_match(exec: impl PgExecutor<'_>, id: Uuid) -> Result<bool> {
    let rows = sqlx::query(
        "UPDATE community_wishes \
         SET status = 'open', matched_with = NULL, matched_at = NULL \
         WHERE id = $1",
    )
    .bind(id)
    .execute(exec)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub(crate) async fn set_fulfilled(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    fulfilled_at: DateTime<Utc>,
) -> Result<bool> {
    let rows = sqlx::query(
        "UPDATE community_wishes \
         SET status = 'fulfilled', fulfilled_at = $1 \
         WHERE id = $2",
    )
    .bind(fulfilled_at)
    .bind(id)
    .execute(exec)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub(crate) async fn set_closed(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    closed_at: DateTime<Utc>,
) -> Result<bool> {
    let rows = sqlx::query(
        "UPDATE community_wishes \
         SET status = 'closed', closed_at = $1 \
         WHERE id = $2",
    )
    .bind(closed_at)
    .bind(id)
    .execute(exec)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub(crate) async fn increment_report_count(exec: impl PgExecutor<'_>, id: Uuid) -> Result<i32> {
    let row: (i32,) = sqlx::query_as(
        "UPDATE community_wishes \
         SET report_count = report_count + 1 \
         WHERE id = $1 \
         RETURNING report_count",
    )
    .bind(id)
    .fetch_one(exec)
    .await?;
    Ok(row.0)
}

pub(crate) async fn reset_reports(exec: impl PgExecutor<'_>, id: Uuid) -> Result<bool> {
    let rows = sqlx::query("UPDATE community_wishes SET report_count = 0 WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

pub(crate) async fn increment_reopen_count(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    now: DateTime<Utc>,
) -> Result<i32> {
    let row: (i32,) = sqlx::query_as(
        "UPDATE community_wishes \
         SET reopen_count = reopen_count + 1, last_reopen_at = $1 \
         WHERE id = $2 \
         RETURNING reopen_count",
    )
    .bind(now)
    .bind(id)
    .fetch_one(exec)
    .await?;
    Ok(row.0)
}

pub(crate) async fn update_content(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    category: Option<&str>,
    image_url: Option<&str>,
    links: Option<&[String]>,
) -> Result<Option<CommunityWish>> {
    let mut builder = sqlx::QueryBuilder::new("UPDATE community_wishes SET ");
    let mut sep = builder.separated(", ");
    let mut has_update = false;

    if let Some(t) = title {
        sep.push("title = ").push_bind_unseparated(t);
        has_update = true;
    }
    if let Some(d) = description {
        sep.push("description = ").push_bind_unseparated(d);
        has_update = true;
    }
    if let Some(c) = category {
        sep.push("category = ").push_bind_unseparated(c);
        has_update = true;
    }
    if let Some(url) = image_url {
        sep.push("image_url = ").push_bind_unseparated(url);
        has_update = true;
    }
    if let Some(l) = links {
        sep.push("links = ").push_bind_unseparated(l.to_vec());
        has_update = true;
    }

    if !has_update {
        return find_by_id(exec, id).await;
    }

    builder.push(" WHERE id = ").push_bind(id);
    builder.push(format!(" RETURNING {WISH_COLS}"));

    let wish = builder
        .build_query_as::<CommunityWish>()
        .fetch_optional(exec)
        .await?;
    Ok(wish)
}

pub(crate) async fn list_flagged(
    exec: impl PgExecutor<'_>,
    limit: i64,
    offset: i64,
) -> Result<Vec<CommunityWish>> {
    let sql = format!(
        "SELECT {WISH_COLS} FROM community_wishes \
         WHERE status IN ('flagged', 'review') \
         ORDER BY created_at ASC \
         LIMIT $1 OFFSET $2"
    );
    let wishes = sqlx::query_as::<_, CommunityWish>(&sql)
        .bind(limit)
        .bind(offset)
        .fetch_all(exec)
        .await?;
    Ok(wishes)
}

pub(crate) async fn count_flagged(exec: impl PgExecutor<'_>) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM community_wishes WHERE status IN ('flagged', 'review')",
    )
    .fetch_one(exec)
    .await?;
    Ok(count.0)
}
