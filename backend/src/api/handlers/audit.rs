use crate::infrastructure::database::AppState;
use crate::infrastructure::db_models::AuditLog;
use crate::middleware::auth::AuthSession;
use crate::utils::error::internal_error;
use crate::api::dto::AuditLogQuery;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Query(params): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLog>>, (StatusCode, String)> {
    session.require_admin()?;

    let limit = params.limit.unwrap_or(100).min(1000);
    let offset = params.offset.unwrap_or(0);

    let logs = if let Some(codelab_id) = params.codelab_id {
        if let Some(action) = params.action {
            sqlx::query_as::<_, AuditLog>(&state.q(
                "SELECT * FROM audit_logs WHERE codelab_id = ? AND action = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            ))
            .bind(codelab_id)
            .bind(action)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
            .map_err(internal_error)?
        } else {
            sqlx::query_as::<_, AuditLog>(&state.q(
                "SELECT * FROM audit_logs WHERE codelab_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            ))
            .bind(codelab_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
            .map_err(internal_error)?
        }
    } else if let Some(action) = params.action {
        sqlx::query_as::<_, AuditLog>(&state.q(
            "SELECT * FROM audit_logs WHERE action = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        ))
        .bind(action)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    } else {
        sqlx::query_as::<_, AuditLog>(&state.q(
            "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT ? OFFSET ?"
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
    };

    Ok(Json(logs))
}
