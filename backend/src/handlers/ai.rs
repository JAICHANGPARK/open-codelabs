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
}

pub async fn proxy_gemini_stream(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    // 1. 요청 페이로드의 키 확인
    // 2. 없으면 서버 환경변수의 GEMINI_API_KEY 확인
    let api_key = match payload.api_key {
        Some(key) if !key.is_empty() => key,
        _ => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
    };

    if api_key.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Gemini API key is required (either in request or server config)".to_string(),
        )
            .into_response();
    }

    let model = "gemini-1.5-flash"; // 기본 모델
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        model, api_key
    );

    let client = reqwest::Client::new();
    
    // Google API에 보낼 페이로드 구성
    let mut contents = serde_json::json!([
        {
            "parts": [{ "text": payload.prompt }]
        }
    ]);

    // 시스템 명령어가 있는 경우 추가
    let body = if let Some(sys) = payload.system_instruction {
        serde_json::json!({
            "contents": contents,
            "system_instruction": {
                "parts": [{ "text": sys }]
            }
        })
    } else {
        serde_json::json!({ "contents": contents })
    };

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
