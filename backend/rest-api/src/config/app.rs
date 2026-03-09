use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub api_port: u16,
    pub resend_api_key: String,
    pub email_from: String,
    pub apns_key_path: String,
    pub apns_key_id: String,
    pub apns_team_id: String,
    pub apns_bundle_id: String,
    pub apns_sandbox: bool,
    pub app_base_url: String,
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

        let apns_key_path =
            var("APNS_KEY_PATH").map_err(|_| anyhow::anyhow!("APNS_KEY_PATH must be set"))?;
        let apns_key_id =
            var("APNS_KEY_ID").map_err(|_| anyhow::anyhow!("APNS_KEY_ID must be set"))?;
        let apns_team_id =
            var("APNS_TEAM_ID").map_err(|_| anyhow::anyhow!("APNS_TEAM_ID must be set"))?;
        let apns_bundle_id =
            var("APNS_BUNDLE_ID").map_err(|_| anyhow::anyhow!("APNS_BUNDLE_ID must be set"))?;
        let apns_sandbox = var("APNS_SANDBOX")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .map_err(|_| anyhow::anyhow!("APNS_SANDBOX must be true or false"))?;

        let app_base_url =
            var("APP_BASE_URL").unwrap_or_else(|_| format!("http://localhost:{api_port}"));

        Ok(Self {
            database_url,
            redis_url,
            api_port,
            resend_api_key,
            email_from,
            apns_key_path,
            apns_key_id,
            apns_team_id,
            apns_bundle_id,
            apns_sandbox,
            app_base_url,
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
            "APNS_KEY_PATH" => "./AuthKey_TEST.p8",
            "APNS_KEY_ID" => "TESTKEY123",
            "APNS_TEAM_ID" => "TESTTEAM99",
            "APNS_BUNDLE_ID" => "com.offrii.test",
            "APNS_SANDBOX" => "true",
        })
        .unwrap();
        assert_eq!(cfg.database_url, "postgres://localhost/test");
        assert_eq!(cfg.redis_url, "redis://custom:1234");
        assert_eq!(cfg.api_port, 8080);
        assert_eq!(cfg.resend_api_key, "re_test_key");
        assert_eq!(cfg.email_from, "Test <test@example.com>");
        assert_eq!(cfg.apns_key_path, "./AuthKey_TEST.p8");
        assert_eq!(cfg.apns_key_id, "TESTKEY123");
        assert_eq!(cfg.apns_team_id, "TESTTEAM99");
        assert_eq!(cfg.apns_bundle_id, "com.offrii.test");
        assert!(cfg.apns_sandbox);
        assert_eq!(cfg.app_base_url, "http://localhost:8080");
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
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
            "APNS_BUNDLE_ID" => "com.test",
        })
        .unwrap();
        assert_eq!(cfg.redis_url, "redis://localhost:6379");
    }

    #[test]
    fn from_vars_defaults_api_port() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
            "APNS_BUNDLE_ID" => "com.test",
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
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
            "APNS_BUNDLE_ID" => "com.test",
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
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
            "APNS_BUNDLE_ID" => "com.test",
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
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
            "APNS_BUNDLE_ID" => "com.test",
        })
        .unwrap();
        assert_eq!(cfg.email_from, "Offrii <noreply@offrii.com>");
    }

    #[test]
    fn from_vars_missing_apns_key_path() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("APNS_KEY_PATH must be set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_missing_apns_key_id() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "APNS_KEY_PATH" => "./key.p8",
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("APNS_KEY_ID must be set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_missing_apns_team_id() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("APNS_TEAM_ID must be set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_missing_apns_bundle_id() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
        })
        .unwrap_err();
        assert!(
            err.to_string().contains("APNS_BUNDLE_ID must be set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_vars_defaults_apns_sandbox() {
        let cfg = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
            "APNS_BUNDLE_ID" => "com.test",
        })
        .unwrap();
        assert!(!cfg.apns_sandbox);
    }

    #[test]
    fn from_vars_invalid_apns_sandbox() {
        let err = Config::from_vars(vars! {
            "DATABASE_URL" => "postgres://localhost/test",
            "RESEND_API_KEY" => "re_test",
            "APNS_KEY_PATH" => "./key.p8",
            "APNS_KEY_ID" => "K1",
            "APNS_TEAM_ID" => "T1",
            "APNS_BUNDLE_ID" => "com.test",
            "APNS_SANDBOX" => "maybe",
        })
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("APNS_SANDBOX must be true or false"),
            "unexpected error: {err}"
        );
    }
}
