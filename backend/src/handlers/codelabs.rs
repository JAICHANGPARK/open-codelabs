use crate::models::{ChatMessageRow, Codelab, CreateCodelab, Step, UpdateStepsPayload};
use crate::state::AppState;
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
    headers: HeaderMap,
) -> Result<Json<Vec<Codelab>>, (StatusCode, String)> {
    let is_admin = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s == "Bearer mock-jwt-token" || s == "mock-jwt-token")
        .unwrap_or(false);

    let query = if is_admin {
        "SELECT * FROM codelabs"
    } else {
        "SELECT * FROM codelabs WHERE is_public = 1"
    };

    let codelabs = sqlx::query_as::<_, Codelab>(&state.q(query))
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codelabs))
}

pub async fn get_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<(Codelab, Vec<Step>)>, (StatusCode, String)> {
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    let is_admin = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s == "Bearer mock-jwt-token" || s == "mock-jwt-token")
        .unwrap_or(false);

    if codelab.is_public == 0 && !is_admin {
        return Err((StatusCode::FORBIDDEN, "This codelab is private".to_string()));
    }

    let steps =
        sqlx::query_as::<_, Step>(&state.q("SELECT * FROM steps WHERE codelab_id = ? ORDER BY step_number"))
            .bind(&id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json((codelab, steps)))
}

pub async fn create_codelab(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCodelab>,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();
    let is_public = payload.is_public.unwrap_or(true);
    let quiz_enabled = payload.quiz_enabled.unwrap_or(false);
    let require_quiz = payload.require_quiz.unwrap_or(false);
    let require_feedback = payload.require_feedback.unwrap_or(false);

    sqlx::query(&state.q("INSERT INTO codelabs (id, title, description, author, is_public, quiz_enabled, require_quiz, require_feedback) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"))
        .bind(&id)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.author)
        .bind(is_public as i32)
        .bind(quiz_enabled as i32)
        .bind(require_quiz as i32)
        .bind(require_feedback as i32)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codelab))
}

pub async fn update_codelab_info(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCodelab>,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let is_public = payload.is_public.unwrap_or(true);
    let quiz_enabled = payload.quiz_enabled.unwrap_or(false);
    let require_quiz = payload.require_quiz.unwrap_or(false);
    let require_feedback = payload.require_feedback.unwrap_or(false);

    sqlx::query(&state.q("UPDATE codelabs SET title = ?, description = ?, author = ?, is_public = ?, quiz_enabled = ?, require_quiz = ?, require_feedback = ? WHERE id = ?"))
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.author)
        .bind(is_public as i32)
        .bind(quiz_enabled as i32)
        .bind(require_quiz as i32)
        .bind(require_feedback as i32)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codelab))
}

pub async fn update_codelab_steps(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateStepsPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete existing steps
    sqlx::query(&state.q("DELETE FROM steps WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn export_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    let steps =
        sqlx::query_as::<_, Step>(&state.q("SELECT * FROM steps WHERE codelab_id = ? ORDER BY step_number"))
            .bind(&id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut buf = Vec::new();
    let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add metadata
    zip.start_file("codelab.json", options)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let metadata = serde_json::to_string_pretty(&codelab)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    zip.write_all(metadata.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Add steps
    for step in steps {
        let filename = format!(
            "step_{:02}_{}.md",
            step.step_number,
            step.title.replace(" ", "_")
        );
        zip.start_file(filename, options)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        zip.write_all(step.content_markdown.as_bytes())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    zip.finish()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

    Ok((headers, buf))
}

pub async fn import_codelab(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let mut zip_data = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(
        |e: axum_extra::extract::multipart::MultipartError| {
            (StatusCode::BAD_REQUEST, e.to_string())
        },
    )? {
        if field.name() == Some("file") {
            zip_data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                .to_vec();
            break;
        }
    }

    if zip_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()));
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(zip_data))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut codelab: Option<Codelab> = None;
    let mut steps_content: Vec<(i32, String, String)> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let name = file.name().to_string();

        if name == "codelab.json" {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            codelab = Some(
                serde_json::from_str(&contents)
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
            );
        } else if name.ends_with(".md") && name.starts_with("step_") {
            // format: step_01_Title.md
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let parts: Vec<&str> = name.split('_').collect();
            if parts.len() >= 3 {
                let step_num: i32 = parts[1].parse().unwrap_or(0);
                let title = parts[2..].join("_").replace(".md", "").replace("_", " ");
                steps_content.push((step_num, title, contents));
            }
        }
    }

    let mut codelab =
        codelab.ok_or((StatusCode::BAD_REQUEST, "Missing codelab.json".to_string()))?;
    codelab.id = uuid::Uuid::new_v4().to_string(); // New ID for imported codelab

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(&state.q("INSERT INTO codelabs (id, title, description, author, is_public, quiz_enabled, require_quiz, require_feedback) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"))
        .bind(&codelab.id)
        .bind(&codelab.title)
        .bind(&codelab.description)
        .bind(&codelab.author)
        .bind(codelab.is_public)
        .bind(codelab.quiz_enabled)
        .bind(codelab.require_quiz)
        .bind(codelab.require_feedback)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    steps_content.sort_by_key(|s| s.0);

    for (step_num, title, content) in steps_content {
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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codelab))
}

pub async fn get_chat_history(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ChatMessageRow>>, (StatusCode, String)> {
    let messages = sqlx::query_as::<_, ChatMessageRow>(
        &state.q("SELECT * FROM chat_messages WHERE codelab_id = ? ORDER BY created_at ASC"),
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

pub async fn delete_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::debug!("Attempting to delete codelab: {}", id);
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete steps
    sqlx::query(&state.q("DELETE FROM steps WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete chat messages
    sqlx::query(&state.q("DELETE FROM chat_messages WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete help requests
    sqlx::query(&state.q("DELETE FROM help_requests WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete attendees
    sqlx::query(&state.q("DELETE FROM attendees WHERE codelab_id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete codelab itself
    sqlx::query(&state.q("DELETE FROM codelabs WHERE id = ?"))
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
