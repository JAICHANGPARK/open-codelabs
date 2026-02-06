use crate::domain::models::{ChatMessageRow, Codelab, CreateCodelab, Step, UpdateStepsPayload};
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::infrastructure::database::AppState;
use crate::middleware::auth::AuthSession;
use crate::middleware::request_info::RequestInfo;
use crate::utils::error::{bad_request, forbidden, internal_error, unauthorized};
use crate::utils::validation::{validate_codelab, validate_steps};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::Multipart;
use serde_json;
use sqlx;
use std::io::{Cursor, Read, Write};
use std::sync::Arc;
use uuid;
use zip;

pub async fn list_codelabs(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<Codelab>>, (StatusCode, String)> {
    let is_admin = session
        .claims
        .as_ref()
        .map(|claims| claims.role == "admin")
        .unwrap_or(false);

    let query = if is_admin {
        "SELECT * FROM codelabs"
    } else {
        "SELECT * FROM codelabs WHERE is_public = 1"
    };

    let codelabs = sqlx::query_as::<_, Codelab>(&state.q(query))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok(Json(codelabs))
}

pub async fn get_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<(Codelab, Vec<Step>)>, (StatusCode, String)> {
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    if !can_access_codelab(&codelab, &session) {
        return Err(forbidden());
    }

    let steps = sqlx::query_as::<_, Step>(
        &state.q("SELECT * FROM steps WHERE codelab_id = ? ORDER BY step_number"),
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json((codelab, steps)))
}

pub async fn create_codelab(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CreateCodelab>,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    validate_codelab(&payload)?;
    let id = uuid::Uuid::new_v4().to_string();
    let is_public = payload.is_public.unwrap_or(true);
    let quiz_enabled = payload.quiz_enabled.unwrap_or(false);
    let require_quiz = payload.require_quiz.unwrap_or(false);
    let require_feedback = payload.require_feedback.unwrap_or(false);

    sqlx::query(&state.q("INSERT INTO codelabs (id, title, description, author, is_public, quiz_enabled, require_quiz, require_feedback, guide_markdown) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"))
        .bind(&id)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.author)
        .bind(is_public as i32)
        .bind(quiz_enabled as i32)
        .bind(require_quiz as i32)
        .bind(require_feedback as i32)
        .bind(&payload.guide_markdown)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_create".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(id.clone()),
            codelab_id: Some(id.clone()),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(codelab))
}

pub async fn copy_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    let steps = sqlx::query_as::<_, Step>(
        &state.q("SELECT * FROM steps WHERE codelab_id = ? ORDER BY step_number"),
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    let new_id = uuid::Uuid::new_v4().to_string();
    let new_title = format!("{} (Copy)", codelab.title);

    sqlx::query(&state.q("INSERT INTO codelabs (id, title, description, author, is_public, quiz_enabled, require_quiz, require_feedback, guide_markdown) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"))
        .bind(&new_id)
        .bind(&new_title)
        .bind(&codelab.description)
        .bind(&codelab.author)
        .bind(codelab.is_public)
        .bind(codelab.quiz_enabled)
        .bind(codelab.require_quiz)
        .bind(codelab.require_feedback)
        .bind(&codelab.guide_markdown)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    for step in steps {
        let step_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            &state.q("INSERT INTO steps (id, codelab_id, step_number, title, content_markdown) VALUES (?, ?, ?, ?, ?)"),
        )
        .bind(&step_id)
        .bind(&new_id)
        .bind(step.step_number)
        .bind(&step.title)
        .bind(&step.content_markdown)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    let new_codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&new_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_copy".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(new_id.clone()),
            codelab_id: Some(id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(new_codelab))
}

pub async fn update_codelab_info(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CreateCodelab>,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    validate_codelab(&payload)?;
    let is_public = payload.is_public.unwrap_or(true);
    let quiz_enabled = payload.quiz_enabled.unwrap_or(false);
    let require_quiz = payload.require_quiz.unwrap_or(false);
    let require_feedback = payload.require_feedback.unwrap_or(false);

    sqlx::query(&state.q("UPDATE codelabs SET title = ?, description = ?, author = ?, is_public = ?, quiz_enabled = ?, require_quiz = ?, require_feedback = ?, guide_markdown = ? WHERE id = ?"))
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.author)
        .bind(is_public as i32)
        .bind(quiz_enabled as i32)
        .bind(require_quiz as i32)
        .bind(require_feedback as i32)
        .bind(&payload.guide_markdown)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_update".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(id.clone()),
            codelab_id: Some(id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(codelab))
}

pub async fn update_codelab_steps(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<UpdateStepsPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    validate_steps(&payload)?;
    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    // Delete existing steps
    sqlx::query(&state.q("DELETE FROM steps WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    // Insert new steps
    for (i, step) in payload.steps.into_iter().enumerate() {
        let step_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            &state.q("INSERT INTO steps (id, codelab_id, step_number, title, content_markdown) VALUES (?, ?, ?, ?, ?)"),
        )
        .bind(&step_id)
        .bind(&id)
        .bind((i + 1) as i32)
        .bind(&step.title)
        .bind(&step.content_markdown)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_steps_update".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(id.clone()),
            codelab_id: Some(id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn export_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let admin = session.require_admin()?;
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    let steps = sqlx::query_as::<_, Step>(
        &state.q("SELECT * FROM steps WHERE codelab_id = ? ORDER BY step_number"),
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut buf = Vec::new();
    let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add metadata
    zip.start_file("codelab.json", options)
        .map_err(internal_error)?;
    let metadata = serde_json::to_string_pretty(&codelab).map_err(internal_error)?;
    zip.write_all(metadata.as_bytes()).map_err(internal_error)?;

    // Add steps
    for step in steps {
        let safe_title = sanitize_filename(&step.title);
        let filename = format!("step_{:02}_{}.md", step.step_number, safe_title);
        zip.start_file(filename, options).map_err(internal_error)?;
        zip.write_all(step.content_markdown.as_bytes())
            .map_err(internal_error)?;
    }

    // Add preparation guide (if present)
    if let Some(guide_markdown) = codelab.guide_markdown.as_deref() {
        let guide_markdown = guide_markdown.trim();
        if !guide_markdown.is_empty() {
            zip.start_file("preparation_guide.md", options)
                .map_err(internal_error)?;
            zip.write_all(guide_markdown.as_bytes())
                .map_err(internal_error)?;
        }
    }

    zip.finish().map_err(internal_error)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&format!("attachment; filename=\"codelab_{}.zip\"", id))
            .unwrap(),
    );

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_export".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(id.clone()),
            codelab_id: Some(id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok((headers, buf))
}

pub async fn import_codelab(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    mut multipart: Multipart,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    let mut zip_data = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e: axum_extra::extract::multipart::MultipartError| bad_request(&e.to_string()))?
    {
        if field.name() == Some("file") {
            zip_data = field
                .bytes()
                .await
                .map_err(|e| bad_request(&e.to_string()))?
                .to_vec();
            if zip_data.len() > 20 * 1024 * 1024 {
                return Err(bad_request("import file too large"));
            }
            break;
        }
    }

    if zip_data.is_empty() {
        return Err(bad_request("No file uploaded"));
    }

    let mut archive =
        zip::ZipArchive::new(Cursor::new(zip_data)).map_err(|e| bad_request(&e.to_string()))?;

    let mut codelab: Option<Codelab> = None;
    let mut steps_content: Vec<(i32, String, String)> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| bad_request(&e.to_string()))?;
        let name = file.name().to_string();

        if name == "codelab.json" {
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(internal_error)?;
            codelab =
                Some(serde_json::from_str(&contents).map_err(|e| bad_request(&e.to_string()))?);
        } else if name.ends_with(".md") && name.starts_with("step_") {
            // format: step_01_Title.md
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(internal_error)?;

            let parts: Vec<&str> = name.split('_').collect();
            if parts.len() >= 3 {
                let step_num: i32 = parts[1].parse().unwrap_or(0);
                let title = parts[2..].join("_").replace(".md", "").replace("_", " ");
                steps_content.push((step_num, title, contents));
            }
        }
    }

    let mut codelab = codelab.ok_or_else(|| bad_request("Missing codelab.json"))?;
    let create = CreateCodelab {
        title: codelab.title.clone(),
        description: codelab.description.clone(),
        author: codelab.author.clone(),
        is_public: Some(codelab.is_public != 0),
        quiz_enabled: Some(codelab.quiz_enabled != 0),
        require_quiz: Some(codelab.require_quiz != 0),
        require_feedback: Some(codelab.require_feedback != 0),
        guide_markdown: codelab.guide_markdown.clone(),
    };
    validate_codelab(&create)?;
    codelab.id = uuid::Uuid::new_v4().to_string(); // New ID for imported codelab

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    sqlx::query(&state.q("INSERT INTO codelabs (id, title, description, author, is_public, quiz_enabled, require_quiz, require_feedback, guide_markdown) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"))
        .bind(&codelab.id)
        .bind(&codelab.title)
        .bind(&codelab.description)
        .bind(&codelab.author)
        .bind(codelab.is_public)
        .bind(codelab.quiz_enabled)
        .bind(codelab.require_quiz)
        .bind(codelab.require_feedback)
        .bind(&codelab.guide_markdown)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    steps_content.sort_by_key(|s| s.0);

    for (step_num, title, content) in steps_content {
        if title.len() > 200 || content.len() > 50_000 {
            return Err(bad_request("step content too large"));
        }
        let step_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            &state.q("INSERT INTO steps (id, codelab_id, step_number, title, content_markdown) VALUES (?, ?, ?, ?, ?)"),
        )
        .bind(&step_id)
        .bind(&codelab.id)
        .bind(step_num)
        .bind(&title)
        .bind(&content)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_import".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(codelab.id.clone()),
            codelab_id: Some(codelab.id.clone()),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(codelab))
}

pub async fn get_chat_history(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<ChatMessageRow>>, (StatusCode, String)> {
    let claims = session.claims.ok_or_else(unauthorized)?;
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    if claims.role == "admin" {
        // allowed
    } else if claims.role == "attendee" && claims.codelab_id.as_deref() == Some(codelab.id.as_str())
    {
        // allowed
    } else {
        return Err(forbidden());
    }

    let messages = sqlx::query_as::<_, ChatMessageRow>(
        &state.q("SELECT * FROM chat_messages WHERE codelab_id = ? ORDER BY created_at ASC"),
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(messages))
}

pub async fn delete_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    tracing::debug!("Attempting to delete codelab: {}", id);
    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    // Delete steps
    sqlx::query(&state.q("DELETE FROM steps WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    // Delete chat messages
    sqlx::query(&state.q("DELETE FROM chat_messages WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    // Delete help requests
    sqlx::query(&state.q("DELETE FROM help_requests WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    // Delete attendees
    sqlx::query(&state.q("DELETE FROM attendees WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    // Delete codelab itself
    sqlx::query(&state.q("DELETE FROM codelabs WHERE id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_delete".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(id.clone()),
            codelab_id: Some(id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

fn can_access_codelab(codelab: &Codelab, session: &AuthSession) -> bool {
    if codelab.is_public != 0 {
        return true;
    }
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

fn sanitize_filename(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if ch == '-' || ch == '_' || ch.is_whitespace() {
            if !out.is_empty() && !out.ends_with('_') {
                out.push('_');
            }
        }
    }
    let res = out.trim_end_matches('_');
    if res.is_empty() {
        "step".to_string()
    } else {
        res.to_string()
    }
}

pub async fn get_reference_codelabs() -> Result<impl IntoResponse, (StatusCode, String)> {
    // Commented out as requested. Now using direct raw import in frontend.
    /*
    let path = std::path::Path::new("../docs/codelabs.csv");
    if !path.exists() {
        // Fallback for different working directories
        let alt_path = std::path::Path::new("docs/codelabs.csv");
        if !alt_path.exists() {
            return Err((StatusCode::NOT_FOUND, "Reference codelabs file not found".to_string()));
        }
        let content = std::fs::read_to_string(alt_path).map_err(|e| internal_error(e))?;
        return Ok(content);
    }

    let content = std::fs::read_to_string(path).map_err(|e| internal_error(e))?;
    Ok(content)
    */
    Ok("Reference API is temporarily disabled. Data is embedded in frontend.".to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::SessionClaims;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World"), "Hello_World");
        assert_eq!(sanitize_filename("test-file_123"), "test_file_123");
        assert_eq!(sanitize_filename("!!!"), "step");
        assert_eq!(sanitize_filename("   "), "step");
        assert_eq!(sanitize_filename("My Codelab!"), "My_Codelab");
    }

    #[test]
    fn test_can_access_codelab_public() {
        let codelab = Codelab {
            id: "123".to_string(),
            is_public: 1,
            ..Default::default()
        };
        let session = AuthSession { claims: None };
        assert!(can_access_codelab(&codelab, &session));
    }

    #[test]
    fn test_can_access_codelab_private_admin() {
        let codelab = Codelab {
            id: "123".to_string(),
            is_public: 0,
            ..Default::default()
        };
        let session = AuthSession {
            claims: Some(SessionClaims {
                sub: "admin".to_string(),
                role: "admin".to_string(),
                codelab_id: None,
                iss: "test".to_string(),
                aud: "test".to_string(),
                iat: 0,
                exp: 0,
            }),
        };
        assert!(can_access_codelab(&codelab, &session));
    }

    #[test]
    fn test_can_access_codelab_private_attendee_match() {
        let codelab = Codelab {
            id: "123".to_string(),
            is_public: 0,
            ..Default::default()
        };
        let session = AuthSession {
            claims: Some(SessionClaims {
                sub: "user".to_string(),
                role: "attendee".to_string(),
                codelab_id: Some("123".to_string()),
                iss: "test".to_string(),
                aud: "test".to_string(),
                iat: 0,
                exp: 0,
            }),
        };
        assert!(can_access_codelab(&codelab, &session));
    }

    #[test]
    fn test_can_access_codelab_private_attendee_mismatch() {
        let codelab = Codelab {
            id: "123".to_string(),
            is_public: 0,
            ..Default::default()
        };
        let session = AuthSession {
            claims: Some(SessionClaims {
                sub: "user".to_string(),
                role: "attendee".to_string(),
                codelab_id: Some("456".to_string()),
                iss: "test".to_string(),
                aud: "test".to_string(),
                iat: 0,
                exp: 0,
            }),
        };
        assert!(!can_access_codelab(&codelab, &session));
    }
}
