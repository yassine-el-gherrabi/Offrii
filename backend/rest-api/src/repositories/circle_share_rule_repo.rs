use anyhow::Result;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::CircleShareRule;

pub struct PgCircleShareRuleRepo {
    pool: sqlx::PgPool,
}

impl PgCircleShareRuleRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl crate::traits::CircleShareRuleRepo for PgCircleShareRuleRepo {
    async fn get(&self, circle_id: Uuid, user_id: Uuid) -> Result<Option<CircleShareRule>> {
        get(&self.pool, circle_id, user_id).await
    }

    async fn upsert(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        share_mode: &str,
        category_ids: &[Uuid],
    ) -> Result<CircleShareRule> {
        upsert(&self.pool, circle_id, user_id, share_mode, category_ids).await
    }

    async fn delete(&self, circle_id: Uuid, user_id: Uuid) -> Result<bool> {
        delete(&self.pool, circle_id, user_id).await
    }
}

pub(crate) async fn get(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    user_id: Uuid,
) -> Result<Option<CircleShareRule>> {
    let rule = sqlx::query_as::<_, CircleShareRule>(
        "SELECT * FROM circle_share_rules WHERE circle_id = $1 AND user_id = $2",
    )
    .bind(circle_id)
    .bind(user_id)
    .fetch_optional(exec)
    .await?;

    Ok(rule)
}

pub(crate) async fn upsert(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    user_id: Uuid,
    share_mode: &str,
    category_ids: &[Uuid],
) -> Result<CircleShareRule> {
    let rule = sqlx::query_as::<_, CircleShareRule>(
        "INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (circle_id, user_id) \
         DO UPDATE SET share_mode = $3, category_ids = $4, updated_at = NOW() \
         RETURNING *",
    )
    .bind(circle_id)
    .bind(user_id)
    .bind(share_mode)
    .bind(category_ids)
    .fetch_one(exec)
    .await?;

    Ok(rule)
}

pub(crate) async fn delete(
    exec: impl PgExecutor<'_>,
    circle_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let result =
        sqlx::query("DELETE FROM circle_share_rules WHERE circle_id = $1 AND user_id = $2")
            .bind(circle_id)
            .bind(user_id)
            .execute(exec)
            .await?;

    Ok(result.rows_affected() > 0)
}
