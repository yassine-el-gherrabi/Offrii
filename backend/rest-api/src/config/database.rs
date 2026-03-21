use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn create_pg_pool(url: &str) -> anyhow::Result<PgPool> {
    let max_conn: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let pool = PgPoolOptions::new()
        .max_connections(max_conn)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(url)
        .await?;
    Ok(pool)
}

/// Create a Redis client from the given URL.
///
/// The returned `redis::Client` is intentionally shared as-is (via `Arc` or clone)
/// across services. Callers obtain connections via `client.get_multiplexed_async_connection()`.
/// In redis-rs >= 0.25 (we use 1.0), this method returns a **multiplexed** handle that
/// reuses a single underlying TCP connection -- it does NOT open a new connection each time.
/// This makes an explicit `ConnectionManager` or pre-created `MultiplexedConnection`
/// unnecessary for our use case.
pub fn create_redis_client(url: &str) -> anyhow::Result<redis::Client> {
    let client = redis::Client::open(url)?;
    Ok(client)
}
