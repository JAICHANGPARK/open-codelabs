use crate::models::LoginPayload;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if payload.admin_id == state.admin_id && payload.admin_pw == state.admin_pw {
        Ok(Json(
            serde_json::json!({ "status": "ok", "token": "mock-jwt-token" }),
        ))
    } else {
        Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
    }
}

#[derive(Deserialize)]
pub struct SettingsPayload {
    pub gemini_api_key: String,
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SettingsPayload>,
) -> StatusCode {
    // In a real app, we'd use a proper session ID. 
    // Here, we use "global_admin" for simplicity since it's a single-facilitator tool for now.
    if !payload.gemini_api_key.trim().is_empty() {
        state.admin_api_keys.insert("global_admin".to_string(), payload.gemini_api_key.trim().to_string());
    } else {
        state.admin_api_keys.remove("global_admin");
    }
    StatusCode::OK
}
