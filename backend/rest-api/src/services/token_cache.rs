use async_trait::async_trait;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::traits;
use crate::utils::jwt::REFRESH_TOKEN_TTL_SECS;

// ── Concrete implementation ──────────────────────────────────────────

pub struct RedisTokenCache {
    client: redis::Client,
}

impl RedisTokenCache {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl traits::TokenCache for RedisTokenCache {
    async fn store(&self, token_hash: &str, user_id: Uuid) {
        store(&self.client, token_hash, user_id).await;
    }

    async fn get(&self, token_hash: &str) -> Option<Uuid> {
        get(&self.client, token_hash).await
    }

    async fn delete(&self, token_hash: &str) {
        delete(&self.client, token_hash).await;
    }

    async fn delete_many(&self, token_hashes: &[String]) {
        delete_many(&self.client, token_hashes).await;
    }
}

// ── Free functions (private — no transactional use outside this module) ──

fn cache_key(token_hash: &str) -> String {
    format!("refresh:{token_hash}")
}

/// Store a refresh token hash → user_id mapping in Redis with TTL.
async fn store(client: &redis::Client, token_hash: &str, user_id: Uuid) {
    let result: Result<(), redis::RedisError> = async {
        let mut conn = client.get_multiplexed_async_connection().await?;
        conn.set_ex::<_, _, ()>(
            cache_key(token_hash),
            user_id.to_string(),
            REFRESH_TOKEN_TTL_SECS,
        )
        .await?;
        Ok(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "failed to store refresh token in Redis cache");
    }
}

/// Look up user_id by refresh token hash in Redis.
async fn get(client: &redis::Client, token_hash: &str) -> Option<Uuid> {
    let result: Result<Option<String>, redis::RedisError> = async {
        let mut conn = client.get_multiplexed_async_connection().await?;
        conn.get(cache_key(token_hash)).await
    }
    .await;

    match result {
        Ok(Some(val)) => val.parse().ok(),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!(error = %e, "failed to get refresh token from Redis cache");
            None
        }
    }
}

/// Delete a single refresh token hash from Redis.
async fn delete(client: &redis::Client, token_hash: &str) {
    let result: Result<(), redis::RedisError> = async {
        let mut conn = client.get_multiplexed_async_connection().await?;
        conn.del::<_, ()>(cache_key(token_hash)).await?;
        Ok(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "failed to delete refresh token from Redis cache");
    }
}

/// Delete multiple refresh token hashes from Redis.
async fn delete_many(client: &redis::Client, token_hashes: &[String]) {
    if token_hashes.is_empty() {
        return;
    }

    let keys: Vec<String> = token_hashes.iter().map(|h| cache_key(h)).collect();

    let result: Result<(), redis::RedisError> = async {
        let mut conn = client.get_multiplexed_async_connection().await?;
        conn.del::<_, ()>(keys).await?;
        Ok(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "failed to delete refresh tokens from Redis cache");
    }
}
