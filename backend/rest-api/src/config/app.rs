use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub api_port: u16,
    pub resend_api_key: String,
    pub email_from: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_vars(|key| env::var(key))
    }

    pub fn from_vars(var: impl Fn(&str) -> Result<String, env::VarError>) -> anyhow::Result<Self> {
        let database_url =
            var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        let redis_url = var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let api_port = var("API_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| anyhow::anyhow!("API_PORT must be a valid u16"))?;

        let resend_api_key =
            var("RESEND_API_KEY").map_err(|_| anyhow::anyhow!("RESEND_API_KEY must be set"))?;

        let email_from =
            var("EMAIL_FROM").unwrap_or_else(|_| "Offrii <noreply@offrii.com>".to_string());

        Ok(Self {
            database_url,
            redis_url,
            api_port,
            resend_api_key,
            email_from,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    macro_rules! vars {
        ($($key:expr => $val:expr),* $(,)?) => {{
            let map: HashMap<&str, &str> = HashMap::from([$(($key, $val)),*]);
            move |key: &str| -> Result<String, env::VarError> {
                map.get(key)
                    .map(|v| v.to_string())
                    .ok_or(env::VarError::NotPresent)
            }
        }};
    }

    #[test]
    fn from_vars_all_set() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "REDIS_URL" => "redis://custom:1234",
            "API_PORT" => "8080",
            "RESEND_API_KEY" => "re_test_key",
            "EMAIL_FROM" => "Test <test@example.com>",
        })
        .unwrap();
        assert_eq!(cfg.database_url, "postgres://localhost/test");
        assert_eq!(cfg.redis_url, "redis://custom:1234");
        assert_eq!(cfg.api_port, 8080);
        assert_eq!(cfg.resend_api_key, "re_test_key");
        assert_eq!(cfg.email_from, "Test <test@example.com>");
    }

    #[test]
    fn from_vars_missing_database_url() {
        let err = Config::from_vars(vars! {}).unwrap_err();
        assert!(
            err.to_string().contains("DATABASE_URL must be set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_defaults_redis_url() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
        })
        .unwrap();
        assert_eq!(cfg.redis_url, "redis://localhost:6379");
    }

    #[test]
    fn from_vars_defaults_api_port() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
        })
        .unwrap();
        assert_eq!(cfg.api_port, 3000);
    }

    #[test]
    fn from_vars_invalid_api_port() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "API_PORT" => "not_a_number",
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("API_PORT must be a valid u16"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_empty_database_url_is_accepted() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "",
            "RESEND_API_KEY" => "re_test",
        })
        .unwrap();
        assert_eq!(cfg.database_url, "");
    }

    #[test]
    fn from_vars_port_zero_is_valid() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "API_PORT" => "0",
        })
        .unwrap();
        assert_eq!(cfg.api_port, 0);
    }

    #[test]
    fn from_vars_port_overflow() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "API_PORT" => "99999",
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("API_PORT must be a valid u16"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_missing_resend_api_key() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("RESEND_API_KEY must be set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_defaults_email_from() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
        })
        .unwrap();
        assert_eq!(cfg.email_from, "Offrii <noreply@offrii.com>");
    }
}
