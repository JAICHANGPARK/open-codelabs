use crate::models::{Attendee, HelpRequest, HelpRequestPayload, RegistrationPayload, Codelab, CertificateInfo};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    Json,
};
use serde_json;
use sqlx;
use std::sync::Arc;
use uuid;

pub async fn register_attendee(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<RegistrationPayload>,
) -> Result<Json<Attendee>, (StatusCode, String)> {
    // Check if codelab is public
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    let is_admin = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s == "Bearer mock-jwt-token" || s == "mock-jwt-token")
        .unwrap_or(false);

    if codelab.is_public == 0 && !is_admin {
        return Err((StatusCode::FORBIDDEN, "This codelab is private".to_string()));
    }

    // Check for duplicate name in the same codelab
    let existing = sqlx::query(&state.q("SELECT id FROM attendees WHERE codelab_id = ? AND name = ?"))
        .bind(&id)
        .bind(&payload.name)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Nickname already taken".to_string()));
    }

    let attendee_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        &state.q("INSERT INTO attendees (id, codelab_id, name, code, current_step) VALUES (?, ?, ?, ?, 1)"),
    )
    .bind(&attendee_id)
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.code)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let attendee = sqlx::query_as::<_, Attendee>(&state.q("SELECT * FROM attendees WHERE id = ?"))
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
        &state.q("SELECT * FROM attendees WHERE codelab_id = ? ORDER BY created_at DESC"),
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
        &state.q("INSERT INTO help_requests (id, codelab_id, attendee_id, step_number) VALUES (?, ?, ?, ?)"),
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
        &state.q("SELECT hr.*, a.name as attendee_name FROM help_requests hr 
         JOIN attendees a ON hr.attendee_id = a.id 
         WHERE hr.codelab_id = ? AND hr.status = 'pending' 
         ORDER BY hr.created_at DESC"),
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(requests))
}

pub async fn resolve_help_request(
    State(state): State<Arc<AppState>>,
    Path((_id, help_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query(&state.q("UPDATE help_requests SET status = 'resolved' WHERE id = ?"))
        .bind(help_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn complete_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let attendee_id = headers
        .get("X-Attendee-ID")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Attendee ID".to_string()))?;

    sqlx::query(&state.q("UPDATE attendees SET is_completed = 1, completed_at = CURRENT_TIMESTAMP WHERE id = ? AND codelab_id = ?"))
        .bind(attendee_id)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn get_certificate(
    Path(attendee_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<CertificateInfo>, (StatusCode, String)> {
    let row: (String, String, String, String) = sqlx::query_as(&state.q(
        "SELECT a.name as attendee_name, c.title as codelab_title, c.author, a.completed_at 
         FROM attendees a 
         JOIN codelabs c ON a.codelab_id = c.id 
         WHERE a.id = ? AND a.is_completed = 1"
    ))
    .bind(&attendee_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CertificateInfo {
        attendee_name: row.0,
        codelab_title: row.1,
        author: row.2,
        completed_at: row.3,
        verification_url: format!("/verify/{}", attendee_id),
    }))
}
