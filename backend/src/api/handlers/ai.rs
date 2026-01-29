use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::middleware::auth::AuthSession;
use crate::crypto::decrypt_with_password;
use crate::utils::error::{bad_request, internal_error};
use crate::request_info::RequestInfo;
use crate::infrastructure::database::AppState;
use crate::utils::validation::validate_prompt;
use axum::{
    body::Body,
    extract::State,
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
}

pub async fn proxy_gemini_stream(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    if session.require_admin().is_err() {
        return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response();
    }
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

    record_audit(
        &state,
        AuditEntry {
            action: "ai_proxy_request".to_string(),
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

    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
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
