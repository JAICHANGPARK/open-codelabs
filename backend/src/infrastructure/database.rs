use axum::extract::ws::Message;
use dashmap::DashMap;
use sqlx::AnyPool;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::infrastructure::AppConfig;
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
    // Map of (codelab_id, user_id) -> list of (session_id, sender) for DMs (supports multiple tabs)
    pub sessions:
        Arc<DashMap<(String, String), Vec<(String, tokio::sync::mpsc::UnboundedSender<Message>)>>>,
    // Map of codelab_id -> is_screen_sharing
    pub active_screen_shares: Arc<DashMap<String, bool>>,
    // Map of (codelab_id, attendee_id) -> is_sharing
    pub attendee_sharing: Arc<DashMap<(String, String), bool>>,
}

impl AppState {
    pub fn new(
        pool: AnyPool,
        db_kind: DbKind,
        admin_id: String,
        admin_pw: String,
        trust_proxy: bool,
    ) -> Self {
        Self {
            pool,
            db_kind,
            admin_id,
            admin_pw,
            auth: AuthConfig::from_env(),
            rate_limit_config: RateLimitConfig::from_env(),
            rate_limiter: Arc::new(RateLimiter::new()),
            security_headers: SecurityHeadersConfig::from_env(),
            trust_proxy,
            admin_api_keys: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
            active_screen_shares: Arc::new(DashMap::new()),
            attendee_sharing: Arc::new(DashMap::new()),
        }
    }

    pub fn new_with_config(pool: AnyPool, db_kind: DbKind, config: AppConfig) -> Self {
        Self::new(
            pool,
            db_kind,
            config.admin_id,
            config.admin_pw,
            config.trust_proxy,
        )
    }

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

pub fn db_kind_from_url(database_url: &str) -> DbKind {
    if database_url.starts_with("postgres") {
        DbKind::Postgres
    } else if database_url.starts_with("mysql") {
        DbKind::Mysql
    } else {
        DbKind::Sqlite
    }
}

pub fn sqlite_path_from_url(database_url: &str) -> Option<std::path::PathBuf> {
    if !database_url.starts_with("sqlite:") {
        return None;
    }

    let path = database_url.trim_start_matches("sqlite:");
    let path = path.split('?').next().unwrap_or(path);
    if path.is_empty() || path.starts_with(":memory:") || path.starts_with("::memory:") {
        return None;
    }

    Some(std::path::PathBuf::from(path))
}

pub fn ensure_sqlite_directory(database_url: &str) -> std::io::Result<()> {
    let Some(path) = sqlite_path_from_url(database_url) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::config::AppConfig;
    use sqlx::any::AnyPoolOptions;
    use tempfile::TempDir;

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

        let state_sqlite = AppState::new(
            pool.clone(),
            DbKind::Sqlite,
            "admin".to_string(),
            "pass".to_string(),
            false,
        );

        let state_postgres = AppState::new(
            pool,
            DbKind::Postgres,
            "admin".to_string(),
            "pass".to_string(),
            false,
        );

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

        let state_postgres = AppState::new(
            pool,
            DbKind::Postgres,
            "admin".to_string(),
            "pass".to_string(),
            false,
        );

        let sql = "SELECT * FROM users WHERE name = '?' AND id = ?";
        assert_eq!(
            state_postgres.q(sql),
            "SELECT * FROM users WHERE name = '?' AND id = $1"
        );
    }

    #[test]
    fn test_db_kind_from_url() {
        assert_eq!(
            db_kind_from_url("postgresql://localhost/db"),
            DbKind::Postgres
        );
        assert_eq!(db_kind_from_url("mysql://localhost/db"), DbKind::Mysql);
        assert_eq!(db_kind_from_url("sqlite:data/db.sqlite"), DbKind::Sqlite);
    }

    #[test]
    fn test_sqlite_path_from_url() {
        let path = sqlite_path_from_url("sqlite:data/sqlite.db?mode=rwc").unwrap();
        assert_eq!(path.to_string_lossy(), "data/sqlite.db");
        assert!(sqlite_path_from_url("sqlite::memory:").is_none());
        assert!(sqlite_path_from_url("sqlite:").is_none());
    }

    #[test]
    fn test_ensure_sqlite_directory_creates_parent() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let nested = temp_dir.path().join("nested/sqlite.db");
        let database_url = format!("sqlite:{}", nested.to_string_lossy());

        assert!(!nested.parent().expect("missing parent").exists());
        ensure_sqlite_directory(&database_url).expect("failed to create sqlite directory");
        assert!(nested.parent().expect("missing parent").exists());
    }

    #[tokio::test]
    async fn test_new_with_config_uses_config_values() {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let state = AppState::new_with_config(
            pool,
            DbKind::Sqlite,
            AppConfig {
                admin_id: "admin-id".to_string(),
                admin_pw: "admin-pw".to_string(),
                trust_proxy: true,
            },
        );

        assert_eq!(state.admin_id, "admin-id");
        assert_eq!(state.admin_pw, "admin-pw");
        assert!(state.trust_proxy);
    }

    #[tokio::test]
    async fn test_q_keeps_question_mark_in_escaped_quote() {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let state_postgres = AppState::new(
            pool,
            DbKind::Postgres,
            "admin".to_string(),
            "pass".to_string(),
            false,
        );

        let sql = r"SELECT 'it\\'s ?' as quoted, ?";
        assert_eq!(state_postgres.q(sql), r"SELECT 'it\\'s $1' as quoted, ?");
    }

    #[test]
    fn test_sqlite_path_from_url_non_sqlite() {
        assert!(sqlite_path_from_url("postgres://localhost/db").is_none());
        assert!(sqlite_path_from_url("sqlite::memory:?cache=shared").is_none());
    }

    #[test]
    fn test_ensure_sqlite_directory_noop_cases() {
        ensure_sqlite_directory("sqlite:db.sqlite").expect("relative file is fine");
        ensure_sqlite_directory("sqlite::memory:").expect("memory sqlite is fine");
        ensure_sqlite_directory("postgres://localhost/db").expect("non-sqlite is ignored");
        ensure_sqlite_directory("sqlite:/").expect("root path is fine");
    }
}
