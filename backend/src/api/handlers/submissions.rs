use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::middleware::auth::AuthSession;
use crate::utils::error::{bad_request, forbidden, internal_error, unauthorized};
use crate::domain::models::{Submission, SubmissionWithAttendee};
use crate::middleware::request_info::RequestInfo;
use crate::infrastructure::database::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use axum_extra::extract::Multipart;
use image::{codecs::webp::WebPEncoder, ExtendedColorType};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

const MAX_TOTAL_SIZE: i64 = 10 * 1024 * 1024; // 10MB
const MAX_UPLOAD_SIZE: usize = 5 * 1024 * 1024; // 5MB per file

pub async fn submit_file(
    State(state): State<Arc<AppState>>,
    Path((codelab_id, attendee_id)): Path<(String, String)>,
    session: AuthSession,
    info: RequestInfo,
    mut multipart: Multipart,
) -> Result<Json<Submission>, (StatusCode, String)> {
    let attendee = session.require_attendee()?;
    if attendee.codelab_id.as_deref() != Some(codelab_id.as_str()) {
        return Err(forbidden());
    }
    if attendee.sub != attendee_id {
        return Err(forbidden());
    }
    // 1. Check total size of existing submissions for this attendee
    let row: (i64,) = sqlx::query_as(&state.q("SELECT COALESCE(SUM(file_size), 0) FROM submissions WHERE codelab_id = ? AND attendee_id = ?"))
        .bind(&codelab_id)
        .bind(&attendee_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let total_size = row.0;

    if let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let file_name = sanitize_original_name(field.file_name().unwrap_or("unnamed"));
        if file_name.is_empty() {
            return Err(bad_request("invalid file name"));
        }
        let data = field.bytes().await.map_err(internal_error)?;
        if data.len() > MAX_UPLOAD_SIZE {
            return Err(bad_request("file too large"));
        }

        // Convert images to webp to reduce size
        let (stored_bytes, stored_name, stored_ext) = match convert_image_to_webp(&file_name, &data)
        {
            Some((bytes, new_name)) => (bytes, new_name, "webp".to_string()),
            None => {
                let ext = std::path::Path::new(&file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("bin")
                    .to_string();
                (data.to_vec(), file_name.clone(), ext)
            }
        };

        let file_size = stored_bytes.len() as i64;

        if total_size + file_size > MAX_TOTAL_SIZE {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "Total submission size exceeds 10MB limit (Current: {} bytes, New: {} bytes)",
                    total_size, file_size
                ),
            ));
        }

        // Generate a unique filename to avoid collisions
        let new_filename = format!("{}.{}", Uuid::new_v4(), stored_ext);
        let upload_dir = "static/uploads/submissions";
        let file_path = format!("{}/{}", upload_dir, new_filename);

        fs::create_dir_all(upload_dir)
            .await
            .map_err(internal_error)?;

        fs::write(&file_path, &stored_bytes)
            .await
            .map_err(internal_error)?;

        let db_path = format!("/uploads/submissions/{}", new_filename);
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            &state.q("INSERT INTO submissions (id, codelab_id, attendee_id, file_path, file_name, file_size) VALUES (?, ?, ?, ?, ?, ?)")
        )
        .bind(&id)
        .bind(&codelab_id)
        .bind(&attendee_id)
        .bind(&db_path)
        .bind(&stored_name)
        .bind(&file_size)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

        let submission = Submission {
            id,
            codelab_id,
            attendee_id,
            file_path: db_path,
            file_name: stored_name,
            file_size,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };

        record_audit(
            &state,
            AuditEntry {
                action: "submission_upload".to_string(),
                actor_type: "attendee".to_string(),
                actor_id: Some(attendee.sub),
                target_id: Some(submission.id.clone()),
                codelab_id: Some(submission.codelab_id.clone()),
                ip: Some(info.ip),
                user_agent: info.user_agent,
                metadata: None,
            },
        )
        .await;

        return Ok(Json(submission));
    }

    Err(bad_request("No file uploaded"))
}

pub async fn get_submissions(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    session: AuthSession,
) -> Result<Json<Vec<SubmissionWithAttendee>>, (StatusCode, String)> {
    let claims = match session.claims {
        Some(claims) => claims,
        None => return Err(unauthorized()),
    };
    tracing::debug!("Fetching submissions for codelab: {}", codelab_id);
    let (query, bind_attendee) = if claims.role == "admin" {
        (
            r#"
            SELECT 
                s.id, s.codelab_id, s.attendee_id, COALESCE(a.name, 'Unknown') as attendee_name, 
                s.file_path, s.file_name, s.file_size, CAST(s.created_at AS TEXT) AS created_at
            FROM submissions s
            LEFT JOIN attendees a ON s.attendee_id = a.id
            WHERE s.codelab_id = ?
            ORDER BY s.created_at DESC
            "#,
            None,
        )
    } else if claims.role == "attendee" && claims.codelab_id.as_deref() == Some(codelab_id.as_str())
    {
        (
            r#"
            SELECT 
                s.id, s.codelab_id, s.attendee_id, COALESCE(a.name, 'Unknown') as attendee_name, 
                s.file_path, s.file_name, s.file_size, CAST(s.created_at AS TEXT) AS created_at
            FROM submissions s
            LEFT JOIN attendees a ON s.attendee_id = a.id
            WHERE s.codelab_id = ? AND s.attendee_id = ?
            ORDER BY s.created_at DESC
            "#,
            Some(claims.sub),
        )
    } else {
        return Err(forbidden());
    };

    let sql = state.q(query);
    let mut query = sqlx::query_as::<_, SubmissionWithAttendeeRaw>(&sql).bind(&codelab_id);
    if let Some(attendee_id) = bind_attendee {
        query = query.bind(attendee_id);
    }

    let raw = query.fetch_all(&state.pool).await.map_err(|e| {
        tracing::error!("Error fetching submissions: {}", e);
        internal_error(e)
    })?;

    let submissions: Vec<SubmissionWithAttendee> = raw
        .into_iter()
        .map(|r| SubmissionWithAttendee {
            id: r.id,
            codelab_id: r.codelab_id,
            attendee_id: r.attendee_id,
            attendee_name: r.attendee_name,
            file_path: r.file_path,
            file_name: r.file_name,
            file_size: r.file_size,
            created_at: r.created_at,
        })
        .collect();

    tracing::debug!("Found {} submissions", submissions.len());
    Ok(Json(submissions))
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct SubmissionWithAttendeeRaw {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub attendee_name: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub created_at: Option<String>,
}

fn convert_image_to_webp(original_name: &str, data: &[u8]) -> Option<(Vec<u8>, String)> {
    let img = image::load_from_memory(data).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());

    let mut out = Cursor::new(Vec::new());
    let encoder = WebPEncoder::new_lossless(&mut out);
    encoder
        .encode(&rgba, width, height, ExtendedColorType::Rgba8)
        .ok()?;

    let stem = StdPath::new(original_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("submission");
    let new_name = format!("{}.webp", stem);

    Some((out.into_inner(), new_name))
}

fn sanitize_original_name(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-' {
            out.push(ch);
        }
    }
    if out.len() > 120 {
        out.truncate(120);
    }
    out
}

pub async fn delete_submission(
    State(state): State<Arc<AppState>>,
    Path((_codelab_id, _attendee_id, submission_id)): Path<(String, String, String)>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<StatusCode, (StatusCode, String)> {
    let claims = match session.claims {
        Some(claims) => claims,
        None => return Err(unauthorized()),
    };
    // Get file path first
    let submission_row: Option<(String, String, String)> = sqlx::query_as(
        &state.q("SELECT file_path, codelab_id, attendee_id FROM submissions WHERE id = ?"),
    )
    .bind(&submission_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;
    let (file_path, submission_codelab_id, submission_attendee_id) =
        submission_row.ok_or((StatusCode::NOT_FOUND, "Submission not found".to_string()))?;

    if claims.role == "admin" {
        // allowed
    } else if claims.role == "attendee"
        && claims.sub == submission_attendee_id
        && claims.codelab_id.as_deref() == Some(submission_codelab_id.as_str())
    {
        // allowed
    } else {
        return Err(forbidden());
    }

    // Delete from DB
    sqlx::query(&state.q("DELETE FROM submissions WHERE id = ?"))
        .bind(&submission_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    // Delete from filesystem (path starts with /uploads/...)
    let relative_path = file_path.trim_start_matches('/');
    let full_path = format!("static/{}", relative_path);
    let _ = fs::remove_file(full_path).await;

    record_audit(
        &state,
        AuditEntry {
            action: "submission_delete".to_string(),
            actor_type: claims.role.clone(),
            actor_id: Some(claims.sub),
            target_id: Some(submission_id),
            codelab_id: Some(submission_codelab_id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
