use crate::domain::models::{
    AiConversation, AiMessage, AiThread, Attendee, ChatMessageRow, Codelab, Feedback, HelpRequest,
    InlineCommentMessage, InlineCommentThread, Material, Quiz, QuizSubmission, Step, Submission,
};
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::infrastructure::db_models::AuditLog;
use crate::infrastructure::database::AppState;
use crate::middleware::auth::AuthSession;
use crate::middleware::request_info::RequestInfo;
use crate::utils::error::{bad_request, internal_error};
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::Multipart;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zip;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct CodeServerWorkspace {
    pub id: String,
    pub codelab_id: String,
    pub url: String,
    pub structure_type: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BackupData {
    codelabs: Vec<Codelab>,
    steps: Vec<Step>,
    attendees: Vec<Attendee>,
    help_requests: Vec<HelpRequest>,
    chat_messages: Vec<ChatMessageRow>,
    feedback: Vec<Feedback>,
    materials: Vec<Material>,
    quizzes: Vec<Quiz>,
    quiz_submissions: Vec<QuizSubmission>,
    submissions: Vec<Submission>,
    audit_logs: Vec<AuditLog>,
    codeserver_workspaces: Vec<CodeServerWorkspace>,
    ai_conversations: Vec<AiConversation>,
    ai_threads: Vec<AiThread>,
    ai_messages: Vec<AiMessage>,
    #[serde(default)]
    inline_comment_threads: Vec<InlineCommentThread>,
    #[serde(default)]
    inline_comment_messages: Vec<InlineCommentMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BackupPayload {
    version: u32,
    created_at: String,
    data: BackupData,
}

#[derive(Debug, Serialize)]
pub struct BackupSummary {
    version: u32,
    created_at: String,
    codelabs: usize,
    steps: usize,
    attendees: usize,
    help_requests: usize,
    chat_messages: usize,
    feedback: usize,
    materials: usize,
    quizzes: usize,
    quiz_submissions: usize,
    submissions: usize,
    audit_logs: usize,
    codeserver_workspaces: usize,
    ai_conversations: usize,
    ai_threads: usize,
    ai_messages: usize,
    inline_comment_threads: usize,
    inline_comment_messages: usize,
    uploads_files: usize,
    workspaces_files: usize,
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<Cursor<&mut Vec<u8>>>,
    options: zip::write::SimpleFileOptions,
    source_dir: &Path,
    prefix: &str,
) -> Result<(), (StatusCode, String)> {
    if !source_dir.exists() {
        return Ok(());
    }

    fn walk(
        zip: &mut zip::ZipWriter<Cursor<&mut Vec<u8>>>,
        options: zip::write::SimpleFileOptions,
        base: &Path,
        dir: &Path,
        prefix: &str,
    ) -> Result<(), (StatusCode, String)> {
        for entry in fs::read_dir(dir).map_err(internal_error)? {
            let entry = entry.map_err(internal_error)?;
            let path = entry.path();
            let rel = path.strip_prefix(base).map_err(internal_error)?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let zip_path = format!("{}/{}", prefix.trim_end_matches('/'), rel_str);

            if path.is_dir() {
                let dir_name = format!("{}/", zip_path);
                zip.add_directory(dir_name, options).map_err(internal_error)?;
                walk(zip, options, base, &path, prefix)?;
            } else {
                zip.start_file(zip_path, options).map_err(internal_error)?;
                let mut file = fs::File::open(&path).map_err(internal_error)?;
                std::io::copy(&mut file, zip).map_err(internal_error)?;
            }
        }
        Ok(())
    }

    walk(zip, options, source_dir, source_dir, prefix)
}

fn is_safe_zip_path(path: &str) -> bool {
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    !path.split('/').any(|part| part == "..")
}

fn restore_dir_from_zip(
    archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    prefix: &str,
    dest_base: &Path,
) -> Result<(), (StatusCode, String)> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(internal_error)?;
        let name = file.name().to_string();
        if !name.starts_with(prefix) {
            continue;
        }
        if !is_safe_zip_path(&name) {
            return Err(bad_request("Invalid path in backup archive"));
        }

        let rel = name.trim_start_matches(prefix).trim_start_matches('/');
        if rel.is_empty() {
            continue;
        }
        let dest_path = dest_base.join(rel);

        if file.is_dir() {
            fs::create_dir_all(&dest_path).map_err(internal_error)?;
            continue;
        }

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(internal_error)?;
        }
        let mut outfile = fs::File::create(&dest_path).map_err(internal_error)?;
        std::io::copy(&mut file, &mut outfile).map_err(internal_error)?;
    }
    Ok(())
}

fn replace_dir_contents(dir: &Path) -> Result<(), (StatusCode, String)> {
    if dir.exists() {
        match fs::remove_dir_all(dir) {
            Ok(()) => {}
            Err(_) => {
                // If directory is a mount point (e.g., Docker volume), removing it can fail.
                // Fallback: delete contents but keep the directory.
                for entry in fs::read_dir(dir).map_err(internal_error)? {
                    let entry = entry.map_err(internal_error)?;
                    let path = entry.path();
                    if path.is_dir() {
                        fs::remove_dir_all(&path).map_err(internal_error)?;
                    } else {
                        fs::remove_file(&path).map_err(internal_error)?;
                    }
                }
            }
        }
    } else {
        fs::create_dir_all(dir).map_err(internal_error)?;
    }
    Ok(())
}

pub async fn export_backup(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    session.require_admin()?;

    let codelabs = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let steps = sqlx::query_as::<_, Step>(&state.q("SELECT * FROM steps"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let attendees = sqlx::query_as::<_, Attendee>(&state.q("SELECT * FROM attendees"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let help_requests = sqlx::query_as::<_, HelpRequest>(&state.q(
        "SELECT hr.id, hr.codelab_id, hr.attendee_id, COALESCE(a.name, '') AS attendee_name, hr.step_number, hr.status, hr.created_at FROM help_requests hr LEFT JOIN attendees a ON a.id = hr.attendee_id",
    ))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let chat_messages = sqlx::query_as::<_, ChatMessageRow>(&state.q("SELECT * FROM chat_messages"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let feedback = sqlx::query_as::<_, Feedback>(&state.q("SELECT * FROM feedback"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let materials = sqlx::query_as::<_, Material>(&state.q("SELECT * FROM materials"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let quizzes = sqlx::query_as::<_, Quiz>(&state.q("SELECT * FROM quizzes"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let quiz_submissions =
        sqlx::query_as::<_, QuizSubmission>(&state.q("SELECT * FROM quiz_submissions"))
            .fetch_all(&state.pool)
            .await
            .map_err(internal_error)?;
    let submissions = sqlx::query_as::<_, Submission>(&state.q(
        "SELECT id, codelab_id, attendee_id, file_path, file_name, file_size, submission_type, link_url, CAST(created_at AS TEXT) AS created_at FROM submissions",
    ))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let audit_logs = sqlx::query_as::<_, AuditLog>(&state.q("SELECT * FROM audit_logs"))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let codeserver_workspaces =
        sqlx::query_as::<_, CodeServerWorkspace>(&state.q("SELECT * FROM codeserver_workspaces"))
            .fetch_all(&state.pool)
            .await
            .map_err(internal_error)?;
    let ai_conversations = sqlx::query_as::<_, AiConversation>(&state.q(
        "SELECT id, codelab_id, user_id, user_type, user_name, step_number, question, answer, model, usage_metadata, CAST(created_at AS TEXT) AS created_at FROM ai_conversations",
    ))
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;
    let ai_threads = sqlx::query_as::<_, AiThread>(&state.q(
        "SELECT id, title, user_id, user_type, codelab_id, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at FROM ai_threads",
    ))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let ai_messages = sqlx::query_as::<_, AiMessage>(&state.q(
        "SELECT id, thread_id, role, content, grounding_metadata, usage_metadata, CAST(created_at AS TEXT) AS created_at FROM ai_messages",
    ))
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;
    let inline_comment_threads = sqlx::query_as::<_, InlineCommentThread>(&state.q(
        "SELECT id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id, CAST(created_at AS TEXT) AS created_at FROM inline_comment_threads",
    ))
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;
    let inline_comment_messages = sqlx::query_as::<_, InlineCommentMessage>(&state.q(
        "SELECT id, thread_id, codelab_id, author_role, author_id, author_name, message, CAST(created_at AS TEXT) AS created_at FROM inline_comment_messages",
    ))
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let payload = BackupPayload {
        version: 1,
        created_at: Utc::now().to_rfc3339(),
        data: BackupData {
            codelabs,
            steps,
            attendees,
            help_requests,
            chat_messages,
            feedback,
            materials,
            quizzes,
            quiz_submissions,
            submissions,
            audit_logs,
            codeserver_workspaces,
            ai_conversations,
            ai_threads,
            ai_messages,
            inline_comment_threads,
            inline_comment_messages,
        },
    };

    let mut buf = Vec::new();
    let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("backup.json", options)
        .map_err(internal_error)?;
    let metadata = serde_json::to_string_pretty(&payload).map_err(internal_error)?;
    zip.write_all(metadata.as_bytes()).map_err(internal_error)?;

    // Add uploads
    add_dir_to_zip(
        &mut zip,
        options,
        Path::new("static/uploads"),
        "uploads",
    )?;

    // Add workspaces
    let workspace_base = PathBuf::from(
        std::env::var("WORKSPACE_BASE").unwrap_or_else(|_| "/app/workspaces".to_string()),
    );
    add_dir_to_zip(&mut zip, options, &workspace_base, "workspaces")?;

    zip.finish().map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "backup_export".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(state.admin_id.clone()),
            target_id: None,
            codelab_id: None,
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("attachment; filename=\"backup_full.zip\""),
    );

    Ok((headers, buf))
}

pub async fn restore_backup(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    mut multipart: Multipart,
) -> Result<StatusCode, (StatusCode, String)> {
    session.require_admin()?;

    let mut zip_data = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e: axum_extra::extract::multipart::MultipartError| {
            bad_request(&e.to_string())
        })?
    {
        if field.name() == Some("file") {
            zip_data = field
                .bytes()
                .await
                .map_err(|_| bad_request("Failed to read backup file"))?
                .to_vec();
            if zip_data.len() > 200 * 1024 * 1024 {
                return Err(bad_request("Backup file too large"));
            }
            break;
        }
    }

    if zip_data.is_empty() {
        return Err(bad_request("No backup file provided"));
    }

    let mut archive =
        zip::ZipArchive::new(Cursor::new(zip_data)).map_err(|e| bad_request(&e.to_string()))?;

    let mut backup_json = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(internal_error)?;
        if file.name() == "backup.json" {
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(internal_error)?;
            backup_json = Some(contents);
            break;
        }
    }

    let backup_json = backup_json.ok_or_else(|| bad_request("backup.json missing"))?;
    let payload: BackupPayload = serde_json::from_str(&backup_json).map_err(internal_error)?;

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    // Clear existing data (children first)
    sqlx::query(&state.q("DELETE FROM ai_messages"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM ai_threads"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM ai_conversations"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM quiz_submissions"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM quizzes"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM submissions"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM materials"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM feedback"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM chat_messages"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM inline_comment_messages"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM inline_comment_threads"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM help_requests"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM attendees"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM steps"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM codeserver_workspaces"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM codelabs"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query(&state.q("DELETE FROM audit_logs"))
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    // Restore data
    for row in &payload.data.codelabs {
        sqlx::query(&state.q("INSERT INTO codelabs (id, title, description, author, is_public, quiz_enabled, require_quiz, require_feedback, guide_markdown, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.title)
            .bind(&row.description)
            .bind(&row.author)
            .bind(row.is_public)
            .bind(row.quiz_enabled)
            .bind(row.require_quiz)
            .bind(row.require_feedback)
            .bind(&row.guide_markdown)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.steps {
        sqlx::query(&state.q("INSERT INTO steps (id, codelab_id, step_number, title, content_markdown) VALUES (?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(row.step_number)
            .bind(&row.title)
            .bind(&row.content_markdown)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.attendees {
        sqlx::query(&state.q("INSERT INTO attendees (id, codelab_id, name, code, email, current_step, is_completed, completed_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.name)
            .bind(&row.code)
            .bind(&row.email)
            .bind(row.current_step)
            .bind(row.is_completed)
            .bind(&row.completed_at)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.help_requests {
        sqlx::query(&state.q("INSERT INTO help_requests (id, codelab_id, attendee_id, step_number, status, created_at) VALUES (?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.attendee_id)
            .bind(row.step_number)
            .bind(&row.status)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.chat_messages {
        sqlx::query(&state.q("INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type, target_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.sender_name)
            .bind(&row.message)
            .bind(&row.msg_type)
            .bind(&row.target_id)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.inline_comment_threads {
        sqlx::query(&state.q("INSERT INTO inline_comment_threads (id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.anchor_key)
            .bind(&row.target_type)
            .bind(&row.target_step_id)
            .bind(row.start_offset)
            .bind(row.end_offset)
            .bind(&row.selected_text)
            .bind(&row.content_hash)
            .bind(&row.created_by_attendee_id)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.inline_comment_messages {
        sqlx::query(&state.q("INSERT INTO inline_comment_messages (id, thread_id, codelab_id, author_role, author_id, author_name, message, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.thread_id)
            .bind(&row.codelab_id)
            .bind(&row.author_role)
            .bind(&row.author_id)
            .bind(&row.author_name)
            .bind(&row.message)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.feedback {
        sqlx::query(&state.q("INSERT INTO feedback (id, codelab_id, attendee_id, difficulty, satisfaction, comment, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.attendee_id)
            .bind(&row.difficulty)
            .bind(&row.satisfaction)
            .bind(&row.comment)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.materials {
        sqlx::query(&state.q("INSERT INTO materials (id, codelab_id, title, material_type, link_url, file_path, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.title)
            .bind(&row.material_type)
            .bind(&row.link_url)
            .bind(&row.file_path)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.quizzes {
        sqlx::query(&state.q("INSERT INTO quizzes (id, codelab_id, question, quiz_type, options, correct_answer, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.question)
            .bind(&row.quiz_type)
            .bind(&row.options)
            .bind(row.correct_answer)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.quiz_submissions {
        sqlx::query(&state.q("INSERT INTO quiz_submissions (id, codelab_id, attendee_id, quiz_id, answer, is_correct, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.attendee_id)
            .bind(&row.quiz_id)
            .bind(&row.answer)
            .bind(row.is_correct)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.submissions {
        let submission_type = row.submission_type.clone();
        sqlx::query(&state.q("INSERT INTO submissions (id, codelab_id, attendee_id, file_path, file_name, file_size, submission_type, link_url, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.attendee_id)
            .bind(&row.file_path)
            .bind(&row.file_name)
            .bind(row.file_size)
            .bind(&submission_type)
            .bind(&row.link_url)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.codeserver_workspaces {
        sqlx::query(&state.q("INSERT INTO codeserver_workspaces (id, codelab_id, url, structure_type, created_at) VALUES (?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.url)
            .bind(&row.structure_type)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.ai_conversations {
        sqlx::query(&state.q("INSERT INTO ai_conversations (id, codelab_id, user_id, user_type, user_name, step_number, question, answer, model, usage_metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.codelab_id)
            .bind(&row.user_id)
            .bind(&row.user_type)
            .bind(&row.user_name)
            .bind(&row.step_number)
            .bind(&row.question)
            .bind(&row.answer)
            .bind(&row.model)
            .bind(&row.usage_metadata)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.ai_threads {
        sqlx::query(&state.q("INSERT INTO ai_threads (id, title, user_id, user_type, codelab_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.title)
            .bind(&row.user_id)
            .bind(&row.user_type)
            .bind(&row.codelab_id)
            .bind(&row.created_at)
            .bind(&row.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.ai_messages {
        sqlx::query(&state.q("INSERT INTO ai_messages (id, thread_id, role, content, grounding_metadata, usage_metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.thread_id)
            .bind(&row.role)
            .bind(&row.content)
            .bind(&row.grounding_metadata)
            .bind(&row.usage_metadata)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    for row in &payload.data.audit_logs {
        sqlx::query(&state.q("INSERT INTO audit_logs (id, action, actor_type, actor_id, target_id, codelab_id, ip, user_agent, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(&row.id)
            .bind(&row.action)
            .bind(&row.actor_type)
            .bind(&row.actor_id)
            .bind(&row.target_id)
            .bind(&row.codelab_id)
            .bind(&row.ip)
            .bind(&row.user_agent)
            .bind(&row.metadata)
            .bind(&row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    // Restore uploads + workspaces (after DB commit)
    replace_dir_contents(Path::new("static/uploads"))?;
    let workspace_base = PathBuf::from(
        std::env::var("WORKSPACE_BASE").unwrap_or_else(|_| "/app/workspaces".to_string()),
    );
    replace_dir_contents(&workspace_base)?;

    restore_dir_from_zip(&mut archive, "uploads/", Path::new("static/uploads"))?;
    restore_dir_from_zip(&mut archive, "workspaces/", &workspace_base)?;

    record_audit(
        &state,
        AuditEntry {
            action: "backup_restore".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(state.admin_id.clone()),
            target_id: None,
            codelab_id: None,
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(StatusCode::OK)
}

pub async fn inspect_backup(
    State(_state): State<Arc<AppState>>,
    session: AuthSession,
    mut multipart: Multipart,
) -> Result<axum::Json<BackupSummary>, (StatusCode, String)> {
    session.require_admin()?;

    let mut zip_data = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e: axum_extra::extract::multipart::MultipartError| {
            bad_request(&e.to_string())
        })?
    {
        if field.name() == Some("file") {
            zip_data = field
                .bytes()
                .await
                .map_err(|_| bad_request("Failed to read backup file"))?
                .to_vec();
            if zip_data.len() > 200 * 1024 * 1024 {
                return Err(bad_request("Backup file too large"));
            }
            break;
        }
    }

    if zip_data.is_empty() {
        return Err(bad_request("No backup file provided"));
    }

    let mut archive =
        zip::ZipArchive::new(Cursor::new(zip_data)).map_err(|e| bad_request(&e.to_string()))?;

    let mut backup_json = None;
    let mut uploads_files = 0usize;
    let mut workspaces_files = 0usize;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(internal_error)?;
        let name = file.name().to_string();

        if name == "backup.json" {
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(internal_error)?;
            backup_json = Some(contents);
            continue;
        }

        if file.is_dir() {
            continue;
        }
        if name.starts_with("uploads/") {
            uploads_files += 1;
        } else if name.starts_with("workspaces/") {
            workspaces_files += 1;
        }
    }

    let backup_json = backup_json.ok_or_else(|| bad_request("backup.json missing"))?;
    let payload: BackupPayload = serde_json::from_str(&backup_json).map_err(internal_error)?;

    let summary = BackupSummary {
        version: payload.version,
        created_at: payload.created_at,
        codelabs: payload.data.codelabs.len(),
        steps: payload.data.steps.len(),
        attendees: payload.data.attendees.len(),
        help_requests: payload.data.help_requests.len(),
        chat_messages: payload.data.chat_messages.len(),
        feedback: payload.data.feedback.len(),
        materials: payload.data.materials.len(),
        quizzes: payload.data.quizzes.len(),
        quiz_submissions: payload.data.quiz_submissions.len(),
        submissions: payload.data.submissions.len(),
        audit_logs: payload.data.audit_logs.len(),
        codeserver_workspaces: payload.data.codeserver_workspaces.len(),
        ai_conversations: payload.data.ai_conversations.len(),
        ai_threads: payload.data.ai_threads.len(),
        ai_messages: payload.data.ai_messages.len(),
        inline_comment_threads: payload.data.inline_comment_threads.len(),
        inline_comment_messages: payload.data.inline_comment_messages.len(),
        uploads_files,
        workspaces_files,
    };

    Ok(axum::Json(summary))
}
