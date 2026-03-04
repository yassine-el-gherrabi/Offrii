use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn create_pg_pool(url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    Ok(pool)
}

pub fn create_redis_client(url: &str) -> anyhow::Result<redis::Client> {
    let client = redis::Client::open(url)?;
    Ok(client)
}
