use std::env;

#[derive(Debug)]
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

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// # Safety
    /// Must be called while holding ENV_LOCK — single-threaded env access.
    unsafe fn cleanup_env() {
        unsafe {
            env::remove_var("DATABASE_URL");
            env::remove_var("REDIS_URL");
            env::remove_var("API_PORT");
        }
    }

    #[test]
    fn from_env_all_set() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            cleanup_env();
            env::set_var("DATABASE_URL", "postgres://localhost/test");
            env::set_var("REDIS_URL", "redis://custom:1234");
            env::set_var("API_PORT", "8080");
        }

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.database_url, "postgres://localhost/test");
        assert_eq!(cfg.redis_url, "redis://custom:1234");
        assert_eq!(cfg.api_port, 8080);

        unsafe { cleanup_env() };
    }

    #[test]
    fn from_env_missing_database_url() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { cleanup_env() };

        let err = Config::from_env().unwrap_err();
        assert!(
            err.to_string().contains("DATABASE_URL must be set"),
            "unexpected error: {err}"
        );

        unsafe { cleanup_env() };
    }

    #[test]
    fn from_env_defaults_redis_url() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            cleanup_env();
            env::set_var("DATABASE_URL", "postgres://localhost/test");
        }

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.redis_url, "redis://localhost:6379");

        unsafe { cleanup_env() };
    }

    #[test]
    fn from_env_defaults_api_port() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            cleanup_env();
            env::set_var("DATABASE_URL", "postgres://localhost/test");
        }

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.api_port, 3000);

        unsafe { cleanup_env() };
    }

    #[test]
    fn from_env_invalid_api_port() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            cleanup_env();
            env::set_var("DATABASE_URL", "postgres://localhost/test");
            env::set_var("API_PORT", "not_a_number");
        }

        let err = Config::from_env().unwrap_err();
        assert!(
            err.to_string().contains("API_PORT must be a valid u16"),
            "unexpected error: {err}"
        );

        unsafe { cleanup_env() };
    }
}
