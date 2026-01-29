use crate::audit::{record_audit, AuditEntry};
use crate::auth::AuthSession;
use crate::error::{forbidden, internal_error};
use crate::models::{CreateFeedback, Feedback};
use crate::request_info::RequestInfo;
use crate::state::AppState;
use crate::validation::validate_feedback;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CreateFeedback>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let attendee = session.require_attendee()?;
    if attendee.codelab_id.as_deref() != Some(id.as_str()) {
        return Err(forbidden());
    }
    validate_feedback(&payload)?;
    let feedback_id = Uuid::new_v4().to_string();
    let attendee_id = attendee.sub.clone();

    let result = sqlx::query(&state.q("INSERT INTO feedback (id, codelab_id, difficulty, satisfaction, comment, attendee_id) VALUES (?, ?, ?, ?, ?, ?)"))
        .bind(&feedback_id)
        .bind(&id)
        .bind(&payload.difficulty)
        .bind(&payload.satisfaction)
        .bind(&payload.comment)
        .bind(&attendee_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => {
            record_audit(
                &state,
                AuditEntry {
                    action: "feedback_submit".to_string(),
                    actor_type: "attendee".to_string(),
                    actor_id: Some(attendee_id),
                    target_id: Some(feedback_id.clone()),
                    codelab_id: Some(id.clone()),
                    ip: Some(info.ip),
                    user_agent: info.user_agent,
                    metadata: None,
                },
            )
            .await;
            Ok(Json(serde_json::json!({ "id": feedback_id })))
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("UNIQUE constraint failed")
                || err_msg.contains("unique violation")
                || err_msg.contains("Duplicate entry")
            {
                Err((
                    StatusCode::CONFLICT,
                    "Feedback already submitted".to_string(),
                ))
            } else {
                Err(internal_error(err_msg))
            }
        }
    }
}

pub async fn get_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    session: AuthSession,
) -> Result<Json<Vec<Feedback>>, (StatusCode, String)> {
    session.require_admin()?;
    let feedback = sqlx::query_as::<_, Feedback>(
        &state.q("SELECT * FROM feedback WHERE codelab_id = ? ORDER BY created_at DESC"),
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(feedback))
}
