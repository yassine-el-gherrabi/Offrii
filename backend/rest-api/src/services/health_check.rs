use async_trait::async_trait;
use sqlx::PgPool;

use crate::traits;

pub struct PgHealthCheck {
    pool: PgPool,
    redis: redis::Client,
}

impl PgHealthCheck {
    pub fn new(pool: PgPool, redis: redis::Client) -> Self {
        Self { pool, redis }
    }
}

#[async_trait]
impl traits::HealthCheck for PgHealthCheck {
    async fn check_db(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await.is_ok()
    }

    async fn check_cache(&self) -> bool {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            return false;
        };
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .is_ok()
    }
}
