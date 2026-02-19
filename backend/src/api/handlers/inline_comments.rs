use crate::domain::models::{
    CreateInlineCommentPayload, InlineCommentMessage, InlineCommentThread,
    InlineCommentThreadWithMessages, ReplyInlineCommentPayload,
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

    query
        .target_type
        .as_deref()
        .map(validate_target_type)
        .transpose()?;

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

    query.target_type.as_ref().map(|target_type| {
        threads.retain(|thread| thread.target_type == *target_type);
    });
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
    validate_target_integrity(
        &state,
        &id,
        &payload.target_type,
        payload.target_step_id.as_deref(),
    )
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
    sqlx::query(&state.q("INSERT INTO inline_comment_messages
         (id, thread_id, codelab_id, author_role, author_id, author_name, message)
         VALUES (?, ?, ?, ?, ?, ?, ?)"))
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
    sqlx::query(&state.q("INSERT INTO inline_comment_messages
         (id, thread_id, codelab_id, author_role, author_id, author_name, message)
         VALUES (?, ?, ?, ?, ?, ?, ?)"))
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
        "SELECT m.author_id, t.target_type, t.target_step_id
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
    .ok_or((
        StatusCode::NOT_FOUND,
        "Inline comment not found".to_string(),
    ))?;

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

    let remaining: i64 =
        sqlx::query_scalar(&state.q(
            "SELECT COUNT(*) FROM inline_comment_messages WHERE codelab_id = ? AND thread_id = ?",
        ))
        .bind(&id)
        .bind(&thread_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let mut deleted_thread = false;
    if remaining == 0 {
        sqlx::query(&state.q("DELETE FROM inline_comment_threads WHERE codelab_id = ? AND id = ?"))
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

    let codelab_exists =
        sqlx::query_scalar::<_, String>(&state.q("SELECT id FROM codelabs WHERE id = ? LIMIT 1"))
            .bind(codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;
    if codelab_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Codelab not found".to_string()));
    }

    if target_type == "guide" {
        if target_step_id.is_some() {
            return Err(bad_request("guide comments cannot have target_step_id"));
        }
        return Ok(());
    }

    let step_id = target_step_id.ok_or_else(|| bad_request("target_step_id is required"))?;
    let exists = sqlx::query_scalar::<_, String>(
        &state.q("SELECT id FROM steps WHERE id = ? AND codelab_id = ? LIMIT 1"),
    )
    .bind(step_id)
    .bind(codelab_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;
    if exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Step not found".to_string()));
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

fn validate_create_payload(
    payload: &CreateInlineCommentPayload,
) -> Result<(), (StatusCode, String)> {
    let (k, s, e, t, h) = (
        &payload.anchor_key,
        payload.start_offset,
        payload.end_offset,
        &payload.selected_text,
        &payload.content_hash,
    );
    validate_common_anchor_fields(k, s, e, t, h)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{AppState, DbKind};
    use crate::middleware::auth::SessionClaims;
    use axum::extract::{Path, Query, State};
    use axum::http::StatusCode;
    use serde_json::json;
    use sqlx::any::AnyPoolOptions;
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn make_state() -> Arc<AppState> {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("sqlite");
        Arc::new(AppState::new(
            pool,
            DbKind::Sqlite,
            "admin".to_string(),
            "pw".to_string(),
            false,
        ))
    }

    async fn make_state_with_schema() -> Arc<AppState> {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .expect("sqlite");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        Arc::new(AppState::new(
            pool,
            DbKind::Sqlite,
            "admin".to_string(),
            "pw".to_string(),
            false,
        ))
    }

    fn claims(role: &str, sub: &str, codelab_id: Option<&str>) -> SessionClaims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_secs() as usize;
        SessionClaims {
            sub: sub.to_string(),
            role: role.to_string(),
            codelab_id: codelab_id.map(|id| id.to_string()),
            iss: "open-codelabs".to_string(),
            aud: "open-codelabs".to_string(),
            iat: now,
            exp: now + 3600,
        }
    }

    fn admin_session() -> AuthSession {
        let admin = claims("admin", "admin-1", None);
        AuthSession {
            claims: Some(admin.clone()),
            admin_claims: Some(admin),
            attendee_claims: None,
        }
    }

    fn attendee_session(sub: &str, codelab_id: &str) -> AuthSession {
        let attendee = claims("attendee", sub, Some(codelab_id));
        AuthSession {
            claims: Some(attendee.clone()),
            admin_claims: None,
            attendee_claims: Some(attendee),
        }
    }

    async fn seed_codelab_step(state: &AppState, codelab_id: &str, step_id: &str) {
        sqlx::query(&state.q(
            "INSERT INTO codelabs (id, title, description, author, is_public, quiz_enabled, require_quiz, require_feedback, require_submission, guide_markdown) VALUES (?, 't', 'd', 'a', 1, 0, 0, 0, 0, NULL)",
        ))
        .bind(codelab_id)
        .execute(&state.pool)
        .await
        .expect("insert codelab");
        sqlx::query(&state.q(
            "INSERT INTO steps (id, codelab_id, step_number, title, content_markdown) VALUES (?, ?, 1, 's1', '# step')",
        ))
        .bind(step_id)
        .bind(codelab_id)
        .execute(&state.pool)
        .await
        .expect("insert step");
    }

    #[test]
    fn validate_target_type_checks_values() {
        assert!(validate_target_type("step").is_ok());
        assert!(validate_target_type("guide").is_ok());
        assert!(validate_target_type("other").is_err());
    }

    #[test]
    fn validate_common_anchor_fields_checks_constraints() {
        assert!(validate_common_anchor_fields("k", 0, 1, "text", "hash").is_ok());
        assert!(validate_common_anchor_fields("", 0, 1, "text", "hash").is_err());
        assert!(validate_common_anchor_fields("k", 5, 1, "text", "hash").is_err());
        assert!(validate_common_anchor_fields("k", 0, 1, "", "hash").is_err());
        assert!(validate_common_anchor_fields("k", 0, 1, "text", "").is_err());
    }

    #[test]
    fn validate_payloads_check_message_and_hash() {
        let create = CreateInlineCommentPayload {
            anchor_key: "a1".to_string(),
            target_type: "step".to_string(),
            target_step_id: Some("s1".to_string()),
            start_offset: 1,
            end_offset: 2,
            selected_text: "hi".to_string(),
            content_hash: "h".to_string(),
            message: "hello".to_string(),
        };
        assert!(validate_create_payload(&create).is_ok());

        let reply = ReplyInlineCommentPayload {
            message: "reply".to_string(),
            content_hash: "h2".to_string(),
        };
        assert!(validate_reply_payload(&reply).is_ok());

        let bad_reply = ReplyInlineCommentPayload {
            message: "ok".to_string(),
            content_hash: " ".to_string(),
        };
        assert!(validate_reply_payload(&bad_reply).is_err());
    }

    #[test]
    fn validate_message_checks_required_and_length() {
        assert!(validate_message("ok").is_ok());
        assert!(validate_message("   ").is_err());
        assert!(validate_message(&"m".repeat(1001)).is_err());
    }

    #[tokio::test]
    async fn broadcast_inline_comment_changed_sends_message() {
        let state = make_state().await;
        let codelab_id = "lab-1";
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        state.channels.insert(codelab_id.to_string(), tx);

        broadcast_inline_comment_changed(&state, codelab_id, "step", Some("s1"));
        let msg = rx.recv().await.expect("broadcast");
        let payload: serde_json::Value = serde_json::from_str(&msg).expect("json");
        assert_eq!(payload["type"], "inline_comment_changed");
        assert_eq!(payload["target_type"], "step");
        assert_eq!(payload["target_step_id"], "s1");
    }

    #[tokio::test]
    async fn validate_target_integrity_covers_error_paths() {
        let state = make_state_with_schema().await;
        seed_codelab_step(&state, "lab-1", "step-1").await;

        let missing_codelab = validate_target_integrity(&state, "missing", "guide", None).await;
        assert_eq!(missing_codelab.unwrap_err().0, StatusCode::NOT_FOUND);

        let guide_with_step =
            validate_target_integrity(&state, "lab-1", "guide", Some("step-1")).await;
        assert_eq!(guide_with_step.unwrap_err().0, StatusCode::BAD_REQUEST);

        let guide_ok = validate_target_integrity(&state, "lab-1", "guide", None).await;
        assert!(guide_ok.is_ok());

        let missing_step =
            validate_target_integrity(&state, "lab-1", "step", Some("missing-step")).await;
        assert_eq!(missing_step.unwrap_err().0, StatusCode::NOT_FOUND);

        let invalid_type = validate_target_integrity(&state, "lab-1", "other", None).await;
        assert_eq!(invalid_type.unwrap_err().0, StatusCode::BAD_REQUEST);

        let ok = validate_target_integrity(&state, "lab-1", "step", Some("step-1")).await;
        assert!(ok.is_ok());
    }

    #[tokio::test]
    async fn get_inline_comments_filters_by_target_and_step() {
        let state = make_state_with_schema().await;
        seed_codelab_step(&state, "lab-1", "step-1").await;
        sqlx::query(&state.q(
            "INSERT INTO inline_comment_threads (id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id) VALUES ('th-step', 'lab-1', 'a1', 'step', 'step-1', 1, 2, 'x', 'h1', 'att-1')",
        ))
        .execute(&state.pool)
        .await
        .expect("insert step thread");
        sqlx::query(&state.q(
            "INSERT INTO inline_comment_threads (id, codelab_id, anchor_key, target_type, target_step_id, start_offset, end_offset, selected_text, content_hash, created_by_attendee_id) VALUES ('th-guide', 'lab-1', 'a2', 'guide', NULL, 1, 2, 'x', 'h2', 'att-1')",
        ))
        .execute(&state.pool)
        .await
        .expect("insert guide thread");
        sqlx::query(&state.q(
            "INSERT INTO inline_comment_messages (id, thread_id, codelab_id, author_role, author_id, author_name, message) VALUES ('m1', 'th-step', 'lab-1', 'attendee', 'att-1', 'A', 'hello')",
        ))
        .execute(&state.pool)
        .await
        .expect("insert message");

        let res = get_inline_comments(
            Path("lab-1".to_string()),
            State(state.clone()),
            Query(InlineCommentQuery {
                target_type: Some("step".to_string()),
                target_step_id: Some("step-1".to_string()),
            }),
            admin_session(),
        )
        .await
        .expect("get comments");
        assert_eq!(res.0.len(), 1);
        assert_eq!(res.0[0].thread.id, "th-step");

        let invalid = get_inline_comments(
            Path("lab-1".to_string()),
            State(state.clone()),
            Query(InlineCommentQuery {
                target_type: Some("invalid".to_string()),
                target_step_id: None,
            }),
            admin_session(),
        )
        .await;
        assert_eq!(invalid.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_reply_delete_inline_comment_covers_conflict_overlap_and_forbidden() {
        let state = make_state_with_schema().await;
        seed_codelab_step(&state, "lab-1", "step-1").await;
        sqlx::query(&state.q(
            "INSERT INTO attendees (id, codelab_id, name, code) VALUES ('att-1', 'lab-1', 'Alice', 'x')",
        ))
        .execute(&state.pool)
        .await
        .expect("insert attendee");

        let created = create_inline_comment(
            Path("lab-1".to_string()),
            State(state.clone()),
            admin_session(),
            Json(CreateInlineCommentPayload {
                anchor_key: "anchor-1".to_string(),
                target_type: "step".to_string(),
                target_step_id: Some("step-1".to_string()),
                start_offset: 1,
                end_offset: 5,
                selected_text: "abcd".to_string(),
                content_hash: "hash-1".to_string(),
                message: "hello".to_string(),
            }),
        )
        .await
        .expect("create");
        assert_eq!(created.0.messages.len(), 1);

        let reused = create_inline_comment(
            Path("lab-1".to_string()),
            State(state.clone()),
            admin_session(),
            Json(CreateInlineCommentPayload {
                anchor_key: "anchor-1".to_string(),
                target_type: "step".to_string(),
                target_step_id: Some("step-1".to_string()),
                start_offset: 1,
                end_offset: 5,
                selected_text: "abcd".to_string(),
                content_hash: "hash-1".to_string(),
                message: "hello again".to_string(),
            }),
        )
        .await
        .expect("reuse");
        assert_eq!(reused.0.thread.id, created.0.thread.id);
        assert_eq!(reused.0.messages.len(), 2);

        let conflict = create_inline_comment(
            Path("lab-1".to_string()),
            State(state.clone()),
            admin_session(),
            Json(CreateInlineCommentPayload {
                anchor_key: "anchor-1".to_string(),
                target_type: "step".to_string(),
                target_step_id: Some("step-1".to_string()),
                start_offset: 1,
                end_offset: 6,
                selected_text: "abcd".to_string(),
                content_hash: "hash-1".to_string(),
                message: "conflict".to_string(),
            }),
        )
        .await;
        assert_eq!(conflict.unwrap_err().0, StatusCode::BAD_REQUEST);

        let overlap_payload = CreateInlineCommentPayload {
            anchor_key: "anchor-2".to_string(),
            target_type: "step".to_string(),
            target_step_id: Some("step-1".to_string()),
            start_offset: 4,
            end_offset: 8,
            selected_text: "efgh".to_string(),
            content_hash: "hash-1".to_string(),
            message: "overlap".to_string(),
        };
        let overlap = create_inline_comment(
            Path("lab-1".to_string()),
            State(state.clone()),
            admin_session(),
            Json(overlap_payload),
        )
        .await;
        assert_eq!(overlap.unwrap_err().0, StatusCode::BAD_REQUEST);

        let stale_reply = reply_inline_comment(
            Path(("lab-1".to_string(), created.0.thread.id.clone())),
            State(state.clone()),
            admin_session(),
            Json(ReplyInlineCommentPayload {
                message: "reply".to_string(),
                content_hash: "stale".to_string(),
            }),
        )
        .await;
        assert_eq!(stale_reply.unwrap_err().0, StatusCode::BAD_REQUEST);

        let message_id: String = sqlx::query_scalar(&state.q(
            "SELECT id FROM inline_comment_messages WHERE thread_id = ? ORDER BY created_at ASC LIMIT 1",
        ))
        .bind(&created.0.thread.id)
        .fetch_one(&state.pool)
        .await
        .expect("message id");

        let forbidden_delete = delete_inline_comment(
            Path(("lab-1".to_string(), created.0.thread.id.clone(), message_id)),
            State(state.clone()),
            attendee_session("att-2", "lab-1"),
        )
        .await;
        assert_eq!(forbidden_delete.unwrap_err().0, StatusCode::FORBIDDEN);

        let wrong_lab = get_inline_comments(
            Path("lab-1".to_string()),
            State(state.clone()),
            Query(InlineCommentQuery::default()),
            attendee_session("att-2", "lab-2"),
        )
        .await;
        assert_eq!(wrong_lab.unwrap_err().0, StatusCode::FORBIDDEN);

        let payload = json!({
            "type": "inline_comment_changed",
            "target_type": "step",
            "target_step_id": "step-1"
        });
        assert_eq!(payload["type"], "inline_comment_changed");
    }
}
