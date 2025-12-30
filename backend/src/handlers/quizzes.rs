use crate::models::{Quiz, CreateQuiz, QuizSubmissionPayload, QuizSubmissionWithAttendee};
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
            &state.q("INSERT INTO quizzes (id, codelab_id, question, quiz_type, options, correct_answer) VALUES (?, ?, ?, ?, ?, ?)")
        )
        .bind(&id)
        .bind(&codelab_id)
        .bind(&quiz.question)
        .bind(quiz.quiz_type.unwrap_or_else(|| "multiple_choice".to_string()))
        .bind(&options_json)
        .bind(quiz.correct_answer)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(StatusCode::OK)
}

pub async fn submit_quiz(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    Json(payload): Json<QuizSubmissionPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Delete previous submissions for this attendee in this codelab
    sqlx::query(&state.q("DELETE FROM quiz_submissions WHERE codelab_id = ? AND attendee_id = ?"))
        .bind(&codelab_id)
        .bind(&payload.attendee_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for sub in payload.submissions {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            &state.q("INSERT INTO quiz_submissions (id, codelab_id, attendee_id, quiz_id, answer, is_correct) VALUES (?, ?, ?, ?, ?, ?)")
        )
        .bind(&id)
        .bind(&codelab_id)
        .bind(&payload.attendee_id)
        .bind(&sub.quiz_id)
        .bind(&sub.answer)
        .bind(if sub.is_correct { 1 } else { 0 })
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    Ok(StatusCode::OK)
}

pub async fn get_quiz_submissions(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
) -> Result<Json<Vec<QuizSubmissionWithAttendee>>, (StatusCode, String)> {
    let submissions = sqlx::query_as::<_, QuizSubmissionWithAttendee>(
        &state.q("SELECT qs.*, a.name as attendee_name FROM quiz_submissions qs JOIN attendees a ON qs.attendee_id = a.id WHERE qs.codelab_id = ? ORDER BY qs.created_at DESC")
    )
    .bind(codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(submissions))
}
