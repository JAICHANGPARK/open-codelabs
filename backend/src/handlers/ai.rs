use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
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
    let api_key = match state.admin_api_keys.get("global_admin") {
        Some(entry) => entry.value().clone(),
        None => match payload.api_key {
            Some(key) if !key.is_empty() => key,
            _ => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
        },
    };

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

    let stream = response.bytes_stream().map(|result| {
        match result {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                // Google API는 SSE 형식을 따르므로 그대로 전달하거나 가공 가능
                // 여기서는 클라이언트에 그대로 Event로 감싸서 전달
                Ok(Event::default().data(text))
            }
            Err(e) => Err(e),
        }
    });

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}
