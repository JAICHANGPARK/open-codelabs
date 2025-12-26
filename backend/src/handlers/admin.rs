use crate::models::LoginPayload;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
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
