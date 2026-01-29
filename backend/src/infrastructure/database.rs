use axum::extract::ws::Message;
use dashmap::DashMap;
use sqlx::AnyPool;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::middleware::auth::AuthConfig;
use crate::middleware::rate_limit::{RateLimitConfig, RateLimiter};
use crate::middleware::security::SecurityHeadersConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbKind {
    Postgres,
    Mysql,
    Sqlite,
}

pub struct AppState {
    pub pool: AnyPool,
    pub db_kind: DbKind,
    pub admin_id: String,
    pub admin_pw: String,
    pub auth: AuthConfig,
    pub rate_limit_config: RateLimitConfig,
    pub rate_limiter: Arc<RateLimiter>,
    pub security_headers: SecurityHeadersConfig,
    pub trust_proxy: bool,
    // Map of session/admin -> Gemini API Key
    pub admin_api_keys: Arc<DashMap<String, String>>,
    // Map of codelab_id -> broadcast sender
    pub channels: Arc<DashMap<String, broadcast::Sender<String>>>,
    // Map of (codelab_id, attendee_id) -> sender for DMs
    pub sessions: Arc<DashMap<(String, String), tokio::sync::mpsc::UnboundedSender<Message>>>,
}

impl AppState {
    /// Fix query placeholders for different databases (e.g., ? -> $1 for Postgres)
    pub fn q(&self, sql: &str) -> String {
        if self.db_kind == DbKind::Postgres {
            let mut result = String::new();
            let mut count = 1;
            let mut in_quotes = false;
            let mut escaped = false;
            for c in sql.chars() {
                if escaped {
                    result.push(c);
                    escaped = false;
                    continue;
                }
                if c == '\\' {
                    result.push(c);
                    escaped = true;
                    continue;
                }
                if c == '\'' {
                    in_quotes = !in_quotes;
                    result.push(c);
                    continue;
                }
                if c == '?' && !in_quotes {
                    result.push_str(&format!("${}", count));
                    count += 1;
                } else {
                    result.push(c);
                }
            }
            result
        } else {
            sql.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::any::AnyPoolOptions;

    #[tokio::test]
    async fn test_q_placeholder_transformation() {
        // We don't need a real pool for this test as we only test the `q` method
        // But AppState requires one. We can use a dummy/invalid one or a mock if possible.
        // Actually, sqlx::AnyPool can be created from an in-memory sqlite connection.
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let state_sqlite = AppState {
            pool: pool.clone(),
            db_kind: DbKind::Sqlite,
            admin_id: "admin".to_string(),
            admin_pw: "pass".to_string(),
            auth: AuthConfig::from_env(),
            rate_limit_config: RateLimitConfig::from_env(),
            rate_limiter: Arc::new(RateLimiter::new()),
            security_headers: SecurityHeadersConfig::from_env(),
            trust_proxy: false,
            admin_api_keys: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        };

        let state_postgres = AppState {
            pool,
            db_kind: DbKind::Postgres,
            admin_id: "admin".to_string(),
            admin_pw: "pass".to_string(),
            auth: AuthConfig::from_env(),
            rate_limit_config: RateLimitConfig::from_env(),
            rate_limiter: Arc::new(RateLimiter::new()),
            security_headers: SecurityHeadersConfig::from_env(),
            trust_proxy: false,
            admin_api_keys: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        };

        let sql = "SELECT * FROM users WHERE id = ? AND name = ?";

        assert_eq!(
            state_sqlite.q(sql),
            "SELECT * FROM users WHERE id = ? AND name = ?"
        );
        assert_eq!(
            state_postgres.q(sql),
            "SELECT * FROM users WHERE id = $1 AND name = $2"
        );
    }

    #[tokio::test]
    async fn test_q_with_quotes() {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let state_postgres = AppState {
            pool,
            db_kind: DbKind::Postgres,
            admin_id: "admin".to_string(),
            admin_pw: "pass".to_string(),
            auth: AuthConfig::from_env(),
            rate_limit_config: RateLimitConfig::from_env(),
            rate_limiter: Arc::new(RateLimiter::new()),
            security_headers: SecurityHeadersConfig::from_env(),
            trust_proxy: false,
            admin_api_keys: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        };

        let sql = "SELECT * FROM users WHERE name = '?' AND id = ?";
        assert_eq!(
            state_postgres.q(sql),
            "SELECT * FROM users WHERE name = '?' AND id = $1"
        );
    }
}
