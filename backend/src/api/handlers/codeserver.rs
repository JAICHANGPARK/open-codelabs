use crate::audit::{record_audit, AuditEntry};
use crate::auth::AuthSession;
use crate::codeserver::CodeServerManager;
use crate::error::{bad_request, internal_error};
use crate::request_info::RequestInfo;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateCodeServerRequest {
    pub codelab_id: String,
    pub workspace_files: Option<Vec<WorkspaceFile>>,
}

#[derive(Deserialize, Serialize)]
pub struct WorkspaceFile {
    pub path: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct CodeServerInfo {
    pub url: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    pub step_number: i32,
    pub branch_type: String, // "start" or "end"
}

/// Create workspace in code-server for a codelab
pub async fn create_codeserver(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CreateCodeServerRequest>,
) -> Result<Json<CodeServerInfo>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify codelab exists
    let _codelab = sqlx::query!(&state.q("SELECT id FROM codelabs WHERE id = ?"), payload.codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Codelab not found"))?;

    let manager = CodeServerManager::new().map_err(internal_error)?;

    // Create workspace directory
    manager
        .create_workspace(&payload.codelab_id)
        .await
        .map_err(internal_error)?;

    // Write workspace files if provided
    if let Some(files) = payload.workspace_files {
        for file in files {
            manager
                .write_file(&payload.codelab_id, &file.path, &file.content)
                .await
                .map_err(internal_error)?;
        }
    }

    // Initialize git repository
    manager
        .init_git_repo(&payload.codelab_id)
        .await
        .map_err(internal_error)?;

    let url = manager.get_url().map_err(internal_error)?;
    let password = std::env::var("CODESERVER_PASSWORD")
        .unwrap_or_else(|_| "codelab123".to_string());

    // Store workspace info in database
    sqlx::query!(&state.q(
        "INSERT INTO codeserver_workspaces (codelab_id, url) VALUES (?, ?)"
    ), payload.codelab_id, url)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codeserver_create".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(payload.codelab_id.clone()),
            codelab_id: Some(payload.codelab_id.clone()),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(CodeServerInfo { url, password }))
}

/// Create a git branch for a step
pub async fn create_branch(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CreateBranchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify workspace exists
    let _workspace = sqlx::query!(&state.q(
        "SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"
    ), codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Code server workspace not found"))?;

    let manager = CodeServerManager::new().map_err(internal_error)?;
    manager
        .create_step_branch(&codelab_id, payload.step_number, &payload.branch_type)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codeserver_branch_create".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(codelab_id.clone()),
            codelab_id: Some(codelab_id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: Some(serde_json::json!({
                "step_number": payload.step_number,
                "branch_type": payload.branch_type
            })),
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Download workspace as tar archive
pub async fn download_workspace(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<Response, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify workspace exists
    let _workspace = sqlx::query!(&state.q(
        "SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"
    ), codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Code server workspace not found"))?;

    let manager = CodeServerManager::new().map_err(internal_error)?;
    let archive = manager
        .archive_workspace(&codelab_id)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codeserver_download".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(codelab_id.clone()),
            codelab_id: Some(codelab_id.clone()),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/gzip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"codelab-{}-workspace.tar.gz\"", codelab_id),
        )
        .body(Body::from(archive))
        .unwrap())
}

/// Get code server info
pub async fn get_codeserver_info(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<CodeServerInfo>, (StatusCode, String)> {
    session.require_admin()?;

    let workspace = sqlx::query!(&state.q(
        "SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"
    ), codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Code server workspace not found"))?;

    let password = std::env::var("CODESERVER_PASSWORD")
        .unwrap_or_else(|_| "codelab123".to_string());

    Ok(Json(CodeServerInfo {
        url: workspace.url,
        password,
    }))
}

/// Delete code server workspace
pub async fn delete_codeserver(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    let _workspace = sqlx::query!(&state.q(
        "SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"
    ), codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Code server workspace not found"))?;

    // Remove workspace directory
    let manager = CodeServerManager::new().map_err(internal_error)?;
    manager
        .remove_workspace(&codelab_id)
        .await
        .map_err(internal_error)?;

    // Remove from database
    sqlx::query!(&state.q(
        "DELETE FROM codeserver_workspaces WHERE codelab_id = ?"
    ), codelab_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "codeserver_delete".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(codelab_id.clone()),
            codelab_id: Some(codelab_id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "success": true })))
}
