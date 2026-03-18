use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Notification;
use crate::traits;

pub struct PgNotificationRepo {
    pool: PgPool,
}

impl PgNotificationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl traits::NotificationRepo for PgNotificationRepo {
    async fn create(
        &self,
        user_id: Uuid,
        notif_type: &str,
        title: &str,
        body: &str,
        circle_id: Option<Uuid>,
        item_id: Option<Uuid>,
        actor_id: Option<Uuid>,
    ) -> Result<Notification> {
        let notif = sqlx::query_as::<_, Notification>(
            "INSERT INTO notifications (user_id, type, title, body, circle_id, item_id, actor_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING id, user_id, type, title, body, read, circle_id, item_id, actor_id, created_at",
        )
        .bind(user_id)
        .bind(notif_type)
        .bind(title)
        .bind(body)
        .bind(circle_id)
        .bind(item_id)
        .bind(actor_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(notif)
    }

    async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>> {
        let notifs = sqlx::query_as::<_, Notification>(
            "SELECT id, user_id, type, title, body, read, circle_id, item_id, actor_id, created_at \
             FROM notifications WHERE user_id = $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(notifs)
    }

    async fn count_unread(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read = FALSE",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    async fn count_total(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    async fn mark_read(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result =
            sqlx::query("UPDATE notifications SET read = TRUE WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn mark_all_read(&self, user_id: Uuid) -> Result<i64> {
        let result =
            sqlx::query("UPDATE notifications SET read = TRUE WHERE user_id = $1 AND read = FALSE")
                .bind(user_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
