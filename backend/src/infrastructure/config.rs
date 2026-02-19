use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub admin_id: String,
    pub admin_pw: String,
    pub trust_proxy: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let admin_id = std::env::var("ADMIN_ID").context("ADMIN_ID must be set")?;
        let admin_pw = std::env::var("ADMIN_PW").context("ADMIN_PW must be set")?;
        let trust_proxy = std::env::var("TRUST_PROXY")
            .ok()
            .map(|value| value == "true")
            .unwrap_or(false);

        Ok(Self {
            admin_id,
            admin_pw,
            trust_proxy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    struct EnvRestore {
        key: &'static str,
        value: Option<String>,
    }

    impl EnvRestore {
        fn new(key: &'static str) -> Self {
            Self {
                key,
                value: std::env::var(key).ok(),
            }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            if let Some(value) = &self.value {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    static ENV_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn from_env_reads_values() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _id_guard = EnvRestore::new("ADMIN_ID");
        let _pw_guard = EnvRestore::new("ADMIN_PW");
        let _proxy_guard = EnvRestore::new("TRUST_PROXY");

        std::env::set_var("ADMIN_ID", "admin");
        std::env::set_var("ADMIN_PW", "pw");
        std::env::set_var("TRUST_PROXY", "true");

        let cfg = AppConfig::from_env().expect("config");
        assert_eq!(cfg.admin_id, "admin");
        assert_eq!(cfg.admin_pw, "pw");
        assert!(cfg.trust_proxy);
    }

    #[test]
    fn from_env_requires_admin_credentials() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _id_guard = EnvRestore::new("ADMIN_ID");
        let _pw_guard = EnvRestore::new("ADMIN_PW");
        std::env::remove_var("ADMIN_ID");
        std::env::remove_var("ADMIN_PW");

        let err = AppConfig::from_env().expect_err("must fail");
        let message = err.to_string();
        let has_id = message.contains("ADMIN_ID must be set");
        let has_pw = message.contains("ADMIN_PW must be set");
        assert!(has_id || has_pw);
    }

    #[test]
    fn from_env_requires_admin_pw_when_id_exists() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _id_guard = EnvRestore::new("ADMIN_ID");
        let _pw_guard = EnvRestore::new("ADMIN_PW");
        std::env::set_var("ADMIN_ID", "admin");
        std::env::remove_var("ADMIN_PW");

        let err = AppConfig::from_env().expect_err("must fail");
        assert!(err.to_string().contains("ADMIN_PW must be set"));
    }

    #[test]
    fn env_restore_restores_previous_value() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _restore = EnvRestore::new("ADMIN_ID");
        std::env::set_var("ADMIN_ID", "original");
        {
            let _guard = EnvRestore::new("ADMIN_ID");
            std::env::set_var("ADMIN_ID", "changed");
        }
        assert_eq!(std::env::var("ADMIN_ID").as_deref(), Ok("original"));
    }
}
