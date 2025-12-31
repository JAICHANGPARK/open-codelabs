use crate::models::{Submission, SubmissionWithAttendee};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use axum_extra::extract::Multipart;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

const MAX_TOTAL_SIZE: i64 = 10 * 1024 * 1024; // 10MB

pub async fn submit_file(
    State(state): State<Arc<AppState>>,
    Path((codelab_id, attendee_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<Submission>, (StatusCode, String)> {
    // 1. Check total size of existing submissions for this attendee
    let row: (i64,) = sqlx::query_as(&state.q("SELECT COALESCE(SUM(file_size), 0) FROM submissions WHERE codelab_id = ? AND attendee_id = ?"))
        .bind(&codelab_id)
        .bind(&attendee_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_size = row.0;

    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let file_size = data.len() as i64;

        if total_size + file_size > MAX_TOTAL_SIZE {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Total submission size exceeds 10MB limit (Current: {} bytes, New: {} bytes)", total_size, file_size),
            ));
        }

        // Generate a unique filename to avoid collisions
        let file_ext = std::path::Path::new(&file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");
        let new_filename = format!("{}.{}", Uuid::new_v4(), file_ext);
        let upload_dir = "static/uploads/submissions";
        let file_path = format!("{}/{}", upload_dir, new_filename);

        fs::create_dir_all(upload_dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        fs::write(&file_path, data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let db_path = format!("/uploads/submissions/{}", new_filename);
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            &state.q("INSERT INTO submissions (id, codelab_id, attendee_id, file_path, file_name, file_size) VALUES (?, ?, ?, ?, ?, ?)")
        )
        .bind(&id)
        .bind(&codelab_id)
        .bind(&attendee_id)
        .bind(&db_path)
        .bind(&file_name)
        .bind(&file_size)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let submission = Submission {
            id,
            codelab_id,
            attendee_id,
            file_path: db_path,
            file_name,
            file_size,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };

        return Ok(Json(submission));
    }

    Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))
}

pub async fn get_submissions(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
) -> Result<Json<Vec<SubmissionWithAttendee>>, (StatusCode, String)> {
    tracing::debug!("Fetching submissions for codelab: {}", codelab_id);
    let submissions = sqlx::query_as::<_, SubmissionWithAttendee>(
        &state.q(r#"
        SELECT 
            s.id, s.codelab_id, s.attendee_id, COALESCE(a.name, 'Unknown') as attendee_name, 
            s.file_path, s.file_name, s.file_size, s.created_at
        FROM submissions s
        LEFT JOIN attendees a ON s.attendee_id = a.id
        WHERE s.codelab_id = ?
        ORDER BY s.created_at DESC
        "#)
    )
    .bind(&codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching submissions: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    tracing::debug!("Found {} submissions", submissions.len());
    Ok(Json(submissions))
}

pub async fn delete_submission(
    State(state): State<Arc<AppState>>,
    Path((_codelab_id, _attendee_id, submission_id)): Path<(String, String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Get file path first
    let file_path: (String,) = sqlx::query_as(&state.q("SELECT file_path FROM submissions WHERE id = ?"))
        .bind(&submission_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete from DB
    sqlx::query(&state.q("DELETE FROM submissions WHERE id = ?"))
        .bind(&submission_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete from filesystem (path starts with /uploads/...)
    let relative_path = file_path.0.trim_start_matches('/');
    let full_path = format!("static/{}", relative_path);
    let _ = fs::remove_file(full_path).await;

    Ok(StatusCode::NO_CONTENT)
}