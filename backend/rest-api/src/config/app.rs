use std::env;

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub api_port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let api_port = env::var("API_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| anyhow::anyhow!("API_PORT must be a valid u16"))?;

        Ok(Self {
            database_url,
            redis_url,
            api_port,
        })
    }
}
