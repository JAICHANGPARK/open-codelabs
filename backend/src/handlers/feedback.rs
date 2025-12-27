use crate::models::{CreateFeedback, Feedback};
use crate::state::AppState;
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
    Json(payload): Json<CreateFeedback>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let feedback_id = Uuid::new_v4().to_string();

    let result = sqlx::query(&state.q("INSERT INTO feedback (id, codelab_id, difficulty, satisfaction, comment, attendee_id) VALUES (?, ?, ?, ?, ?, ?)"))
        .bind(&feedback_id)
        .bind(&id)
        .bind(&payload.difficulty)
        .bind(&payload.satisfaction)
        .bind(&payload.comment)
        .bind(&payload.attendee_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => Ok(Json(serde_json::json!({ "id": feedback_id }))),
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("UNIQUE constraint failed") || err_msg.contains("unique violation") || err_msg.contains("Duplicate entry") {
                Err((
                    StatusCode::CONFLICT,
                    "Feedback already submitted".to_string(),
                ))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, err_msg))
            }
        }
    }
}

pub async fn get_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Feedback>>, (StatusCode, String)> {
    let feedback = sqlx::query_as::<_, Feedback>(
        &state.q("SELECT * FROM feedback WHERE codelab_id = ? ORDER BY created_at DESC"),
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(feedback))
}
