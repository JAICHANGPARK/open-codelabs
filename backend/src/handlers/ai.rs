use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use serde::Deserialize;
use std::sync::Arc;

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
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    // 1. Check for admin-provided key in memory (stored via settings)
    // 2. Fallback to request payload (for legacy/custom)
    // 3. Fallback to server env var
    let mut api_key = match state.admin_api_keys.get("global_admin") {
        Some(entry) => entry.value().clone(),
        None => match payload.api_key {
            Some(key) if !key.is_empty() => key,
            _ => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
        },
    };

    // Try to decrypt if it seems encrypted
    if !api_key.is_empty() && api_key.len() > 10 {
        let mc = new_magic_crypt!(&state.admin_pw, 256);
        if let Ok(decrypted) = mc.decrypt_base64_to_string(&api_key) {
            api_key = decrypted;
        }
    }

    if api_key.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Gemini API key is required".to_string(),
        )
            .into_response();
    }

    let model = payload.model.unwrap_or_else(|| "gemini-3-flash-preview".to_string());
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        model, api_key
    );

    let client = reqwest::Client::new();
    
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
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let stream = response.bytes_stream();
    
    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
}
