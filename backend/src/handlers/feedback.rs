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

    sqlx::query("INSERT INTO feedback (id, codelab_id, difficulty, satisfaction, comment) VALUES (?, ?, ?, ?, ?)")
        .bind(&feedback_id)
        .bind(&id)
        .bind(&payload.difficulty)
        .bind(&payload.satisfaction)
        .bind(&payload.comment)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "id": feedback_id })))
}

pub async fn get_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Feedback>>, (StatusCode, String)> {
    let feedback = sqlx::query_as::<_, Feedback>(
        "SELECT * FROM feedback WHERE codelab_id = ? ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(feedback))
}
