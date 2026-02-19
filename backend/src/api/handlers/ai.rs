use crate::api::dto::AiRequest;
use crate::domain::models::{
    AddAiMessagePayload, AiConversation, AiMessage, AiThread, CreateAiThreadPayload,
    SaveAiConversationPayload,
};
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::infrastructure::database::AppState;
use crate::middleware::auth::AuthSession;
use crate::middleware::request_info::RequestInfo;
use crate::utils::crypto::decrypt_with_password;
use crate::utils::error::{bad_request, forbidden, internal_error};
use crate::utils::validation::validate_prompt;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use std::time::Duration;

pub async fn proxy_gemini_stream(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    // Allow both admin and attendees to use AI
    let (user_id, user_type, _user_name) = if let Ok(admin) = session.require_admin() {
        (admin.sub, "admin".to_string(), "Admin".to_string())
    } else if let Ok(attendee) = session.require_attendee() {
        // For attendees, verify they belong to the codelab if codelab_id is provided.
        if payload
            .codelab_id
            .as_deref()
            .is_some_and(|codelab_id| attendee.codelab_id.as_deref() != Some(codelab_id))
        {
            return forbidden().into_response();
        }
        // Get attendee name from database
        let attendee_name = match sqlx::query_scalar::<_, String>(
            &state.q("SELECT name FROM attendees WHERE id = ?"),
        )
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

    // Validate: either prompt or contents must be provided
    if payload.prompt.is_none() && payload.contents.is_none() {
        return bad_request("Either prompt or contents must be provided").into_response();
    }

    if let Some(prompt) = &payload.prompt {
        if let Err(err) = validate_prompt(prompt) {
            return err.into_response();
        }
    }
    // 1. Check for admin-provided key in memory (stored via settings)
    // 2. Fallback to request payload (for legacy/custom)
    // 3. Fallback to server env var
    let payload_key = payload.api_key.as_ref().filter(|key| !key.is_empty());
    let payload_key_provided = payload_key.is_some();

    enum ApiKeySource {
        AdminStored,
        Payload,
        Env,
    }

    let (mut api_key, source) = match state.admin_api_keys.get("global_admin") {
        Some(entry) => (entry.value().clone(), ApiKeySource::AdminStored),
        None => match payload_key {
            Some(key) => (key.to_string(), ApiKeySource::Payload),
            None => (
                std::env::var("GEMINI_API_KEY").unwrap_or_default(),
                ApiKeySource::Env,
            ),
        },
    };

    // Only decrypt when we actually use the payload key.
    if !api_key.is_empty() && matches!(source, ApiKeySource::Payload) && payload_key_provided {
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
    let api_base = std::env::var("GEMINI_API_BASE")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());
    let url = format!(
        "{}/models/{}:streamGenerateContent?alt=sse&key={}",
        api_base.trim_end_matches('/'),
        model,
        api_key
    );

    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let final_contents = if let Some(contents) = payload.contents.clone() {
        contents
    } else {
        serde_json::json!([
            {
                "role": "user",
                "parts": [{ "text": payload.prompt.as_deref().unwrap_or_default() }]
            }
        ])
    };

    let mut body = serde_json::json!({
        "contents": final_contents,
    });

    if let Some(sys) = payload.system_instruction.clone() {
        body["system_instruction"] = serde_json::json!({
            "parts": [{ "text": sys }]
        });
    }

    if let Some(config) = payload.generation_config.clone() {
        body["generationConfig"] = config;
    }

    if let Some(tools) = payload.tools.clone() {
        body["tools"] = tools;
    }

    let response = match client.post(&url).json(&body).send().await {
        Ok(res) => res,
        Err(e) => return internal_error(e).into_response(),
    };

    let stream = response.bytes_stream();

    // Record audit log with user information and codelab context
    let metadata = serde_json::json!({
        "prompt_length": payload.prompt.as_deref().map(|p| p.len()).unwrap_or(0),
        "has_contents": payload.contents.is_some(),
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
        let attendee_name =
            sqlx::query_scalar::<_, String>(&state.q("SELECT name FROM attendees WHERE id = ?"))
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
        "INSERT INTO ai_conversations (id, codelab_id, user_id, user_type, user_name, step_number, question, answer, model, usage_metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
    .bind(payload.usage_metadata.map(|m| m.to_string()))
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

    let conversations = sqlx::query_as::<_, AiConversation>(
        &state.q("SELECT id, codelab_id, user_id, user_type, user_name, step_number, question, answer, model, usage_metadata, CAST(created_at AS TEXT) as created_at FROM ai_conversations WHERE codelab_id = ? ORDER BY created_at DESC"),
    )
    .bind(&codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(conversations))
}

pub async fn create_ai_thread(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Json(payload): Json<CreateAiThreadPayload>,
) -> Result<Json<AiThread>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    let thread_id = uuid::Uuid::new_v4().to_string();

    let thread = sqlx::query_as::<_, AiThread>(&state.q(
        "INSERT INTO ai_threads (id, title, user_id, user_type, codelab_id) VALUES (?, ?, ?, ?, ?) RETURNING id, title, user_id, user_type, codelab_id, CAST(created_at AS TEXT) as created_at, CAST(updated_at AS TEXT) as updated_at"
    ))
    .bind(&thread_id)
    .bind(&payload.title)
    .bind(&admin.sub)
    .bind("admin")
    .bind(&payload.codelab_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(thread))
}

pub async fn get_ai_threads(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<AiThread>>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    let threads = sqlx::query_as::<_, AiThread>(&state.q(
        "SELECT id, title, user_id, user_type, codelab_id, CAST(created_at AS TEXT) as created_at, CAST(updated_at AS TEXT) as updated_at FROM ai_threads WHERE user_id = ? AND user_type = 'admin' ORDER BY updated_at DESC"
    ))
    .bind(&admin.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(threads))
}

pub async fn delete_ai_thread(
    Path(thread_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<StatusCode, (StatusCode, String)> {
    let admin = session.require_admin()?;

    sqlx::query(&state.q("DELETE FROM ai_threads WHERE id = ? AND user_id = ?"))
        .bind(&thread_id)
        .bind(&admin.sub)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_ai_message(
    Path(thread_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Json(payload): Json<AddAiMessagePayload>,
) -> Result<Json<AiMessage>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify thread belongs to user
    let thread_exists = sqlx::query_scalar::<_, i32>(
        &state.q("SELECT COUNT(*) FROM ai_threads WHERE id = ? AND user_id = ?"),
    )
    .bind(&thread_id)
    .bind(&admin.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if thread_exists == 0 {
        return Err(forbidden());
    }

    let message_id = uuid::Uuid::new_v4().to_string();
    let grounding_metadata = payload.grounding_metadata.map(|m| m.to_string());
    let usage_metadata = payload.usage_metadata.map(|m| m.to_string());

    let message = sqlx::query_as::<_, AiMessage>(&state.q(
        "INSERT INTO ai_messages (id, thread_id, role, content, grounding_metadata, usage_metadata) VALUES (?, ?, ?, ?, ?, ?) RETURNING id, thread_id, role, content, grounding_metadata, usage_metadata, CAST(created_at AS TEXT) as created_at"
    ))
    .bind(&message_id)
    .bind(&thread_id)
    .bind(&payload.role)
    .bind(&payload.content)
    .bind(&grounding_metadata)
    .bind(&usage_metadata)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    // Update thread updated_at
    sqlx::query(&state.q("UPDATE ai_threads SET updated_at = CURRENT_TIMESTAMP WHERE id = ?"))
        .bind(&thread_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok(Json(message))
}

pub async fn get_ai_messages(
    Path(thread_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<AiMessage>>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify thread belongs to user
    let thread_exists = sqlx::query_scalar::<_, i32>(
        &state.q("SELECT COUNT(*) FROM ai_threads WHERE id = ? AND user_id = ?"),
    )
    .bind(&thread_id)
    .bind(&admin.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if thread_exists == 0 {
        return Err(forbidden());
    }

    let messages = sqlx::query_as::<_, AiMessage>(
        &state.q("SELECT id, thread_id, role, content, grounding_metadata, usage_metadata, CAST(created_at AS TEXT) as created_at FROM ai_messages WHERE thread_id = ? ORDER BY created_at ASC"),
    )
    .bind(&thread_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(messages))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    struct EnvRestore {
        key: &'static str,
        value: Option<String>,
    }

    impl EnvRestore {
        fn new(key: &'static str) -> Self {
            Self {
                key,
                value: std::env::var(key).ok(),
            }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            if let Some(value) = &self.value {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    static ENV_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn is_valid_model_uses_default_rules() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _guard = EnvRestore::new("ALLOWED_GEMINI_MODELS");
        std::env::remove_var("ALLOWED_GEMINI_MODELS");

        assert!(is_valid_model("gemini-3-flash-preview"));
        assert!(!is_valid_model(""));
        assert!(!is_valid_model(&"a".repeat(65)));
        assert!(!is_valid_model("Model-With-Uppercase"));
        assert!(!is_valid_model("bad/model"));
    }

    #[test]
    fn is_valid_model_respects_allow_list_env() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _guard = EnvRestore::new("ALLOWED_GEMINI_MODELS");
        std::env::set_var(
            "ALLOWED_GEMINI_MODELS",
            "gemini-3-flash-preview, custom-model",
        );

        assert!(is_valid_model("custom-model"));
        assert!(!is_valid_model("gemini-2.0"));
    }

    #[test]
    fn env_restore_restores_previous_value() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        std::env::set_var("ALLOWED_GEMINI_MODELS", "before");
        {
            let _guard = EnvRestore::new("ALLOWED_GEMINI_MODELS");
            std::env::set_var("ALLOWED_GEMINI_MODELS", "during");
        }
        assert_eq!(
            std::env::var("ALLOWED_GEMINI_MODELS").as_deref(),
            Ok("before")
        );
        std::env::remove_var("ALLOWED_GEMINI_MODELS");
    }
}
