use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::middleware::auth::{
    build_csrf_cookie, build_session_cookie, now_epoch_seconds, AuthSession, Role, SessionClaims,
};
use crate::utils::crypto::{decrypt_with_password, encrypt_with_password};
use crate::utils::error::{bad_request, forbidden, internal_error};
use crate::domain::models::{
    Attendee, CertificateInfo, Codelab, HelpRequest, HelpRequestPayload, RegistrationPayload,
};
use crate::middleware::request_info::RequestInfo;
use crate::infrastructure::database::AppState;
use crate::utils::validation::validate_registration;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use serde_json;
use sqlx;
use std::sync::Arc;
use uuid;

pub async fn register_attendee(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    jar: CookieJar,
    info: RequestInfo,
    Json(payload): Json<RegistrationPayload>,
) -> Result<(CookieJar, Json<crate::domain::models::AttendeePublic>), (StatusCode, String)> {
    validate_registration(&payload)?;
    // Check if codelab is public
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    let is_admin = session
        .claims
        .as_ref()
        .map(|claims| claims.role == "admin")
        .unwrap_or(false);

    if codelab.is_public == 0 && !is_admin {
        return Err(forbidden());
    }

    // Check for duplicate name in the same codelab
    let existing =
        sqlx::query(&state.q("SELECT id FROM attendees WHERE codelab_id = ? AND name = ?"))
            .bind(&id)
            .bind(&payload.name)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Nickname already taken".to_string()));
    }

    let attendee_id = uuid::Uuid::new_v4().to_string();
    let encrypted_code = encrypt_with_password(&payload.code, &state.admin_pw)
        .map_err(|err| internal_error(err))?;

    sqlx::query(&state.q(
        "INSERT INTO attendees (id, codelab_id, name, code, current_step) VALUES (?, ?, ?, ?, 1)",
    ))
    .bind(&attendee_id)
    .bind(&id)
    .bind(&payload.name)
    .bind(&encrypted_code)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    let attendee = sqlx::query_as::<_, Attendee>(&state.q("SELECT * FROM attendees WHERE id = ?"))
        .bind(&attendee_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let now = now_epoch_seconds();
    let claims = SessionClaims {
        sub: attendee_id.clone(),
        role: Role::Attendee.as_str().to_string(),
        codelab_id: Some(id.clone()),
        iss: state.auth.issuer.clone(),
        aud: state.auth.audience.clone(),
        iat: now,
        exp: now + state.auth.attendee_ttl.as_secs() as usize,
    };
    let token = state.auth.issue_token(&claims).map_err(internal_error)?;
    let csrf_token = crate::middleware::auth::generate_csrf_token();
    let jar = jar
        .add(build_session_cookie(
            &state.auth,
            token,
            state.auth.attendee_ttl,
        ))
        .add(build_csrf_cookie(
            &state.auth,
            csrf_token,
            state.auth.attendee_ttl,
        ));

    record_audit(
        &state,
        AuditEntry {
            action: "attendee_register".to_string(),
            actor_type: "attendee".to_string(),
            actor_id: Some(attendee_id.clone()),
            target_id: None,
            codelab_id: Some(id.clone()),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok((jar, Json(crate::domain::models::AttendeePublic::from(attendee))))
}

pub async fn get_attendees(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<Attendee>>, (StatusCode, String)> {
    session.require_admin()?;
    let mut attendees = sqlx::query_as::<_, Attendee>(
        &state.q("SELECT * FROM attendees WHERE codelab_id = ? ORDER BY created_at DESC"),
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    for attendee in attendees.iter_mut() {
        if let Ok(decrypted) = decrypt_with_password(&attendee.code, &state.admin_pw) {
            attendee.code = decrypted;
        }
    }

    Ok(Json(attendees))
}

pub async fn request_help(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<HelpRequestPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let attendee = session.require_attendee()?;
    if attendee.codelab_id.as_deref() != Some(id.as_str()) {
        return Err(forbidden());
    }
    if payload.step_number < 1 {
        return Err(bad_request("invalid step_number"));
    }
    let attendee_id = attendee.sub;

    let help_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(&state.q(
        "INSERT INTO help_requests (id, codelab_id, attendee_id, step_number) VALUES (?, ?, ?, ?)",
    ))
    .bind(&help_id)
    .bind(&id)
    .bind(&attendee_id)
    .bind(payload.step_number)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

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

    record_audit(
        &state,
        AuditEntry {
            action: "help_request".to_string(),
            actor_type: "attendee".to_string(),
            actor_id: Some(attendee_id.clone()),
            target_id: Some(help_id.clone()),
            codelab_id: Some(id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn get_help_requests(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<HelpRequest>>, (StatusCode, String)> {
    session.require_admin()?;
    let requests = sqlx::query_as::<_, HelpRequest>(&state.q(
        "SELECT hr.*, a.name as attendee_name FROM help_requests hr 
         JOIN attendees a ON hr.attendee_id = a.id 
         WHERE hr.codelab_id = ? AND hr.status = 'pending' 
         ORDER BY hr.created_at DESC",
    ))
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(requests))
}

pub async fn resolve_help_request(
    State(state): State<Arc<AppState>>,
    Path((_id, help_id)): Path<(String, String)>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    sqlx::query(&state.q("UPDATE help_requests SET status = 'resolved' WHERE id = ?"))
        .bind(help_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "help_request_resolve".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: None,
            codelab_id: None,
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn complete_codelab(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let attendee = session.require_attendee()?;
    if attendee.codelab_id.as_deref() != Some(id.as_str()) {
        return Err(forbidden());
    }
    let attendee_id = attendee.sub;

    sqlx::query(&state.q("UPDATE attendees SET is_completed = 1, completed_at = CURRENT_TIMESTAMP WHERE id = ? AND codelab_id = ?"))
        .bind(&attendee_id)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codelab_complete".to_string(),
            actor_type: "attendee".to_string(),
            actor_id: Some(attendee_id.clone()),
            target_id: None,
            codelab_id: Some(id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn get_certificate(
    Path(attendee_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<CertificateInfo>, (StatusCode, String)> {
    let row: Option<(String, String, String, String, String, i32)> = sqlx::query_as(&state.q(
        "SELECT a.name as attendee_name, c.title as codelab_title, c.author, 
                COALESCE(a.completed_at, ''), c.id as codelab_id, a.is_completed
         FROM attendees a 
         JOIN codelabs c ON a.codelab_id = c.id 
         WHERE a.id = ?",
    ))
    .bind(&attendee_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    match row {
        Some(r) if r.5 == 1 => Ok(Json(CertificateInfo {
            attendee_name: r.0,
            codelab_title: r.1,
            codelab_id: r.4,
            author: r.2,
            completed_at: r.3,
            verification_url: format!("/verify/{}", attendee_id),
        })),
        Some(_) => Err((StatusCode::FORBIDDEN, "REQUIREMENTS_NOT_MET".to_string())),
        None => Err((StatusCode::NOT_FOUND, "Certificate not found".to_string())),
    }
}
