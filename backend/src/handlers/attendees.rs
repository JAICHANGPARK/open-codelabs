use crate::models::{Attendee, HelpRequest, HelpRequestPayload, RegistrationPayload};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json;
use sqlx;
use std::sync::Arc;
use uuid;

pub async fn register_attendee(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegistrationPayload>,
) -> Result<Json<Attendee>, (StatusCode, String)> {
    // Check for duplicate name in the same codelab
    let existing = sqlx::query("SELECT id FROM attendees WHERE codelab_id = ? AND name = ?")
        .bind(&id)
        .bind(&payload.name)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Nickname already taken".to_string()));
    }

    let attendee_id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO attendees (id, codelab_id, name, code) VALUES (?, ?, ?, ?)")
        .bind(&attendee_id)
        .bind(&id)
        .bind(&payload.name)
        .bind(&payload.code)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let attendee = sqlx::query_as::<_, Attendee>("SELECT * FROM attendees WHERE id = ?")
        .bind(&attendee_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(attendee))
}

pub async fn get_attendees(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Attendee>>, (StatusCode, String)> {
    let attendees = sqlx::query_as::<_, Attendee>(
        "SELECT * FROM attendees WHERE codelab_id = ? ORDER BY created_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(attendees))
}

pub async fn request_help(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<HelpRequestPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let attendee_id = headers
        .get("X-Attendee-ID")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Attendee ID".to_string()))?;

    let help_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO help_requests (id, codelab_id, attendee_id, step_number) VALUES (?, ?, ?, ?)",
    )
    .bind(&help_id)
    .bind(&id)
    .bind(attendee_id)
    .bind(payload.step_number)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Notify via WebSocket if possible
    if let Some(res) = state.channels.get(&id) {
        let msg = serde_json::json!({
            "type": "help_request",
            "attendee_id": attendee_id,
            "step_number": payload.step_number
        })
        .to_string();
        let _ = res.send(msg);
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn get_help_requests(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<HelpRequest>>, (StatusCode, String)> {
    let requests = sqlx::query_as::<_, HelpRequest>(
        "SELECT hr.*, a.name as attendee_name FROM help_requests hr 
         JOIN attendees a ON hr.attendee_id = a.id 
         WHERE hr.codelab_id = ? AND hr.status = 'pending' 
         ORDER BY hr.created_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(requests))
}

pub async fn resolve_help_request(
    Path((codelab_id, help_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query("UPDATE help_requests SET status = 'resolved' WHERE id = ?")
        .bind(&help_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Notify via WebSocket
    if let Some(res) = state.channels.get(&codelab_id) {
        let msg = serde_json::json!({
            "type": "help_resolved",
            "id": help_id,
        })
        .to_string();
        let _ = res.send(msg);
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
