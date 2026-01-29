use crate::audit::{record_audit, AuditEntry};
use crate::auth::AuthSession;
use crate::error::{bad_request, forbidden, internal_error};
use crate::models::{Codelab, CreateQuiz, Quiz, QuizSubmissionPayload, QuizSubmissionWithAttendee};
use crate::request_info::RequestInfo;
use crate::state::AppState;
use crate::validation::validate_quiz;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_quizzes(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    session: AuthSession,
) -> Result<Json<Vec<Quiz>>, (StatusCode, String)> {
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;
    if !can_access_codelab(&codelab, &session) {
        return Err(forbidden());
    }

    let quizzes = sqlx::query_as::<_, Quiz>(
        &state.q("SELECT * FROM quizzes WHERE codelab_id = ? ORDER BY created_at ASC"),
    )
    .bind(codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(quizzes))
}

pub async fn update_quizzes(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<Vec<CreateQuiz>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let admin = session.require_admin()?;
    if payload.is_empty() {
        return Err(bad_request("quizzes cannot be empty"));
    }
    // Delete existing quizzes
    sqlx::query(&state.q("DELETE FROM quizzes WHERE codelab_id = ?"))
        .bind(&codelab_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    for quiz in payload {
        validate_quiz(&quiz)?;
        let id = Uuid::new_v4().to_string();
        let options_json = serde_json::to_string(&quiz.options).map_err(internal_error)?;

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
        .map_err(internal_error)?;
    }

    record_audit(
        &state,
        AuditEntry {
            action: "quiz_update".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: None,
            codelab_id: Some(codelab_id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(StatusCode::OK)
}

pub async fn submit_quiz(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    session: AuthSession,
    Json(payload): Json<QuizSubmissionPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let attendee = session.require_attendee()?;
    if attendee.codelab_id.as_deref() != Some(codelab_id.as_str()) {
        return Err(forbidden());
    }
    if payload.submissions.is_empty() {
        return Err(bad_request("submissions cannot be empty"));
    }
    // Delete previous submissions for this attendee in this codelab
    sqlx::query(&state.q("DELETE FROM quiz_submissions WHERE codelab_id = ? AND attendee_id = ?"))
        .bind(&codelab_id)
        .bind(&attendee.sub)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    for sub in payload.submissions {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            &state.q("INSERT INTO quiz_submissions (id, codelab_id, attendee_id, quiz_id, answer, is_correct) VALUES (?, ?, ?, ?, ?, ?)")
        )
        .bind(&id)
        .bind(&codelab_id)
        .bind(&attendee.sub)
        .bind(&sub.quiz_id)
        .bind(&sub.answer)
        .bind(if sub.is_correct { 1 } else { 0 })
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    }
    Ok(StatusCode::OK)
}

pub async fn get_quiz_submissions(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    session: AuthSession,
) -> Result<Json<Vec<QuizSubmissionWithAttendee>>, (StatusCode, String)> {
    session.require_admin()?;
    let submissions = sqlx::query_as::<_, QuizSubmissionWithAttendee>(
        &state.q("SELECT qs.*, a.name as attendee_name FROM quiz_submissions qs JOIN attendees a ON qs.attendee_id = a.id WHERE qs.codelab_id = ? ORDER BY qs.created_at DESC")
    )
    .bind(codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(submissions))
}

fn can_access_codelab(codelab: &Codelab, session: &AuthSession) -> bool {
    match &session.claims {
        Some(claims) if claims.role == "admin" => true,
        Some(claims)
            if claims.role == "attendee"
                && claims.codelab_id.as_deref() == Some(codelab.id.as_str()) =>
        {
            true
        }
        _ => false,
    }
}
