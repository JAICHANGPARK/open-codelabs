use crate::infrastructure::database::AppState;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub action: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub target_id: Option<String>,
    pub codelab_id: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<Value>,
}

pub async fn record_audit(state: &AppState, entry: AuditEntry) {
    let _ = sqlx::query(
        &state.q(
            "INSERT INTO audit_logs (id, action, actor_type, actor_id, target_id, codelab_id, ip, user_agent, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ),
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(entry.action)
    .bind(entry.actor_type)
    .bind(entry.actor_id)
    .bind(entry.target_id)
    .bind(entry.codelab_id)
    .bind(entry.ip)
    .bind(entry.user_agent)
    .bind(entry.metadata.map(|value| value.to_string()))
    .execute(&state.pool)
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{AppState, DbKind};
    use serde_json::json;
    use sqlx::any::AnyPoolOptions;

    async fn setup_state(run_migrations: bool) -> AppState {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .expect("in-memory sqlite");

        if run_migrations {
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migrations");
        }

        AppState::new(
            pool,
            DbKind::Sqlite,
            "admin".to_string(),
            "pw".to_string(),
            false,
        )
    }

    #[tokio::test]
    async fn record_audit_persists_row() {
        let state = setup_state(true).await;
        record_audit(
            &state,
            AuditEntry {
                action: "test.action".to_string(),
                actor_type: "admin".to_string(),
                actor_id: Some("admin".to_string()),
                target_id: Some("target-1".to_string()),
                codelab_id: Some("lab-1".to_string()),
                ip: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
                metadata: Some(json!({ "k": "v" })),
            },
        )
        .await;

        let count: i64 = sqlx::query_scalar(&state.q("SELECT COUNT(*) FROM audit_logs"))
            .fetch_one(&state.pool)
            .await
            .expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn record_audit_ignores_insert_error() {
        let state = setup_state(false).await;
        // audit_logs table does not exist, but function should swallow the error.
        record_audit(
            &state,
            AuditEntry {
                action: "missing.table".to_string(),
                actor_type: "admin".to_string(),
                actor_id: None,
                target_id: None,
                codelab_id: None,
                ip: None,
                user_agent: None,
                metadata: None,
            },
        )
        .await;
    }
}
