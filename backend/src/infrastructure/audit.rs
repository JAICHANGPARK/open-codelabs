use crate::state::AppState;
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
