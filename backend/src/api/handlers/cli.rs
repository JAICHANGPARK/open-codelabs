use crate::api::dto::{
    CliAuthApproveRequest, CliAuthApproveResponse, CliAuthExchangeRequest, CliAuthExchangeResponse,
    CliAuthPollQuery, CliAuthPollResponse, CliAuthStartResponse, CliRuntimeCapabilities,
    CliRuntimeInfo,
};
use crate::api::handlers::admin::{issue_admin_session, validate_admin_credentials};
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::infrastructure::database::AppState;
use crate::middleware::auth::{now_epoch_seconds, AuthSession};
use crate::middleware::request_info::RequestInfo;
use crate::middleware::security::ensure_csrf_cookie;
use crate::utils::error::{bad_request, internal_error, unauthorized};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

const CLI_AUTH_TTL_SECONDS: i64 = 10 * 60;
const CLI_AUTH_POLL_INTERVAL_SECONDS: u64 = 2;

#[derive(Debug, Clone)]
struct CliAuthRequestRow {
    status: String,
    poll_token_hash: String,
    expires_at_epoch: i64,
}

/// Returns runtime metadata used by the standalone CLI to probe server support.
pub async fn get_cli_runtime() -> Json<CliRuntimeInfo> {
    Json(build_cli_runtime_info())
}

/// Starts a browser-based CLI authentication challenge.
pub async fn start_cli_auth(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CliAuthStartResponse>, (StatusCode, String)> {
    cleanup_cli_auth_requests(&state)
        .await
        .map_err(internal_error)?;

    let request_id = Uuid::new_v4().to_string();
    let poll_token = generate_poll_token();
    let now = now_epoch_seconds() as i64;
    let expires_at_epoch = now + CLI_AUTH_TTL_SECONDS;

    sqlx::query(&state.q(
        "INSERT INTO cli_auth_requests (id, poll_token_hash, status, created_at_epoch, expires_at_epoch)
         VALUES (?, ?, 'pending', ?, ?)",
    ))
    .bind(&request_id)
    .bind(hash_poll_token(&poll_token))
    .bind(now)
    .bind(expires_at_epoch)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(CliAuthStartResponse {
        request_id: request_id.clone(),
        poll_token,
        verification_path: format!("/cli/auth/{request_id}"),
        expires_at_epoch,
        poll_interval_seconds: CLI_AUTH_POLL_INTERVAL_SECONDS,
    }))
}

/// Polls the current state of a browser-based CLI authentication challenge.
pub async fn poll_cli_auth(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
    Query(query): Query<CliAuthPollQuery>,
) -> Result<Json<CliAuthPollResponse>, (StatusCode, String)> {
    let challenge = load_cli_auth_request(&state, &request_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Unknown CLI auth request"))?;

    if challenge.poll_token_hash != hash_poll_token(&query.poll_token) {
        return Err(unauthorized());
    }

    let now = now_epoch_seconds() as i64;
    let status = if challenge.status == "pending" && challenge.expires_at_epoch <= now {
        "expired".to_string()
    } else {
        challenge.status
    };

    Ok(Json(CliAuthPollResponse { status }))
}

/// Exchanges an approved CLI authentication challenge for admin session cookies.
pub async fn exchange_cli_auth(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
    jar: CookieJar,
    Json(payload): Json<CliAuthExchangeRequest>,
) -> Result<(CookieJar, Json<CliAuthExchangeResponse>), (StatusCode, String)> {
    let challenge = load_cli_auth_request(&state, &request_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Unknown CLI auth request"))?;

    if challenge.poll_token_hash != hash_poll_token(&payload.poll_token) {
        return Err(unauthorized());
    }

    let now = now_epoch_seconds() as i64;
    if challenge.expires_at_epoch <= now {
        return Err(bad_request("CLI auth request expired"));
    }
    if challenge.status != "approved" {
        return Err(bad_request("CLI auth request is not approved yet"));
    }

    let updated = sqlx::query(&state.q("UPDATE cli_auth_requests
         SET status = 'consumed', consumed_at_epoch = ?
         WHERE id = ? AND status = 'approved'"))
    .bind(now)
    .bind(&request_id)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    if updated.rows_affected() == 0 {
        return Err(bad_request("CLI auth request has already been exchanged"));
    }

    let (jar, claims, _token) = issue_admin_session(&state, jar).map_err(internal_error)?;
    Ok((
        jar,
        Json(CliAuthExchangeResponse {
            sub: claims.sub,
            role: claims.role,
            codelab_id: claims.codelab_id,
            exp: claims.exp,
        }),
    ))
}

/// Approves a CLI authentication challenge from the browser.
pub async fn approve_cli_auth(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CliAuthApproveRequest>,
) -> Result<Json<CliAuthApproveResponse>, (StatusCode, String)> {
    let challenge = load_cli_auth_request(&state, &request_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| bad_request("Unknown CLI auth request"))?;

    let now = now_epoch_seconds() as i64;
    if challenge.expires_at_epoch <= now {
        return Err(bad_request("CLI auth request expired"));
    }

    if challenge.status == "approved" {
        return Ok(Json(CliAuthApproveResponse {
            status: "approved".to_string(),
        }));
    }
    if challenge.status == "consumed" {
        return Err(bad_request("CLI auth request has already been exchanged"));
    }

    let actor_id = if let Ok(claims) = session.require_admin() {
        claims.sub
    } else {
        if payload.admin_id.trim().is_empty() || payload.admin_pw.trim().is_empty() {
            return Err(bad_request("admin_id and admin_pw are required"));
        }

        if !validate_admin_credentials(&state, payload.admin_id.trim(), payload.admin_pw.trim()) {
            record_audit(
                &state,
                AuditEntry {
                    action: "cli_auth_failed".to_string(),
                    actor_type: "admin".to_string(),
                    actor_id: Some(payload.admin_id),
                    target_id: Some(request_id.clone()),
                    codelab_id: None,
                    ip: Some(info.ip),
                    user_agent: info.user_agent,
                    metadata: None,
                },
            )
            .await;
            return Err(unauthorized());
        }

        state.admin_id.clone()
    };

    let updated = sqlx::query(&state.q("UPDATE cli_auth_requests
         SET status = 'approved', approved_at_epoch = ?, approved_by = ?
         WHERE id = ? AND status = 'pending'"))
    .bind(now)
    .bind(&actor_id)
    .bind(&request_id)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    if updated.rows_affected() == 0 {
        return Err(bad_request("CLI auth request is no longer pending"));
    }

    record_audit(
        &state,
        AuditEntry {
            action: "cli_auth_approved".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(actor_id),
            target_id: Some(request_id),
            codelab_id: None,
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(CliAuthApproveResponse {
        status: "approved".to_string(),
    }))
}

/// Serves the lightweight browser page used to approve CLI authentication.
pub async fn cli_auth_page(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
    jar: CookieJar,
    session: AuthSession,
) -> (StatusCode, CookieJar, Html<String>) {
    let now = now_epoch_seconds() as i64;
    let response = match load_cli_auth_request(&state, &request_id).await {
        Ok(Some(challenge)) => {
            let status = if challenge.status == "pending" && challenge.expires_at_epoch <= now {
                "expired".to_string()
            } else {
                challenge.status
            };
            let has_admin_session = session.require_admin().is_ok();
            let jar = if has_admin_session {
                ensure_csrf_cookie(jar, &state, state.auth.admin_ttl)
            } else {
                jar
            };
            (
                StatusCode::OK,
                jar,
                Html(render_cli_auth_page(
                    &request_id,
                    &status,
                    challenge.expires_at_epoch,
                    has_admin_session,
                )),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            jar,
            Html(render_cli_auth_page(&request_id, "missing", now, false)),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            jar,
            Html(render_cli_auth_page(
                &request_id,
                &format!("error:{error}"),
                now,
                false,
            )),
        ),
    };

    response
}

fn build_cli_runtime_info() -> CliRuntimeInfo {
    CliRuntimeInfo {
        runtime: "backend".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        auth_methods: vec!["browser".to_string(), "password".to_string()],
        capabilities: CliRuntimeCapabilities {
            admin_api: true,
            backup: true,
            workspace: true,
            audit: true,
            browser_auth: true,
        },
    }
}

fn generate_poll_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

fn hash_poll_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn cleanup_cli_auth_requests(state: &AppState) -> Result<(), sqlx::Error> {
    let now = now_epoch_seconds() as i64;
    sqlx::query(&state.q("DELETE FROM cli_auth_requests
         WHERE expires_at_epoch <= ? OR status = 'consumed'"))
    .bind(now)
    .execute(&state.pool)
    .await?;
    Ok(())
}

async fn load_cli_auth_request(
    state: &AppState,
    request_id: &str,
) -> Result<Option<CliAuthRequestRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, i64)>(&state.q(
        "SELECT status, poll_token_hash, expires_at_epoch
         FROM cli_auth_requests
         WHERE id = ?",
    ))
    .bind(request_id)
    .fetch_optional(&state.pool)
    .await?;

    Ok(row.map(
        |(status, poll_token_hash, expires_at_epoch)| CliAuthRequestRow {
            status,
            poll_token_hash,
            expires_at_epoch,
        },
    ))
}

fn render_cli_auth_page(
    request_id: &str,
    status: &str,
    expires_at_epoch: i64,
    has_admin_session: bool,
) -> String {
    let title = match status {
        "approved" => "CLI login approved",
        "consumed" => "CLI login complete",
        "expired" => "CLI login expired",
        "missing" => "CLI login not found",
        value if value.starts_with("error:") => "CLI login error",
        _ => "Approve CLI login",
    };
    let status_message = match status {
        "approved" => {
            "Approval has been recorded. Return to the terminal to finish authentication."
        }
        "consumed" => "This CLI login request has already been exchanged.",
        "expired" => "This CLI login request has expired. Start a new `oc auth login` session.",
        "missing" => "This CLI login request is no longer available.",
        value if value.starts_with("error:") => "The server could not load this CLI login request.",
        _ if has_admin_session => {
            "Your browser already has an admin session. Click approve to continue."
        }
        _ => "Sign in below to approve the CLI login request.",
    };
    let form_hidden = if has_admin_session || status != "pending" {
        "hidden"
    } else {
        ""
    };
    let approve_hidden = if !has_admin_session || status != "pending" {
        "hidden"
    } else {
        ""
    };
    let action_disabled = if status == "pending" { "" } else { "disabled" };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #f5f3ef;
      --card: #fffdf8;
      --ink: #1d1b18;
      --muted: #6a6258;
      --accent: #14532d;
      --accent-strong: #0f3f22;
      --border: #d7cfc4;
      --danger: #8a1c1c;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      min-height: 100vh;
      font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #fff8e8, var(--bg) 55%);
      color: var(--ink);
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 24px;
    }}
    .card {{
      width: min(100%, 440px);
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 24px;
      padding: 28px;
      box-shadow: 0 20px 60px rgba(29, 27, 24, 0.12);
    }}
    h1 {{ margin: 0 0 8px; font-size: 28px; line-height: 1.1; }}
    p {{ margin: 0 0 20px; color: var(--muted); line-height: 1.5; }}
    .meta {{
      margin: 0 0 20px;
      padding: 12px 14px;
      border-radius: 14px;
      background: #faf5ec;
      color: var(--muted);
      font-size: 14px;
    }}
    label {{
      display: block;
      font-size: 13px;
      font-weight: 600;
      margin-bottom: 6px;
    }}
    input {{
      width: 100%;
      padding: 12px 14px;
      border: 1px solid var(--border);
      border-radius: 14px;
      font-size: 15px;
      margin-bottom: 14px;
      background: #fff;
    }}
    button {{
      width: 100%;
      border: 0;
      border-radius: 14px;
      padding: 13px 16px;
      font-size: 15px;
      font-weight: 700;
      cursor: pointer;
      color: #fff;
      background: var(--accent);
    }}
    button:hover {{ background: var(--accent-strong); }}
    button[disabled] {{
      opacity: 0.55;
      cursor: not-allowed;
    }}
    .ghost {{
      background: #ece7dd;
      color: var(--ink);
    }}
    .hidden {{ display: none; }}
    .status {{
      margin-top: 14px;
      min-height: 24px;
      font-size: 14px;
      color: var(--muted);
    }}
    .status.error {{ color: var(--danger); }}
    .status.success {{ color: var(--accent); }}
  </style>
</head>
<body>
  <main class="card">
    <h1>{title}</h1>
    <p>{status_message}</p>
    <div class="meta">
      Request: <code>{request_id}</code><br>
      Expires at epoch: <code>{expires_at_epoch}</code>
    </div>

    <section id="login-form" class="{form_hidden}">
      <label for="admin_id">Admin ID</label>
      <input id="admin_id" autocomplete="username" {action_disabled}>
      <label for="admin_pw">Password</label>
      <input id="admin_pw" type="password" autocomplete="current-password" {action_disabled}>
      <button id="login-button" {action_disabled}>Sign In And Approve</button>
    </section>

    <section id="approve-panel" class="{approve_hidden}">
      <button id="approve-button" {action_disabled}>Approve CLI Login</button>
    </section>

    <div id="status" class="status"></div>
  </main>

  <script>
    const requestId = {request_id:?};
    const statusValue = {status:?};

    function findCsrfToken() {{
      const parts = document.cookie.split(";").map((part) => part.trim()).filter(Boolean);
      for (const part of parts) {{
        const [name, ...rest] = part.split("=");
        if (name.endsWith("oc_csrf")) {{
          return decodeURIComponent(rest.join("="));
        }}
      }}
      return "";
    }}

    async function approve(payload) {{
      const statusEl = document.getElementById("status");
      statusEl.className = "status";
      statusEl.textContent = "Waiting for server...";
      const response = await fetch(`/api/cli/auth/approve/${{requestId}}`, {{
        method: "POST",
        credentials: "same-origin",
        headers: {{
          "content-type": "application/json",
          "x-csrf-token": findCsrfToken(),
        }},
        body: JSON.stringify(payload),
      }});

      if (!response.ok) {{
        const text = await response.text();
        statusEl.className = "status error";
        statusEl.textContent = text || "Approval failed.";
        return;
      }}

      statusEl.className = "status success";
      statusEl.textContent = "Approved. Return to the terminal to finish login.";
      document.getElementById("login-form")?.classList.add("hidden");
      document.getElementById("approve-panel")?.classList.add("hidden");
    }}

    if (statusValue === "pending") {{
      document.getElementById("login-button")?.addEventListener("click", async () => {{
        const adminId = document.getElementById("admin_id").value;
        const adminPw = document.getElementById("admin_pw").value;
        await approve({{ admin_id: adminId, admin_pw: adminPw }});
      }});
      document.getElementById("approve-button")?.addEventListener("click", async () => {{
        await approve({{}});
      }});
    }}
  </script>
</body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_router;
    use crate::infrastructure::database::{run_migrations, AppState, DbKind};
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use sqlx::any::{install_default_drivers, AnyPoolOptions};
    use tower::util::ServiceExt;

    async fn make_app() -> axum::Router {
        install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("sqlite");
        run_migrations(&pool, DbKind::Sqlite)
            .await
            .expect("migrations");
        let state = Arc::new(AppState::new(
            pool,
            DbKind::Sqlite,
            "admin".to_string(),
            "pw".to_string(),
            false,
        ));
        create_router(state)
    }

    #[tokio::test]
    async fn runtime_probe_returns_backend_capabilities() {
        let app = make_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/cli/runtime")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: CliRuntimeInfo = serde_json::from_slice(&body).expect("json");
        assert_eq!(payload.runtime, "backend");
        assert!(payload.capabilities.browser_auth);
    }

    #[tokio::test]
    async fn browser_auth_flow_round_trips() {
        let app = make_app().await;

        let start_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/cli/auth/start")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(start_response.status(), StatusCode::OK);
        let start_body = to_bytes(start_response.into_body(), usize::MAX)
            .await
            .expect("body");
        let start: CliAuthStartResponse = serde_json::from_slice(&start_body).expect("json");

        let approve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/cli/auth/approve/{}", start.request_id))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "admin_id": "admin",
                            "admin_pw": "pw",
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(approve_response.status(), StatusCode::OK);

        let poll_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/cli/auth/poll/{}?poll_token={}",
                        start.request_id, start.poll_token
                    ))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(poll_response.status(), StatusCode::OK);
        let poll_body = to_bytes(poll_response.into_body(), usize::MAX)
            .await
            .expect("body");
        let poll: CliAuthPollResponse = serde_json::from_slice(&poll_body).expect("json");
        assert_eq!(poll.status, "approved");

        let exchange_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/cli/auth/exchange/{}", start.request_id))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "poll_token": start.poll_token,
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(exchange_response.status(), StatusCode::OK);
        let set_cookie = exchange_response.headers().get_all("set-cookie");
        assert!(set_cookie.iter().count() >= 2);

        let exchange_body = to_bytes(exchange_response.into_body(), usize::MAX)
            .await
            .expect("body");
        let exchange: CliAuthExchangeResponse =
            serde_json::from_slice(&exchange_body).expect("json");
        assert_eq!(exchange.role, "admin");
        assert_eq!(exchange.sub, "admin");
    }
}
