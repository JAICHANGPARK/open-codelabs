use crate::domain::services::codeserver::CodeServerManager;
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::infrastructure::database::AppState;
use crate::middleware::auth::AuthSession;
use crate::middleware::request_info::RequestInfo;
use crate::utils::error::{bad_request, internal_error};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateCodeServerRequest {
    pub codelab_id: String,
    pub workspace_files: Option<Vec<WorkspaceFile>>,
    pub structure_type: Option<String>, // "branch" or "folder"
}

#[derive(Deserialize, Serialize)]
pub struct WorkspaceFile {
    pub path: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct CodeServerInfo {
    pub path: String,
    pub structure_type: String,
}

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    pub step_number: i32,
    pub branch_type: String, // "start" or "end"
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    pub step_number: i32,
    pub folder_type: String, // "start" or "end"
    pub files: Vec<WorkspaceFile>,
}

#[derive(Deserialize)]
pub struct UpdateWorkspaceFilesRequest {
    pub files: Vec<WorkspaceFile>,
    pub delete_files: Option<Vec<String>>,
    pub commit_message: Option<String>,
}

#[derive(sqlx::FromRow)]
struct WorkspaceRow {
    url: String,
    structure_type: String,
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
    let _codelab = sqlx::query(&state.q("SELECT id FROM codelabs WHERE id = ?"))
        .bind(&payload.codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Codelab not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;

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

    let structure_type = payload.structure_type.as_deref().unwrap_or("branch");

    // Initialize git repository only for branch-based structure
    if structure_type == "branch" {
        manager
            .init_git_repo(&payload.codelab_id)
            .await
            .map_err(internal_error)?;
    }

    let workspace_path = format!("/app/workspaces/{}", payload.codelab_id);

    // Store workspace info in database
    sqlx::query(
        &state.q(
            "INSERT INTO codeserver_workspaces (codelab_id, url, structure_type) VALUES (?, ?, ?)",
        ),
    )
    .bind(&payload.codelab_id)
    .bind(&workspace_path)
    .bind(structure_type)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "workspace_create".to_string(),
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

    Ok(Json(CodeServerInfo {
        path: workspace_path,
        structure_type: structure_type.to_string(),
    }))
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
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Code server workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
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
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Code server workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
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
            format!(
                "attachment; filename=\"codelab-{}-workspace.tar.gz\"",
                codelab_id
            ),
        )
        .body(Body::from(archive))
        .unwrap())
}

/// Get workspace info
pub async fn get_codeserver_info(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<CodeServerInfo>, (StatusCode, String)> {
    session.require_admin()?;

    let workspace = sqlx::query_as::<_, WorkspaceRow>(
        &state.q("SELECT url, structure_type FROM codeserver_workspaces WHERE codelab_id = ?"),
    )
    .bind(&codelab_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| bad_request("Workspace not found"))?;

    Ok(Json(CodeServerInfo {
        path: workspace.url,
        structure_type: workspace.structure_type,
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

    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Code server workspace not found"))?;

    // Remove workspace directory
    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    manager
        .remove_workspace(&codelab_id)
        .await
        .map_err(internal_error)?;

    // Remove from database
    sqlx::query(&state.q("DELETE FROM codeserver_workspaces WHERE codelab_id = ?"))
        .bind(&codelab_id)
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

/// Create a folder for a step (alternative to branches)
pub async fn create_folder(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CreateFolderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;

    // Convert files to tuples
    let files: Vec<(String, String)> = payload
        .files
        .iter()
        .map(|f| (f.path.clone(), f.content.clone()))
        .collect();

    manager
        .create_step_folder(
            &codelab_id,
            payload.step_number,
            &payload.folder_type,
            &files,
        )
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "workspace_folder_create".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(codelab_id.clone()),
            codelab_id: Some(codelab_id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: Some(serde_json::json!({
                "step_number": payload.step_number,
                "folder_type": payload.folder_type
            })),
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// List branches in workspace
pub async fn list_branches(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    let branches = manager
        .list_branches(&codelab_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(branches))
}

/// List files in a branch
pub async fn list_files(
    Path((codelab_id, branch)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    let files = manager
        .list_files(&codelab_id, &branch)
        .await
        .map_err(internal_error)?;

    Ok(Json(files))
}

#[derive(Deserialize)]
pub struct ReadFileQuery {
    pub file: String,
}

/// Read file content from a branch
pub async fn read_file(
    Path((codelab_id, branch)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    axum::extract::Query(query): axum::extract::Query<ReadFileQuery>,
) -> Result<String, (StatusCode, String)> {
    session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    let content = manager
        .read_file(&codelab_id, &branch, &query.file)
        .await
        .map_err(internal_error)?;

    Ok(content)
}

/// List folders in workspace (for folder-based structure)
pub async fn list_folders(
    Path(codelab_id): Path<String>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    let folders = manager
        .list_folders(&codelab_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(folders))
}

/// List files in a folder (for folder-based structure)
pub async fn list_folder_files(
    Path((codelab_id, folder)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    let files = manager
        .list_folder_files(&codelab_id, &folder)
        .await
        .map_err(internal_error)?;

    Ok(Json(files))
}

/// Read file from folder (for folder-based structure)
pub async fn read_folder_file(
    Path((codelab_id, folder)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    axum::extract::Query(query): axum::extract::Query<ReadFileQuery>,
) -> Result<String, (StatusCode, String)> {
    session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    let content = manager
        .read_folder_file(&codelab_id, &folder, &query.file)
        .await
        .map_err(internal_error)?;

    Ok(content)
}

/// Update files in a branch (for branch-based structure)
pub async fn update_branch_files(
    Path((codelab_id, branch)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<UpdateWorkspaceFilesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;
    let current_branch = manager
        .current_branch(&codelab_id)
        .await
        .map_err(internal_error)?;

    manager
        .checkout_branch(&codelab_id, &branch)
        .await
        .map_err(internal_error)?;

    for file in &payload.files {
        manager
            .write_file(&codelab_id, &file.path, &file.content)
            .await
            .map_err(internal_error)?;
    }

    if let Some(deletes) = &payload.delete_files {
        for file in deletes {
            manager
                .remove_path(&codelab_id, file)
                .await
                .map_err(internal_error)?;
        }
    }

    let commit_message = payload
        .commit_message
        .as_deref()
        .unwrap_or("Update workspace files");
    manager
        .commit_changes(&codelab_id, commit_message)
        .await
        .map_err(internal_error)?;

    manager
        .checkout_branch(&codelab_id, &current_branch)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "workspace_branch_update".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(codelab_id.clone()),
            codelab_id: Some(codelab_id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: Some(serde_json::json!({
                "branch": branch,
                "files_updated": payload.files.len(),
                "files_deleted": payload.delete_files.as_ref().map(|v| v.len()).unwrap_or(0)
            })),
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Update files in a folder (for folder-based structure)
pub async fn update_folder_files(
    Path((codelab_id, folder)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<UpdateWorkspaceFilesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;

    // Verify workspace exists
    let _workspace =
        sqlx::query(&state.q("SELECT url FROM codeserver_workspaces WHERE codelab_id = ?"))
            .bind(&codelab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| bad_request("Workspace not found"))?;

    let manager = CodeServerManager::from_env().map_err(internal_error)?;

    for file in &payload.files {
        let path = format!("{}/{}", folder, file.path);
        manager
            .write_file(&codelab_id, &path, &file.content)
            .await
            .map_err(internal_error)?;
    }

    if let Some(deletes) = &payload.delete_files {
        for file in deletes {
            let path = format!("{}/{}", folder, file);
            manager
                .remove_path(&codelab_id, &path)
                .await
                .map_err(internal_error)?;
        }
    }

    record_audit(
        &state,
        AuditEntry {
            action: "workspace_folder_update".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(codelab_id.clone()),
            codelab_id: Some(codelab_id),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: Some(serde_json::json!({
                "folder": folder,
                "files_updated": payload.files.len(),
                "files_deleted": payload.delete_files.as_ref().map(|v| v.len()).unwrap_or(0)
            })),
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "success": true })))
}
