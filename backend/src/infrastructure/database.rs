//! Shared application state and database-specific helpers.

use axum::extract::ws::Message;
use dashmap::DashMap;
use sqlx::migrate::Migrator;
use sqlx::AnyPool;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::infrastructure::AppConfig;
use crate::middleware::auth::AuthConfig;
use crate::middleware::rate_limit::{RateLimitConfig, RateLimiter};
use crate::middleware::security::SecurityHeadersConfig;

/// Supported database backends for SQL placeholder rewriting and setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbKind {
    /// PostgreSQL-compatible databases using `$1`, `$2`, ... placeholders.
    Postgres,
    /// MySQL-compatible databases using `?` placeholders.
    Mysql,
    /// SQLite databases using `?` placeholders.
    Sqlite,
}

static SQLITE_MIGRATOR: Migrator = sqlx::migrate!("./migrations");
static POSTGRES_MIGRATOR: Migrator = sqlx::migrate!("./migrations-postgres");

/// Shared runtime state stored in Axum and accessed by handlers/middleware.
pub struct AppState {
    /// Shared SQLx connection pool for all database access.
    pub pool: AnyPool,
    /// Active database backend for query placeholder translation.
    pub db_kind: DbKind,
    /// Configured admin identifier for built-in authentication.
    pub admin_id: String,
    /// Configured admin password for built-in authentication.
    pub admin_pw: String,
    /// Session and cookie configuration used by auth helpers.
    pub auth: AuthConfig,
    /// Tunable per-bucket rate-limit settings.
    pub rate_limit_config: RateLimitConfig,
    /// In-memory sliding-window limiter shared by incoming requests.
    pub rate_limiter: Arc<RateLimiter>,
    /// Security header values applied by response middleware.
    pub security_headers: SecurityHeadersConfig,
    /// Whether proxy forwarding headers are trusted when deriving request info.
    pub trust_proxy: bool,
    /// Decrypted API keys keyed by session or admin identifier.
    pub admin_api_keys: Arc<DashMap<String, String>>,
    /// Broadcast channels keyed by codelab id for room-wide websocket events.
    pub channels: Arc<DashMap<String, broadcast::Sender<String>>>,
    /// Per-user websocket senders keyed by `(codelab_id, user_id)`.
    ///
    /// Each value stores one sender per browser tab so direct messages can be
    /// fanned out to every active session for the same user.
    pub sessions:
        Arc<DashMap<(String, String), Vec<(String, tokio::sync::mpsc::UnboundedSender<Message>)>>>,
    /// Whether any attendee is actively screen-sharing in a codelab.
    pub active_screen_shares: Arc<DashMap<String, bool>>,
    /// Whether a specific attendee is actively sharing their screen.
    pub attendee_sharing: Arc<DashMap<(String, String), bool>>,
}

impl AppState {
    /// Creates the full shared application state from explicit runtime values.
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

    /// Creates [`AppState`] from a connection pool and [`AppConfig`].
    pub fn new_with_config(pool: AnyPool, db_kind: DbKind, config: AppConfig) -> Self {
        Self::new(
            pool,
            db_kind,
            config.admin_id,
            config.admin_pw,
            config.trust_proxy,
        )
    }

    /// Rewrites generic `?` placeholders for the active database backend.
    ///
    /// PostgreSQL requires numbered placeholders such as `$1` and `$2`, while
    /// SQLite and MySQL accept raw `?`. Quoted string literals are left intact.
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

/// Determines the database backend from a SQLx Any connection URL.
pub fn db_kind_from_url(database_url: &str) -> DbKind {
    if database_url.starts_with("postgres") {
        DbKind::Postgres
    } else if database_url.starts_with("mysql") {
        DbKind::Mysql
    } else {
        DbKind::Sqlite
    }
}

/// Extracts the filesystem path for a file-backed SQLite connection string.
///
/// In-memory SQLite URLs return `None`.
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

/// Ensures the parent directory exists for a file-backed SQLite database URL.
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

/// Runs the migration set for the active database backend.
pub async fn run_migrations(
    pool: &AnyPool,
    db_kind: DbKind,
) -> Result<(), sqlx::migrate::MigrateError> {
    match db_kind {
        DbKind::Postgres => POSTGRES_MIGRATOR.run(pool).await,
        DbKind::Sqlite => SQLITE_MIGRATOR.run(pool).await,
        DbKind::Mysql => unreachable!("MySQL is rejected during startup"),
    }
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
