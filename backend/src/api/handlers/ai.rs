use crate::domain::models::{AiConversation, SaveAiConversationPayload};
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::middleware::auth::AuthSession;
use crate::utils::crypto::decrypt_with_password;
use crate::utils::error::{bad_request, forbidden, internal_error};
use crate::middleware::request_info::RequestInfo;
use crate::infrastructure::database::AppState;
use crate::utils::validation::validate_prompt;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize)]
pub struct AiRequest {
    pub prompt: String,
    pub system_instruction: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub generation_config: Option<serde_json::Value>,
    pub tools: Option<serde_json::Value>,
    pub codelab_id: Option<String>,
    pub step_number: Option<i32>,
}

pub async fn proxy_gemini_stream(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    // Allow both admin and attendees to use AI
    let (user_id, user_type, user_name) = if let Ok(admin) = session.require_admin() {
        (admin.sub, "admin".to_string(), "Admin".to_string())
    } else if let Ok(attendee) = session.require_attendee() {
        // For attendees, verify they belong to the codelab if codelab_id is provided
        if let Some(codelab_id) = &payload.codelab_id {
            if attendee.codelab_id.as_deref() != Some(codelab_id.as_str()) {
                return forbidden().into_response();
            }
        }
        // Get attendee name from database
        let attendee_name = match sqlx::query_scalar::<_, String>(&state.q("SELECT name FROM attendees WHERE id = ?"))
            .bind(&attendee.sub)
            .fetch_one(&state.pool)
            .await
        {
            Ok(name) => name,
            Err(_) => "Unknown".to_string(),
        };
        (attendee.sub, "attendee".to_string(), attendee_name)
    } else {
        return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response();
    };

    if let Err(err) = validate_prompt(&payload.prompt) {
        return err.into_response();
    }
    // 1. Check for admin-provided key in memory (stored via settings)
    // 2. Fallback to request payload (for legacy/custom)
    // 3. Fallback to server env var
    let payload_key = payload.api_key.as_ref().filter(|key| !key.is_empty());
    let payload_key_provided = payload_key.is_some();
    let mut api_key = match state.admin_api_keys.get("global_admin") {
        Some(entry) => entry.value().clone(),
        None => payload_key
            .map(|key| key.to_string())
            .unwrap_or_else(|| std::env::var("GEMINI_API_KEY").unwrap_or_default()),
    };

    if !api_key.is_empty() && payload_key_provided {
        match decrypt_with_password(&api_key, &state.admin_pw) {
            Ok(decrypted) => api_key = decrypted,
            Err(_) => return bad_request("Invalid encrypted API key format").into_response(),
        }
    }

    if api_key.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Gemini API key is required".to_string(),
        )
            .into_response();
    }

    let model = payload
        .model
        .unwrap_or_else(|| "gemini-3-flash-preview".to_string());
    if !is_valid_model(&model) {
        return bad_request("Invalid model").into_response();
    }
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        model, api_key
    );

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(e) => return internal_error(e).into_response(),
    };

    let contents = serde_json::json!([
        {
            "role": "user",
            "parts": [{ "text": payload.prompt }]
        }
    ]);

    let mut body = serde_json::json!({
        "contents": contents,
    });

    if let Some(sys) = payload.system_instruction {
        body["system_instruction"] = serde_json::json!({
            "parts": [{ "text": sys }]
        });
    }

    if let Some(config) = payload.generation_config {
        body["generationConfig"] = config;
    }

    if let Some(tools) = payload.tools {
        body["tools"] = tools;
    }

    let response = match client.post(&url).json(&body).send().await {
        Ok(res) => res,
        Err(e) => {
            return internal_error(e).into_response();
        }
    };

    let stream = response.bytes_stream();

    // Record audit log with user information and codelab context
    let metadata = serde_json::json!({
        "prompt_length": payload.prompt.len(),
        "model": model,
        "step_number": payload.step_number,
    });

    record_audit(
        &state,
        AuditEntry {
            action: "ai_query".to_string(),
            actor_type: user_type.clone(),
            actor_id: Some(user_id.clone()),
            target_id: None,
            codelab_id: payload.codelab_id.clone(),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: Some(metadata),
        },
    )
    .await;

    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
}

pub async fn save_ai_conversation(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    _info: RequestInfo,
    Json(payload): Json<SaveAiConversationPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Get user information
    let (user_id, user_type, user_name) = if let Ok(admin) = session.require_admin() {
        (admin.sub, "admin".to_string(), "Admin".to_string())
    } else if let Ok(attendee) = session.require_attendee() {
        // Verify attendee belongs to the codelab
        if attendee.codelab_id.as_deref() != Some(payload.codelab_id.as_str()) {
            return Err(forbidden());
        }
        // Get attendee name
        let attendee_name = sqlx::query_scalar::<_, String>(&state.q("SELECT name FROM attendees WHERE id = ?"))
            .bind(&attendee.sub)
            .fetch_one(&state.pool)
            .await
            .map_err(internal_error)?;
        (attendee.sub, "attendee".to_string(), attendee_name)
    } else {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    };

    let conversation_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(&state.q(
        "INSERT INTO ai_conversations (id, codelab_id, user_id, user_type, user_name, step_number, question, answer, model) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    ))
    .bind(&conversation_id)
    .bind(&payload.codelab_id)
    .bind(&user_id)
    .bind(&user_type)
    .bind(&user_name)
    .bind(payload.step_number)
    .bind(&payload.question)
    .bind(&payload.answer)
    .bind(&payload.model)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(serde_json::json!({ "id": conversation_id })))
}

pub async fn get_ai_conversations(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<AiConversation>>, (StatusCode, String)> {
    // Only admin can view all AI conversations
    session.require_admin()?;

    let conversations = sqlx::query_as::<_, AiConversation>(&state.q(
        "SELECT * FROM ai_conversations WHERE codelab_id = ? ORDER BY created_at DESC",
    ))
    .bind(&codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(conversations))
}

fn is_valid_model(model: &str) -> bool {
    if model.is_empty() || model.len() > 64 {
        return false;
    }
    if let Ok(allowed) = std::env::var("ALLOWED_GEMINI_MODELS") {
        let list: Vec<&str> = allowed.split(',').map(|value| value.trim()).collect();
        return list.iter().any(|entry| entry == &model);
    }
    model
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
