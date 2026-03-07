//! HTTP client used by the `oclabs` administrative CLI.

use crate::api::dto::{
    CliAuthExchangeRequest, CliAuthExchangeResponse, CliAuthPollResponse, CliAuthStartResponse,
    CliRuntimeInfo, CodeServerInfo, CreateBranchRequest, CreateCodeServerRequest,
    CreateFolderRequest, UpdateWorkspaceFilesRequest, WorkspaceFile,
};
use crate::cli::session::{SessionSnapshot, StoredSession};
use crate::domain::models::{Codelab, CreateCodelab, LoginPayload, Step, UpdateStepsPayload};
use crate::infrastructure::db_models::AuditLog;
use anyhow::{anyhow, bail, Context, Result};
use reqwest::header;
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use url::form_urlencoded::Serializer;

/// Summary returned when inspecting a backup archive.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupSummary {
    /// Backup payload version.
    pub version: u32,
    /// Backup creation timestamp.
    pub created_at: String,
    /// Number of codelabs in the archive.
    pub codelabs: usize,
    /// Number of steps in the archive.
    pub steps: usize,
    /// Number of attendees in the archive.
    pub attendees: usize,
    /// Number of help requests in the archive.
    pub help_requests: usize,
    /// Number of chat messages in the archive.
    pub chat_messages: usize,
    /// Number of feedback rows in the archive.
    pub feedback: usize,
    /// Number of material rows in the archive.
    pub materials: usize,
    /// Number of quiz rows in the archive.
    pub quizzes: usize,
    /// Number of quiz submissions in the archive.
    pub quiz_submissions: usize,
    /// Number of submission rows in the archive.
    pub submissions: usize,
    /// Number of audit log rows in the archive.
    pub audit_logs: usize,
    /// Number of code-server workspace records in the archive.
    pub codeserver_workspaces: usize,
    /// Number of AI conversations in the archive.
    pub ai_conversations: usize,
    /// Number of AI threads in the archive.
    pub ai_threads: usize,
    /// Number of AI messages in the archive.
    pub ai_messages: usize,
    /// Number of inline-comment threads in the archive.
    pub inline_comment_threads: usize,
    /// Number of inline-comment messages in the archive.
    pub inline_comment_messages: usize,
    /// Number of uploaded files found in the ZIP.
    pub uploads_files: usize,
    /// Number of workspace files found in the ZIP.
    pub workspaces_files: usize,
}

/// Update status for a deployed component.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateStatusSummary {
    pub current: Option<String>,
    pub latest: Option<String>,
    pub update_available: bool,
    pub error: Option<String>,
}

/// Combined update summary returned by `/api/admin/updates`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateCheckSummary {
    pub frontend: UpdateStatusSummary,
    pub backend: UpdateStatusSummary,
}

/// Thin wrapper around the backend HTTP API.
pub struct ApiClient {
    base_url: String,
    http: Client,
    session: Option<StoredSession>,
}

impl ApiClient {
    /// Creates a new API client for the provided backend base URL.
    pub fn new(base_url: impl Into<String>, session: Option<StoredSession>) -> Result<Self> {
        let http = Client::builder()
            .no_proxy()
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self {
            base_url: normalize_base_url(&base_url.into()),
            http,
            session,
        })
    }

    /// Authenticates an administrator and returns a persisted session payload.
    pub async fn login_admin(&self, admin_id: &str, admin_pw: &str) -> Result<StoredSession> {
        let response = self
            .http
            .post(self.url("/api/login"))
            .json(&LoginPayload {
                admin_id: admin_id.to_string(),
                admin_pw: admin_pw.to_string(),
            })
            .send()
            .await
            .context("Failed to call /api/login")?;

        let mut session =
            build_session_from_response(response, "/api/login", &self.base_url).await?;
        let snapshot = self.fetch_session_with(&session).await?;
        session.apply_snapshot(&snapshot);
        Ok(session)
    }

    /// Returns the latest session claims for the active CLI session.
    pub async fn session(&self) -> Result<SessionSnapshot> {
        let session = self.require_session()?;
        self.fetch_session_with(session).await
    }

    /// Returns runtime and capability metadata for the connected server.
    pub async fn cli_runtime(&self) -> Result<CliRuntimeInfo> {
        let response = self
            .http
            .get(self.url("/api/cli/runtime"))
            .send()
            .await
            .context("Failed to call /api/cli/runtime")?;
        read_json(response, "/api/cli/runtime").await
    }

    /// Starts a browser-based CLI authentication flow.
    pub async fn start_browser_auth(&self) -> Result<CliAuthStartResponse> {
        let response = self
            .http
            .post(self.url("/api/cli/auth/start"))
            .send()
            .await
            .context("Failed to call /api/cli/auth/start")?;
        read_json(response, "/api/cli/auth/start").await
    }

    /// Polls the approval state of a browser-based CLI authentication flow.
    pub async fn poll_browser_auth(
        &self,
        request_id: &str,
        poll_token: &str,
    ) -> Result<CliAuthPollResponse> {
        let mut serializer = Serializer::new(String::new());
        serializer.append_pair("poll_token", poll_token);
        let path = format!("/api/cli/auth/poll/{request_id}?{}", serializer.finish());
        let response = self
            .http
            .get(self.url(&path))
            .send()
            .await
            .with_context(|| format!("Failed to call {path}"))?;
        read_json(response, "/api/cli/auth/poll/{id}").await
    }

    /// Exchanges an approved CLI browser auth challenge for a stored session.
    pub async fn exchange_browser_auth(
        &self,
        request_id: &str,
        poll_token: &str,
    ) -> Result<StoredSession> {
        let response = self
            .http
            .post(self.url(&format!("/api/cli/auth/exchange/{request_id}")))
            .json(&CliAuthExchangeRequest {
                poll_token: poll_token.to_string(),
            })
            .send()
            .await
            .with_context(|| format!("Failed to call /api/cli/auth/exchange/{request_id}"))?;
        let (mut session, snapshot) = read_session_json::<CliAuthExchangeResponse>(
            response,
            "/api/cli/auth/exchange/{id}",
            &self.base_url,
        )
        .await?;
        session.apply_snapshot(&SessionSnapshot {
            sub: snapshot.sub,
            role: snapshot.role,
            codelab_id: snapshot.codelab_id,
            exp: snapshot.exp,
        });
        Ok(session)
    }

    /// Invokes the backend logout route using the active CLI session.
    pub async fn logout(&self) -> Result<()> {
        let session = self.require_session()?;
        let response = self
            .send_with_session(Method::POST, "/api/logout", None, session)
            .await?;
        ensure_success(response, "/api/logout").await?;
        Ok(())
    }

    /// Lists codelabs visible to the current session.
    pub async fn list_codelabs(&self) -> Result<Vec<Codelab>> {
        self.send_authed_json(Method::GET, "/api/codelabs", None)
            .await
    }

    /// Fetches a single codelab and its ordered steps.
    pub async fn get_codelab(&self, id: &str) -> Result<(Codelab, Vec<Step>)> {
        self.send_authed_json(Method::GET, &format!("/api/codelabs/{id}"), None)
            .await
    }

    /// Creates a new codelab.
    pub async fn create_codelab(&self, payload: &CreateCodelab) -> Result<Codelab> {
        self.send_authed_json(
            Method::POST,
            "/api/codelabs",
            Some(serde_json::to_value(payload).context("serialize create payload")?),
        )
        .await
    }

    /// Updates existing codelab metadata.
    pub async fn update_codelab(&self, id: &str, payload: &CreateCodelab) -> Result<Codelab> {
        self.send_authed_json(
            Method::PUT,
            &format!("/api/codelabs/{id}"),
            Some(serde_json::to_value(payload).context("serialize update payload")?),
        )
        .await
    }

    /// Deletes a codelab and all related data.
    pub async fn delete_codelab(&self, id: &str) -> Result<()> {
        let response = self
            .send_authed(Method::DELETE, &format!("/api/codelabs/{id}"), None)
            .await?;
        ensure_success(response, "/api/codelabs/{id}").await?;
        Ok(())
    }

    /// Copies a codelab including its steps.
    pub async fn copy_codelab(&self, id: &str) -> Result<Codelab> {
        self.send_authed_json(Method::POST, &format!("/api/codelabs/{id}/copy"), None)
            .await
    }

    /// Uploads a replacement step list from a JSON payload.
    pub async fn push_steps(&self, id: &str, payload: &UpdateStepsPayload) -> Result<()> {
        let response = self
            .send_authed(
                Method::PUT,
                &format!("/api/codelabs/{id}/steps"),
                Some(serde_json::to_value(payload).context("serialize steps payload")?),
            )
            .await?;
        ensure_success(response, "/api/codelabs/{id}/steps").await?;
        Ok(())
    }

    /// Imports a codelab ZIP archive.
    pub async fn import_codelab(&self, file_path: &Path) -> Result<Codelab> {
        let form = file_form(file_path, "application/zip").await?;
        self.send_authed_multipart("/api/codelabs/import", form)
            .await
    }

    /// Exports a codelab ZIP archive.
    pub async fn export_codelab(&self, id: &str) -> Result<Vec<u8>> {
        self.send_authed_bytes(Method::GET, &format!("/api/codelabs/{id}/export"), None)
            .await
    }

    /// Exports a full platform backup archive.
    pub async fn export_backup(&self) -> Result<Vec<u8>> {
        self.send_authed_bytes(Method::GET, "/api/admin/backup/export", None)
            .await
    }

    /// Updates encrypted administrator settings.
    pub async fn save_admin_settings(&self, gemini_api_key: &str) -> Result<()> {
        let response = self
            .send_authed(
                Method::POST,
                "/api/admin/settings",
                Some(serde_json::json!({ "gemini_api_key": gemini_api_key })),
            )
            .await?;
        ensure_success(response, "/api/admin/settings").await?;
        Ok(())
    }

    /// Checks frontend/backend image versions.
    pub async fn check_updates(&self) -> Result<UpdateCheckSummary> {
        self.send_authed_json(Method::GET, "/api/admin/updates", None)
            .await
    }

    /// Creates a workspace for a codelab.
    pub async fn create_workspace(
        &self,
        payload: &CreateCodeServerRequest,
    ) -> Result<CodeServerInfo> {
        self.send_authed_json(
            Method::POST,
            "/api/codeserver",
            Some(serde_json::to_value(payload).context("serialize workspace payload")?),
        )
        .await
    }

    /// Fetches workspace metadata for a codelab.
    pub async fn workspace_info(&self, codelab_id: &str) -> Result<CodeServerInfo> {
        self.send_authed_json(Method::GET, &format!("/api/codeserver/{codelab_id}"), None)
            .await
    }

    /// Downloads a workspace archive for a codelab.
    pub async fn download_workspace(&self, codelab_id: &str) -> Result<Vec<u8>> {
        self.send_authed_bytes(
            Method::GET,
            &format!("/api/codeserver/{codelab_id}/download"),
            None,
        )
        .await
    }

    /// Deletes a workspace for a codelab.
    pub async fn delete_workspace(&self, codelab_id: &str) -> Result<()> {
        let response = self
            .send_authed(
                Method::DELETE,
                &format!("/api/codeserver/{codelab_id}"),
                None,
            )
            .await?;
        ensure_success(response, "/api/codeserver/{codelab_id}").await?;
        Ok(())
    }

    /// Lists branch snapshots for a codelab workspace.
    pub async fn list_workspace_branches(&self, codelab_id: &str) -> Result<Vec<String>> {
        self.send_authed_json(
            Method::GET,
            &format!("/api/codeserver/{codelab_id}/branches"),
            None,
        )
        .await
    }

    /// Creates a branch snapshot for a codelab workspace.
    pub async fn create_workspace_branch(
        &self,
        codelab_id: &str,
        step_number: i32,
        branch_type: &str,
    ) -> Result<()> {
        let response = self
            .send_authed(
                Method::POST,
                &format!("/api/codeserver/{codelab_id}/branch"),
                Some(
                    serde_json::to_value(CreateBranchRequest {
                        step_number,
                        branch_type: branch_type.to_string(),
                    })
                    .context("serialize branch payload")?,
                ),
            )
            .await?;
        ensure_success(response, "/api/codeserver/{codelab_id}/branch").await?;
        Ok(())
    }

    /// Lists folder snapshots for a codelab workspace.
    pub async fn list_workspace_folders(&self, codelab_id: &str) -> Result<Vec<String>> {
        self.send_authed_json(
            Method::GET,
            &format!("/api/codeserver/{codelab_id}/folders"),
            None,
        )
        .await
    }

    /// Creates a folder snapshot for a codelab workspace.
    pub async fn create_workspace_folder(
        &self,
        codelab_id: &str,
        step_number: i32,
        folder_type: &str,
        files: Vec<WorkspaceFile>,
    ) -> Result<()> {
        let response = self
            .send_authed(
                Method::POST,
                &format!("/api/codeserver/{codelab_id}/folder"),
                Some(
                    serde_json::to_value(CreateFolderRequest {
                        step_number,
                        folder_type: folder_type.to_string(),
                        files,
                    })
                    .context("serialize folder payload")?,
                ),
            )
            .await?;
        ensure_success(response, "/api/codeserver/{codelab_id}/folder").await?;
        Ok(())
    }

    /// Lists files in a workspace branch.
    pub async fn list_workspace_files(
        &self,
        codelab_id: &str,
        branch: &str,
    ) -> Result<Vec<String>> {
        self.send_authed_json(
            Method::GET,
            &format!("/api/codeserver/{codelab_id}/branches/{branch}/files"),
            None,
        )
        .await
    }

    /// Reads a workspace file from a branch.
    pub async fn read_workspace_file(
        &self,
        codelab_id: &str,
        branch: &str,
        file: &str,
    ) -> Result<String> {
        let mut serializer = Serializer::new(String::new());
        serializer.append_pair("file", file);
        let path = format!(
            "/api/codeserver/{codelab_id}/branches/{branch}/file?{}",
            serializer.finish()
        );
        let response = self.send_authed(Method::GET, &path, None).await?;
        let response = ensure_success(
            response,
            "/api/codeserver/{codelab_id}/branches/{branch}/file",
        )
        .await?;
        response
            .text()
            .await
            .with_context(|| format!("Failed to read response body from {path}"))
    }

    /// Updates files in a workspace branch.
    pub async fn update_workspace_branch_files(
        &self,
        codelab_id: &str,
        branch: &str,
        payload: &UpdateWorkspaceFilesRequest,
    ) -> Result<()> {
        let response = self
            .send_authed(
                Method::POST,
                &format!("/api/codeserver/{codelab_id}/branches/{branch}/files"),
                Some(serde_json::to_value(payload).context("serialize branch file payload")?),
            )
            .await?;
        ensure_success(
            response,
            "/api/codeserver/{codelab_id}/branches/{branch}/files",
        )
        .await?;
        Ok(())
    }

    /// Lists files in a workspace folder snapshot.
    pub async fn list_workspace_folder_files(
        &self,
        codelab_id: &str,
        folder: &str,
    ) -> Result<Vec<String>> {
        self.send_authed_json(
            Method::GET,
            &format!("/api/codeserver/{codelab_id}/folders/{folder}/files"),
            None,
        )
        .await
    }

    /// Reads a workspace file from a folder snapshot.
    pub async fn read_workspace_folder_file(
        &self,
        codelab_id: &str,
        folder: &str,
        file: &str,
    ) -> Result<String> {
        let mut serializer = Serializer::new(String::new());
        serializer.append_pair("file", file);
        let path = format!(
            "/api/codeserver/{codelab_id}/folders/{folder}/file?{}",
            serializer.finish()
        );
        let response = self.send_authed(Method::GET, &path, None).await?;
        let response = ensure_success(
            response,
            "/api/codeserver/{codelab_id}/folders/{folder}/file",
        )
        .await?;
        response
            .text()
            .await
            .with_context(|| format!("Failed to read response body from {path}"))
    }

    /// Updates files inside a folder snapshot.
    pub async fn update_workspace_folder_files(
        &self,
        codelab_id: &str,
        folder: &str,
        payload: &UpdateWorkspaceFilesRequest,
    ) -> Result<()> {
        let response = self
            .send_authed(
                Method::POST,
                &format!("/api/codeserver/{codelab_id}/folders/{folder}/files"),
                Some(serde_json::to_value(payload).context("serialize folder file payload")?),
            )
            .await?;
        ensure_success(
            response,
            "/api/codeserver/{codelab_id}/folders/{folder}/files",
        )
        .await?;
        Ok(())
    }

    /// Inspects a backup archive without restoring it.
    pub async fn inspect_backup(&self, file_path: &Path) -> Result<BackupSummary> {
        let form = file_form(file_path, "application/zip").await?;
        self.send_authed_multipart("/api/admin/backup/inspect", form)
            .await
    }

    /// Restores a backup archive into the active backend.
    pub async fn restore_backup(&self, file_path: &Path) -> Result<()> {
        let form = file_form(file_path, "application/zip").await?;
        let response = self
            .send_with_session_multipart("/api/admin/backup/restore", form, self.require_session()?)
            .await?;
        ensure_success(response, "/api/admin/backup/restore").await?;
        Ok(())
    }

    /// Returns audit logs with optional filters.
    pub async fn audit_logs(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        action: Option<&str>,
        codelab_id: Option<&str>,
    ) -> Result<Vec<AuditLog>> {
        let mut serializer = Serializer::new(String::new());
        if let Some(limit) = limit {
            serializer.append_pair("limit", &limit.to_string());
        }
        if let Some(offset) = offset {
            serializer.append_pair("offset", &offset.to_string());
        }
        if let Some(action) = action {
            serializer.append_pair("action", action);
        }
        if let Some(codelab_id) = codelab_id {
            serializer.append_pair("codelab_id", codelab_id);
        }
        let query = serializer.finish();
        let mut path = "/api/admin/audit-logs".to_string();
        if !query.is_empty() {
            path.push('?');
            path.push_str(&query);
        }
        self.send_authed_json(Method::GET, &path, None).await
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn require_session(&self) -> Result<&StoredSession> {
        self.session
            .as_ref()
            .ok_or_else(|| anyhow!("No saved session. Run `oclabs login` first."))
    }

    async fn fetch_session_with(&self, session: &StoredSession) -> Result<SessionSnapshot> {
        let response = self
            .send_with_session(Method::GET, "/api/session", None, session)
            .await?;
        read_json(response, "/api/session").await
    }

    async fn send_authed_json<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<T> {
        let response = self.send_authed(method, path, body).await?;
        read_json(response, path).await
    }

    async fn send_authed_bytes(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Vec<u8>> {
        let response = self.send_authed(method, path, body).await?;
        read_bytes(response, path).await
    }

    async fn send_authed_multipart<T: DeserializeOwned>(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<T> {
        let response = self
            .send_with_session_multipart(path, form, self.require_session()?)
            .await?;
        read_json(response, path).await
    }

    async fn send_authed(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Response> {
        let session = self.require_session()?;
        self.send_with_session(method, path, body, session).await
    }

    async fn send_with_session(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        session: &StoredSession,
    ) -> Result<Response> {
        let mut request = self
            .http
            .request(method.clone(), self.url(path))
            .header(header::COOKIE, session.cookie_header.clone());

        if !matches!(method, Method::GET | Method::HEAD | Method::OPTIONS)
            && session.csrf_token.is_some()
        {
            request = request.header(
                "x-csrf-token",
                session.csrf_token.as_deref().unwrap_or_default(),
            );
        }

        if let Some(body) = body {
            request = request
                .header(header::CONTENT_TYPE, "application/json")
                .json(&body);
        }

        request
            .send()
            .await
            .with_context(|| format!("Request failed: {path}"))
    }

    async fn send_with_session_multipart(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
        session: &StoredSession,
    ) -> Result<Response> {
        let mut request = self
            .http
            .post(self.url(path))
            .header(header::COOKIE, session.cookie_header.clone());

        if let Some(csrf) = &session.csrf_token {
            request = request.header("x-csrf-token", csrf);
        }

        request
            .multipart(form)
            .send()
            .await
            .with_context(|| format!("Request failed: {path}"))
    }
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_string()
}

async fn file_form(file_path: &Path, mime_type: &str) -> Result<reqwest::multipart::Form> {
    let file_name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("Invalid file path {}", file_path.display()))?
        .to_string();
    let bytes = tokio::fs::read(file_path)
        .await
        .with_context(|| format!("Failed to read {}", file_path.display()))?;
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str(mime_type)
        .context("Failed to create multipart body")?;
    Ok(reqwest::multipart::Form::new().part("file", part))
}

async fn ensure_success(response: Response, path: &str) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().await.unwrap_or_default();
    bail!(
        "{} failed: HTTP {} body={}",
        path,
        status.as_u16(),
        truncate(&body, 200)
    );
}

async fn read_json<T: DeserializeOwned>(response: Response, path: &str) -> Result<T> {
    let response = ensure_success(response, path).await?;
    response
        .json::<T>()
        .await
        .with_context(|| format!("Failed to parse JSON response from {path}"))
}

async fn read_bytes(response: Response, path: &str) -> Result<Vec<u8>> {
    let response = ensure_success(response, path).await?;
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .with_context(|| format!("Failed to read response body from {path}"))
}

async fn build_session_from_response(
    response: Response,
    context_path: &str,
    base_url: &str,
) -> Result<StoredSession> {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        bail!(
            "{} failed: HTTP {} body={}",
            context_path,
            status.as_u16(),
            truncate(&body, 200)
        );
    }

    let cookies = parse_set_cookie_headers(&headers);
    if cookies.is_empty() {
        bail!("No Set-Cookie headers from {context_path}");
    }

    Ok(StoredSession {
        base_url: base_url.to_string(),
        cookie_header: build_cookie_header(&cookies),
        csrf_token: find_csrf_token(&cookies),
        sub: None,
        role: None,
        exp: None,
        codelab_id: None,
    })
}

async fn read_session_json<T: DeserializeOwned>(
    response: Response,
    context_path: &str,
    base_url: &str,
) -> Result<(StoredSession, T)> {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        bail!(
            "{} failed: HTTP {} body={}",
            context_path,
            status.as_u16(),
            truncate(&body, 200)
        );
    }

    let cookies = parse_set_cookie_headers(&headers);
    if cookies.is_empty() {
        bail!("No Set-Cookie headers from {context_path}");
    }

    let session = StoredSession {
        base_url: base_url.to_string(),
        cookie_header: build_cookie_header(&cookies),
        csrf_token: find_csrf_token(&cookies),
        sub: None,
        role: None,
        exp: None,
        codelab_id: None,
    };
    let payload = serde_json::from_str::<T>(&body)
        .with_context(|| format!("Failed to parse JSON response from {context_path}"))?;
    Ok((session, payload))
}

fn parse_set_cookie_headers(headers: &header::HeaderMap) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for value in headers.get_all(header::SET_COOKIE) {
        let Ok(raw) = value.to_str() else {
            continue;
        };
        let Some(first_pair) = raw.split(';').next() else {
            continue;
        };
        let Some((name, value)) = first_pair.split_once('=') else {
            continue;
        };
        let name = name.trim();
        let value = value.trim();
        if !name.is_empty() {
            cookies.insert(name.to_string(), value.to_string());
        }
    }
    cookies
}

fn build_cookie_header(cookies: &HashMap<String, String>) -> String {
    let mut pairs = cookies
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>();
    pairs.sort();
    pairs.join("; ")
}

fn find_csrf_token(cookies: &HashMap<String, String>) -> Option<String> {
    cookies
        .iter()
        .find(|(name, _)| name.ends_with("oc_csrf"))
        .map(|(_, value)| value.clone())
}

fn truncate(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    let mut truncated = value.chars().take(max_len).collect::<String>();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_set_cookie_headers_extracts_first_pair() {
        let mut headers = header::HeaderMap::new();
        headers.append(
            header::SET_COOKIE,
            "session=abc; Path=/; HttpOnly".parse().expect("header"),
        );
        headers.append(
            header::SET_COOKIE,
            "__Host-oc_csrf=def; Path=/".parse().expect("header"),
        );

        let cookies = parse_set_cookie_headers(&headers);
        assert_eq!(cookies.get("session"), Some(&"abc".to_string()));
        assert_eq!(cookies.get("__Host-oc_csrf"), Some(&"def".to_string()));
    }
}
