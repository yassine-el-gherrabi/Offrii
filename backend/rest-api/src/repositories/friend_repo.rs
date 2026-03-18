use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, Row};
use uuid::Uuid;

use crate::models::friend::{FriendRequest, FriendRequestStatus, FriendWithSince, Friendship};
use crate::traits;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgFriendRepo {
    pool: PgPool,
}

impl PgFriendRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::FriendRepo for PgFriendRepo {
    async fn create_friend_request(
        &self,
        from_user_id: Uuid,
        to_user_id: Uuid,
    ) -> Result<FriendRequest> {
        create_friend_request(&self.pool, from_user_id, to_user_id).await
    }

    async fn find_pending_requests(&self, to_user_id: Uuid) -> Result<Vec<FriendRequest>> {
        find_pending_requests(&self.pool, to_user_id).await
    }

    async fn find_request_by_id(&self, id: Uuid) -> Result<Option<FriendRequest>> {
        find_request_by_id(&self.pool, id).await
    }

    async fn update_request_status(&self, id: Uuid, status: FriendRequestStatus) -> Result<bool> {
        update_request_status(&self.pool, id, status).await
    }

    async fn create_friendship(&self, user_a_id: Uuid, user_b_id: Uuid) -> Result<Friendship> {
        create_friendship(&self.pool, user_a_id, user_b_id).await
    }

    async fn delete_friendship(&self, user_a_id: Uuid, user_b_id: Uuid) -> Result<bool> {
        delete_friendship(&self.pool, user_a_id, user_b_id).await
    }

    async fn list_friends(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        list_friends(&self.pool, user_id).await
    }

    async fn are_friends(&self, user_a_id: Uuid, user_b_id: Uuid) -> Result<bool> {
        are_friends(&self.pool, user_a_id, user_b_id).await
    }

    async fn list_friends_with_since(&self, user_id: Uuid) -> Result<Vec<FriendWithSince>> {
        list_friends_with_since(&self.pool, user_id).await
    }

    async fn find_sent_requests(&self, from_user_id: Uuid) -> Result<Vec<FriendRequest>> {
        find_sent_requests(&self.pool, from_user_id).await
    }

    async fn find_pending_between(
        &self,
        from_user_id: Uuid,
        to_user_id: Uuid,
    ) -> Result<Option<FriendRequest>> {
        find_pending_between(&self.pool, from_user_id, to_user_id).await
    }

    async fn count_shared_items_per_user(
        &self,
        friend_ids: &[Uuid],
        viewer_id: Uuid,
    ) -> Result<std::collections::HashMap<Uuid, i64>> {
        count_shared_items_per_user(&self.pool, friend_ids, viewer_id).await
    }
}

// ── Free functions (kept pub(crate) for transactional use) ───────────

pub(crate) async fn create_friend_request(
    exec: impl PgExecutor<'_>,
    from_user_id: Uuid,
    to_user_id: Uuid,
) -> Result<FriendRequest> {
    let req = sqlx::query_as::<_, FriendRequest>(
        "INSERT INTO friend_requests (from_user_id, to_user_id) \
         VALUES ($1, $2) \
         RETURNING id, from_user_id, to_user_id, status, created_at",
    )
    .bind(from_user_id)
    .bind(to_user_id)
    .fetch_one(exec)
    .await?;

    Ok(req)
}

pub(crate) async fn find_pending_requests(
    exec: impl PgExecutor<'_>,
    to_user_id: Uuid,
) -> Result<Vec<FriendRequest>> {
    let reqs = sqlx::query_as::<_, FriendRequest>(
        "SELECT id, from_user_id, to_user_id, status, created_at \
         FROM friend_requests \
         WHERE to_user_id = $1 AND status = 'pending' \
         ORDER BY created_at DESC",
    )
    .bind(to_user_id)
    .fetch_all(exec)
    .await?;

    Ok(reqs)
}

pub(crate) async fn find_request_by_id(
    exec: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<FriendRequest>> {
    let req = sqlx::query_as::<_, FriendRequest>(
        "SELECT id, from_user_id, to_user_id, status, created_at \
         FROM friend_requests WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(exec)
    .await?;

    Ok(req)
}

pub(crate) async fn update_request_status(
    exec: impl PgExecutor<'_>,
    id: Uuid,
    status: FriendRequestStatus,
) -> Result<bool> {
    let result =
        sqlx::query("UPDATE friend_requests SET status = $1 WHERE id = $2 AND status = 'pending'")
            .bind(status.as_str())
            .bind(id)
            .execute(exec)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn create_friendship(
    exec: impl PgExecutor<'_>,
    user_a_id: Uuid,
    user_b_id: Uuid,
) -> Result<Friendship> {
    // Canonical ordering: smaller UUID first
    let (a, b) = if user_a_id < user_b_id {
        (user_a_id, user_b_id)
    } else {
        (user_b_id, user_a_id)
    };

    let fs = sqlx::query_as::<_, Friendship>(
        "INSERT INTO friendships (user_a_id, user_b_id) \
         VALUES ($1, $2) \
         RETURNING user_a_id, user_b_id, created_at",
    )
    .bind(a)
    .bind(b)
    .fetch_one(exec)
    .await?;

    Ok(fs)
}

pub(crate) async fn delete_friendship(
    exec: impl PgExecutor<'_>,
    user_a_id: Uuid,
    user_b_id: Uuid,
) -> Result<bool> {
    let (a, b) = if user_a_id < user_b_id {
        (user_a_id, user_b_id)
    } else {
        (user_b_id, user_a_id)
    };

    let result = sqlx::query("DELETE FROM friendships WHERE user_a_id = $1 AND user_b_id = $2")
        .bind(a)
        .bind(b)
        .execute(exec)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn list_friends(exec: impl PgExecutor<'_>, user_id: Uuid) -> Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT user_b_id AS friend_id FROM friendships WHERE user_a_id = $1 \
         UNION ALL \
         SELECT user_a_id AS friend_id FROM friendships WHERE user_b_id = $1",
    )
    .bind(user_id)
    .fetch_all(exec)
    .await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub(crate) async fn are_friends(
    exec: impl PgExecutor<'_>,
    user_a_id: Uuid,
    user_b_id: Uuid,
) -> Result<bool> {
    let (a, b) = if user_a_id < user_b_id {
        (user_a_id, user_b_id)
    } else {
        (user_b_id, user_a_id)
    };

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM friendships WHERE user_a_id = $1 AND user_b_id = $2)",
    )
    .bind(a)
    .bind(b)
    .fetch_one(exec)
    .await?;

    Ok(exists)
}

pub(crate) async fn list_friends_with_since(
    exec: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<FriendWithSince>> {
    let rows = sqlx::query_as::<_, FriendWithSince>(
        "SELECT u.id AS user_id, u.username, u.display_name, f.created_at AS since \
         FROM friendships f \
         JOIN users u ON u.id = CASE WHEN f.user_a_id = $1 THEN f.user_b_id ELSE f.user_a_id END \
         WHERE f.user_a_id = $1 OR f.user_b_id = $1 \
         ORDER BY f.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(exec)
    .await?;

    Ok(rows)
}

pub(crate) async fn find_sent_requests(
    exec: impl PgExecutor<'_>,
    from_user_id: Uuid,
) -> Result<Vec<FriendRequest>> {
    let reqs = sqlx::query_as::<_, FriendRequest>(
        "SELECT id, from_user_id, to_user_id, status, created_at \
         FROM friend_requests \
         WHERE from_user_id = $1 AND status = 'pending' \
         ORDER BY created_at DESC",
    )
    .bind(from_user_id)
    .fetch_all(exec)
    .await?;

    Ok(reqs)
}

pub(crate) async fn find_pending_between(
    exec: impl PgExecutor<'_>,
    from_user_id: Uuid,
    to_user_id: Uuid,
) -> Result<Option<FriendRequest>> {
    let req = sqlx::query_as::<_, FriendRequest>(
        "SELECT id, from_user_id, to_user_id, status, created_at \
         FROM friend_requests \
         WHERE from_user_id = $1 AND to_user_id = $2 AND status = 'pending'",
    )
    .bind(from_user_id)
    .bind(to_user_id)
    .fetch_optional(exec)
    .await?;

    Ok(req)
}

/// Count items shared via direct (1:1) circles between each friend and the viewer.
/// Only counts items in the direct circle between the two specific users.
pub(crate) async fn count_shared_items_per_user(
    exec: impl PgExecutor<'_>,
    friend_ids: &[Uuid],
    viewer_id: Uuid,
) -> Result<HashMap<Uuid, i64>> {
    if friend_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        "SELECT i.user_id, COUNT(DISTINCT i.id) AS cnt \
         FROM items i \
         JOIN circle_items ci ON ci.item_id = i.id \
         JOIN circles c ON c.id = ci.circle_id \
         JOIN circle_members cm ON cm.circle_id = ci.circle_id \
         WHERE i.user_id = ANY($1) \
           AND i.status = 'active' \
           AND cm.user_id = $2 \
           AND c.is_direct = true \
         GROUP BY i.user_id",
    )
    .bind(friend_ids)
    .bind(viewer_id)
    .fetch_all(exec)
    .await?;

    let map = rows
        .into_iter()
        .map(|row| {
            let uid: Uuid = row.get("user_id");
            let cnt: i64 = row.get("cnt");
            (uid, cnt)
        })
        .collect();

    Ok(map)
}
