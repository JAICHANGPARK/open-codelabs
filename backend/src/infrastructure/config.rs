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
