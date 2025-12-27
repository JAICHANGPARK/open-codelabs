use axum::extract::ws::Message;
use dashmap::DashMap;
use sqlx::AnyPool;
use std::sync::Arc;
use tokio::sync::broadcast;

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
