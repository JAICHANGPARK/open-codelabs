use crate::models::{Quiz, CreateQuiz};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;
use serde_json;

pub async fn get_quizzes(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
) -> Result<Json<Vec<Quiz>>, (StatusCode, String)> {
    let quizzes = sqlx::query_as::<_, Quiz>(
        &state.q("SELECT * FROM quizzes WHERE codelab_id = ? ORDER BY created_at ASC")
    )
    .bind(codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(quizzes))
}

pub async fn update_quizzes(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    Json(payload): Json<Vec<CreateQuiz>>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Delete existing quizzes
    sqlx::query(&state.q("DELETE FROM quizzes WHERE codelab_id = ?"))
        .bind(&codelab_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for quiz in payload {
        let id = Uuid::new_v4().to_string();
        let options_json = serde_json::to_string(&quiz.options)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query(
            &state.q("INSERT INTO quizzes (id, codelab_id, question, options, correct_answer) VALUES (?, ?, ?, ?, ?)")
        )
        .bind(&id)
        .bind(&codelab_id)
        .bind(&quiz.question)
        .bind(&options_json)
        .bind(quiz.correct_answer)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(StatusCode::OK)
}
