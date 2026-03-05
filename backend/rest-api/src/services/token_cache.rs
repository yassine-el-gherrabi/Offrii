use redis::AsyncCommands;
use uuid::Uuid;

use crate::utils::jwt::REFRESH_TOKEN_TTL_SECS;

fn cache_key(token_hash: &str) -> String {
    format!("refresh:{token_hash}")
}

/// Store a refresh token hash → user_id mapping in Redis with TTL.
pub async fn store(client: &redis::Client, token_hash: &str, user_id: Uuid) {
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
pub async fn get(client: &redis::Client, token_hash: &str) -> Option<Uuid> {
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
pub async fn delete(client: &redis::Client, token_hash: &str) {
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
pub async fn delete_many(client: &redis::Client, token_hashes: &[String]) {
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
