use crate::domain::models::{
    InlineCommentMessage, InlineCommentThread, InlineCommentThreadWithMessages,
    CreateInlineCommentPayload, ReplyInlineCommentPayload,
};
use crate::infrastructure::database::AppState;
use crate::middleware::auth::AuthSession;
use crate::utils::error::{bad_request, forbidden, internal_error};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct InlineCommentQuery {
    pub target_type: Option<String>,
    pub target_step_id: Option<String>,
}

#[derive(Debug)]
struct Actor {
    role: String, // "admin" | "attendee"
    id: String,
    name: String,
    is_admin: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct MessageDeleteRow {
    codelab_id: String,
    author_id: String,
    target_type: String,
    target_step_id: Option<String>,
}

pub async fn get_inline_comments(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<InlineCommentQuery>,
    session: AuthSession,
) -> Result<Json<Vec<InlineCommentThreadWithMessages>>, (StatusCode, String)> {
    let _actor = authorize_codelab_actor(&state, &session, &id).await?;

    if let Some(target_type) = query.target_type.as_ref() {
        validate_target_type(target_type)?;
    }

    let mut threads = sqlx::query_as::<_, InlineCommentThread>(&state.q(
        "SELECT id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id, CAST(created_at AS TEXT) as created_at
         FROM inline_comment_threads
         WHERE codelab_id = ?
         ORDER BY created_at ASC",
    ))
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    if let Some(target_type) = query.target_type {
        threads.retain(|thread| thread.target_type == target_type);
    }
    if let Some(target_step_id) = query.target_step_id {
        threads.retain(|thread| thread.target_step_id.as_deref() == Some(target_step_id.as_str()));
    }

    let messages = sqlx::query_as::<_, InlineCommentMessage>(&state.q(
        "SELECT id, thread_id, codelab_id, author_role, author_id, author_name, message, CAST(created_at AS TEXT) as created_at
         FROM inline_comment_messages
         WHERE codelab_id = ?
         ORDER BY created_at ASC",
    ))
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut grouped: HashMap<String, Vec<InlineCommentMessage>> = HashMap::new();
    for message in messages {
        grouped
            .entry(message.thread_id.clone())
            .or_default()
            .push(message);
    }

    let result = threads
        .into_iter()
        .map(|thread| InlineCommentThreadWithMessages {
            messages: grouped.remove(&thread.id).unwrap_or_default(),
            thread,
        })
        .collect::<Vec<_>>();

    Ok(Json(result))
}

pub async fn create_inline_comment(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Json(payload): Json<CreateInlineCommentPayload>,
) -> Result<Json<InlineCommentThreadWithMessages>, (StatusCode, String)> {
    let actor = authorize_codelab_actor(&state, &session, &id).await?;
    validate_create_payload(&payload)?;
    validate_target_integrity(&state, &id, &payload.target_type, payload.target_step_id.as_deref())
        .await?;

    let existing_thread = sqlx::query_as::<_, InlineCommentThread>(&state.q(
        "SELECT id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id, CAST(created_at AS TEXT) as created_at
         FROM inline_comment_threads
         WHERE codelab_id = ? AND anchor_key = ?
         LIMIT 1",
    ))
    .bind(&id)
    .bind(&payload.anchor_key)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    let thread_id = if let Some(thread) = existing_thread {
        if thread.target_type != payload.target_type
            || thread.target_step_id != payload.target_step_id
            || thread.content_hash != payload.content_hash
            || thread.start_offset != payload.start_offset
            || thread.end_offset != payload.end_offset
        {
            return Err(bad_request("anchor_key conflict"));
        }
        thread.id
    } else {
        // Overlap rule: forbid creating a different highlight over an existing highlight range.
        let candidate_threads = sqlx::query_as::<_, InlineCommentThread>(&state.q(
            "SELECT id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id, CAST(created_at AS TEXT) as created_at
             FROM inline_comment_threads
             WHERE codelab_id = ? AND target_type = ? AND content_hash = ?",
        ))
        .bind(&id)
        .bind(&payload.target_type)
        .bind(&payload.content_hash)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;

        let overlaps = candidate_threads.iter().any(|thread| {
            thread.target_step_id == payload.target_step_id
                && thread.start_offset < payload.end_offset
                && thread.end_offset > payload.start_offset
        });
        if overlaps {
            return Err(bad_request(
                "overlapping highlight exists. Please reply on the existing highlight.",
            ));
        }

        let new_thread_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(&state.q(
            "INSERT INTO inline_comment_threads
             (id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ))
        .bind(&new_thread_id)
        .bind(&id)
        .bind(&payload.anchor_key)
        .bind(&payload.target_type)
        .bind(&payload.target_step_id)
        .bind(payload.start_offset)
        .bind(payload.end_offset)
        .bind(payload.selected_text.trim())
        .bind(payload.content_hash.trim())
        .bind(&actor.id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

        new_thread_id
    };

    let message_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(&state.q(
        "INSERT INTO inline_comment_messages
         (id, thread_id, codelab_id, author_role, author_id, author_name, message)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    ))
    .bind(&message_id)
    .bind(&thread_id)
    .bind(&id)
    .bind(&actor.role)
    .bind(&actor.id)
    .bind(&actor.name)
    .bind(payload.message.trim())
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let thread = fetch_thread_with_messages(&state, &id, &thread_id).await?;
    broadcast_inline_comment_changed(
        &state,
        &id,
        &thread.thread.target_type,
        thread.thread.target_step_id.as_deref(),
    );

    Ok(Json(thread))
}

pub async fn reply_inline_comment(
    Path((id, thread_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Json(payload): Json<ReplyInlineCommentPayload>,
) -> Result<Json<InlineCommentThreadWithMessages>, (StatusCode, String)> {
    let actor = authorize_codelab_actor(&state, &session, &id).await?;
    validate_reply_payload(&payload)?;

    let thread = sqlx::query_as::<_, InlineCommentThread>(&state.q(
        "SELECT id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id, CAST(created_at AS TEXT) as created_at
         FROM inline_comment_threads
         WHERE codelab_id = ? AND id = ?
         LIMIT 1",
    ))
    .bind(&id)
    .bind(&thread_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::NOT_FOUND, "Inline comment thread not found".to_string()))?;

    if thread.content_hash != payload.content_hash.trim() {
        return Err(bad_request("stale thread cannot be replied to"));
    }

    let message_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(&state.q(
        "INSERT INTO inline_comment_messages
         (id, thread_id, codelab_id, author_role, author_id, author_name, message)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    ))
    .bind(&message_id)
    .bind(&thread_id)
    .bind(&id)
    .bind(&actor.role)
    .bind(&actor.id)
    .bind(&actor.name)
    .bind(payload.message.trim())
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let thread = fetch_thread_with_messages(&state, &id, &thread_id).await?;
    broadcast_inline_comment_changed(
        &state,
        &id,
        &thread.thread.target_type,
        thread.thread.target_step_id.as_deref(),
    );

    Ok(Json(thread))
}

pub async fn delete_inline_comment(
    Path((id, thread_id, comment_id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let actor = authorize_codelab_actor(&state, &session, &id).await?;

    let row = sqlx::query_as::<_, MessageDeleteRow>(&state.q(
        "SELECT m.codelab_id, m.author_id, t.target_type, t.target_step_id
         FROM inline_comment_messages m
         JOIN inline_comment_threads t ON t.id = m.thread_id
         WHERE m.codelab_id = ? AND m.thread_id = ? AND m.id = ?
         LIMIT 1",
    ))
    .bind(&id)
    .bind(&thread_id)
    .bind(&comment_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::NOT_FOUND, "Inline comment not found".to_string()))?;

    if row.codelab_id != id {
        return Err(forbidden());
    }
    if !actor.is_admin && row.author_id != actor.id {
        return Err(forbidden());
    }

    sqlx::query(&state.q(
        "DELETE FROM inline_comment_messages WHERE codelab_id = ? AND thread_id = ? AND id = ?",
    ))
    .bind(&id)
    .bind(&thread_id)
    .bind(&comment_id)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let remaining: i64 = sqlx::query_scalar(&state.q(
        "SELECT COUNT(*) FROM inline_comment_messages WHERE codelab_id = ? AND thread_id = ?",
    ))
    .bind(&id)
    .bind(&thread_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut deleted_thread = false;
    if remaining == 0 {
        sqlx::query(&state.q(
            "DELETE FROM inline_comment_threads WHERE codelab_id = ? AND id = ?",
        ))
        .bind(&id)
        .bind(&thread_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
        deleted_thread = true;
    }

    broadcast_inline_comment_changed(&state, &id, &row.target_type, row.target_step_id.as_deref());

    Ok(Json(json!({
        "status": "ok",
        "thread_deleted": deleted_thread
    })))
}

async fn fetch_thread_with_messages(
    state: &Arc<AppState>,
    codelab_id: &str,
    thread_id: &str,
) -> Result<InlineCommentThreadWithMessages, (StatusCode, String)> {
    let thread = sqlx::query_as::<_, InlineCommentThread>(&state.q(
        "SELECT id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id, CAST(created_at AS TEXT) as created_at
         FROM inline_comment_threads
         WHERE codelab_id = ? AND id = ?
         LIMIT 1",
    ))
    .bind(codelab_id)
    .bind(thread_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::NOT_FOUND, "Inline comment thread not found".to_string()))?;

    let messages = sqlx::query_as::<_, InlineCommentMessage>(&state.q(
        "SELECT id, thread_id, codelab_id, author_role, author_id, author_name, message, CAST(created_at AS TEXT) as created_at
         FROM inline_comment_messages
         WHERE codelab_id = ? AND thread_id = ?
         ORDER BY created_at ASC",
    ))
    .bind(codelab_id)
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(InlineCommentThreadWithMessages { thread, messages })
}

async fn authorize_codelab_actor(
    state: &Arc<AppState>,
    session: &AuthSession,
    codelab_id: &str,
) -> Result<Actor, (StatusCode, String)> {
    if let Ok(admin) = session.require_admin() {
        return Ok(Actor {
            role: "admin".to_string(),
            id: admin.sub,
            name: "Facilitator".to_string(),
            is_admin: true,
        });
    }

    let attendee = session.require_attendee()?;
    if attendee.codelab_id.as_deref() != Some(codelab_id) {
        return Err(forbidden());
    }

    let attendee_name =
        sqlx::query_scalar::<_, String>(&state.q("SELECT name FROM attendees WHERE id = ?"))
            .bind(&attendee.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .unwrap_or_else(|| "Attendee".to_string());

    Ok(Actor {
        role: "attendee".to_string(),
        id: attendee.sub,
        name: attendee_name,
        is_admin: false,
    })
}

async fn validate_target_integrity(
    state: &Arc<AppState>,
    codelab_id: &str,
    target_type: &str,
    target_step_id: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    validate_target_type(target_type)?;

    let codelab_exists = sqlx::query_scalar::<_, String>(&state.q(
        "SELECT id FROM codelabs WHERE id = ? LIMIT 1",
    ))
    .bind(codelab_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;
    if codelab_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Codelab not found".to_string()));
    }

    match target_type {
        "guide" => {
            if target_step_id.is_some() {
                return Err(bad_request("guide comments cannot have target_step_id"));
            }
        }
        "step" => {
            let step_id = target_step_id.ok_or_else(|| bad_request("target_step_id is required"))?;
            let exists = sqlx::query_scalar::<_, String>(&state.q(
                "SELECT id FROM steps WHERE id = ? AND codelab_id = ? LIMIT 1",
            ))
            .bind(step_id)
            .bind(codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;
            if exists.is_none() {
                return Err((StatusCode::NOT_FOUND, "Step not found".to_string()));
            }
        }
        _ => return Err(bad_request("invalid target_type")),
    }

    Ok(())
}

fn validate_target_type(value: &str) -> Result<(), (StatusCode, String)> {
    if value == "step" || value == "guide" {
        Ok(())
    } else {
        Err(bad_request("invalid target_type"))
    }
}

fn validate_create_payload(payload: &CreateInlineCommentPayload) -> Result<(), (StatusCode, String)> {
    validate_common_anchor_fields(
        &payload.anchor_key,
        payload.start_offset,
        payload.end_offset,
        &payload.selected_text,
        &payload.content_hash,
    )?;
    validate_message(&payload.message)
}

fn validate_reply_payload(payload: &ReplyInlineCommentPayload) -> Result<(), (StatusCode, String)> {
    if payload.content_hash.trim().is_empty() || payload.content_hash.trim().len() > 128 {
        return Err(bad_request("content_hash is invalid"));
    }
    validate_message(&payload.message)
}

fn validate_common_anchor_fields(
    anchor_key: &str,
    start_offset: i32,
    end_offset: i32,
    selected_text: &str,
    content_hash: &str,
) -> Result<(), (StatusCode, String)> {
    let anchor_key = anchor_key.trim();
    if anchor_key.is_empty() || anchor_key.len() > 512 {
        return Err(bad_request("anchor_key is invalid"));
    }
    if start_offset < 0 || end_offset < 0 || end_offset <= start_offset {
        return Err(bad_request("invalid offsets"));
    }
    let selected_text = selected_text.trim();
    if selected_text.is_empty() || selected_text.len() > 2000 {
        return Err(bad_request("selected_text is invalid"));
    }
    let content_hash = content_hash.trim();
    if content_hash.is_empty() || content_hash.len() > 128 {
        return Err(bad_request("content_hash is invalid"));
    }
    Ok(())
}

fn validate_message(message: &str) -> Result<(), (StatusCode, String)> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Err(bad_request("message is required"));
    }
    if trimmed.len() > 1000 {
        return Err(bad_request("message is too long"));
    }
    Ok(())
}

fn broadcast_inline_comment_changed(
    state: &Arc<AppState>,
    codelab_id: &str,
    target_type: &str,
    target_step_id: Option<&str>,
) {
    if let Some(channel) = state.channels.get(codelab_id) {
        let payload = json!({
            "type": "inline_comment_changed",
            "target_type": target_type,
            "target_step_id": target_step_id
        })
        .to_string();
        let _ = channel.send(payload);
    }
}
