use crate::api::dto::SettingsPayload;
use crate::domain::models::LoginPayload;
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::infrastructure::database::AppState;
use crate::middleware::auth::{
    build_csrf_cookie, build_session_cookie, clear_cookie, now_epoch_seconds, AuthSession, Role,
    SessionClaims,
};
use crate::middleware::request_info::RequestInfo;
use crate::middleware::security::ensure_csrf_cookie;
use crate::utils::crypto::decrypt_with_password;
use crate::utils::error::{bad_request, internal_error, unauthorized};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use subtle::ConstantTimeEq;

pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    info: RequestInfo,
    Json(payload): Json<LoginPayload>,
) -> Result<(CookieJar, Json<serde_json::Value>), (StatusCode, String)> {
    if payload.admin_id.trim().is_empty() || payload.admin_pw.trim().is_empty() {
        return Err(bad_request("admin_id and admin_pw are required"));
    }

    let id_ok = payload
        .admin_id
        .as_bytes()
        .ct_eq(state.admin_id.as_bytes())
        .into();
    let pw_ok = payload
        .admin_pw
        .as_bytes()
        .ct_eq(state.admin_pw.as_bytes())
        .into();

    if !(id_ok && pw_ok) {
        record_audit(
            &state,
            AuditEntry {
                action: "admin_login_failed".to_string(),
                actor_type: "admin".to_string(),
                actor_id: Some(payload.admin_id),
                target_id: None,
                codelab_id: None,
                ip: Some(info.ip),
                user_agent: info.user_agent,
                metadata: None,
            },
        )
        .await;
        return Err(unauthorized());
    }

    let now = now_epoch_seconds();
    let claims = SessionClaims {
        sub: state.admin_id.clone(),
        role: Role::Admin.as_str().to_string(),
        codelab_id: None,
        iss: state.auth.issuer.clone(),
        aud: state.auth.audience.clone(),
        iat: now,
        exp: now + state.auth.admin_ttl.as_secs() as usize,
    };
    let token = state.auth.issue_token(&claims).map_err(internal_error)?;
    let csrf_token = crate::middleware::auth::generate_csrf_token();

    let jar = jar
        .add(build_session_cookie(
            &state.auth,
            token.clone(),
            state.auth.admin_ttl,
        ))
        .add(build_csrf_cookie(
            &state.auth,
            csrf_token,
            state.auth.admin_ttl,
        ));

    record_audit(
        &state,
        AuditEntry {
            action: "admin_login_success".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(state.admin_id.clone()),
            target_id: None,
            codelab_id: None,
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok((
        jar,
        Json(serde_json::json!({ "status": "ok", "token": token })),
    ))
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<SettingsPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    session.require_admin()?;
    let mut api_key = payload.gemini_api_key.trim().to_string();
    if api_key.len() > 4096 {
        return Err(bad_request("API key too long"));
    }

    // Try to decrypt if it's not empty
    if !api_key.is_empty() {
        match decrypt_with_password(&api_key, &state.admin_pw) {
            Ok(decrypted) => api_key = decrypted,
            Err(_) => {
                // Decryption failed. This means the key is either:
                // 1. Not encrypted (plain text)
                // 2. Encrypted with a different password
                // Since we now enforce encryption on the client, we should reject plain text.
                tracing::error!("Failed to decrypt API key. Plain text keys are no longer accepted for security reasons.");
                return Err(bad_request("Invalid encrypted API key format"));
            }
        }
    }

    if !api_key.is_empty() {
        state
            .admin_api_keys
            .insert("global_admin".to_string(), api_key);
    } else {
        state.admin_api_keys.remove("global_admin");
    }

    record_audit(
        &state,
        AuditEntry {
            action: "admin_settings_update".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(state.admin_id.clone()),
            target_id: None,
            codelab_id: None,
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;
    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UpdateStatus {
    current: Option<String>,
    latest: Option<String>,
    update_available: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateCheckResponse {
    frontend: UpdateStatus,
    backend: UpdateStatus,
}

#[derive(Debug, Deserialize)]
pub struct GhcrTagsResponse {
    tags: Option<Vec<String>>,
}

#[rustfmt::skip]
async fn fetch_latest_tag(image: &str) -> Result<Option<String>, String> {
    let base = std::env::var("GHCR_API_BASE").unwrap_or_else(|_| "https://ghcr.io/v2".to_string());
    let url = format!("{}/{}/tags/list", base.trim_end_matches('/'), image);
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("http client error: {}", e))?;
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("request error: {}", e))?;
    if res.status().is_success() { res.json::<GhcrTagsResponse>().await.map(|data| pick_latest_tag(data.tags.unwrap_or_default())).map_err(|e| e.to_string()) } else { Err(format!("ghcr status {}", res.status())) }
}

fn pick_latest_tag(tags: Vec<String>) -> Option<String> {
    if tags.is_empty() {
        return None;
    }
    let mut semver_tags: Vec<(String, (u64, u64, u64))> = tags
        .iter()
        .filter_map(|tag| parse_semver(tag).map(|v| (tag.clone(), v)))
        .collect();
    if !semver_tags.is_empty() {
        semver_tags.sort_by(|a, b| a.1.cmp(&b.1));
        return Some(semver_tags.last().expect("non-empty").0.clone());
    }
    if tags.iter().any(|t| t == "latest") {
        return Some("latest".to_string());
    }
    let mut sorted = tags.clone();
    sorted.sort();
    sorted.last().cloned()
}

fn parse_semver(tag: &str) -> Option<(u64, u64, u64)> {
    let cleaned = tag.trim_start_matches('v');
    let parts: Vec<&str> = cleaned.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts[2].parse().ok()?;
    Some((major, minor, patch))
}

fn build_update_status(
    current: Option<String>,
    latest_result: Result<Option<String>, String>,
) -> UpdateStatus {
    match latest_result {
        Ok(latest) => UpdateStatus {
            current: current.clone(),
            latest: latest.clone(),
            update_available: latest
                .as_ref()
                .and_then(|l| current.as_ref().map(|c| l != c))
                .unwrap_or(false),
            error: None,
        },
        Err(err) => UpdateStatus {
            current,
            latest: None,
            update_available: false,
            error: Some(err),
        },
    }
}

pub async fn check_updates(
    State(_state): State<Arc<AppState>>,
    session: AuthSession,
) -> Result<Json<UpdateCheckResponse>, (StatusCode, String)> {
    session.require_admin()?;

    let frontend_current = std::env::var("FRONTEND_IMAGE_TAG").ok();
    let backend_current = std::env::var("BACKEND_IMAGE_TAG").ok();

    let frontend_latest = fetch_latest_tag("jaichangpark/open-codelabs-frontend").await;
    let backend_latest = fetch_latest_tag("jaichangpark/open-codelabs-backend").await;

    let frontend_status = build_update_status(frontend_current, frontend_latest);
    let backend_status = build_update_status(backend_current, backend_latest);

    Ok(Json(UpdateCheckResponse {
        frontend: frontend_status,
        backend: backend_status,
    }))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), (StatusCode, String)> {
    let jar = jar.remove(clear_cookie(&state.auth.cookie_name));
    let jar = if jar.get(&state.auth.attendee_cookie_name).is_some() {
        jar
    } else {
        jar.remove(clear_cookie(&state.auth.csrf_cookie_name))
    };
    Ok((jar, StatusCode::NO_CONTENT))
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    session: AuthSession,
) -> Result<(CookieJar, Json<serde_json::Value>), (StatusCode, String)> {
    let claims = session.claims.ok_or_else(unauthorized)?;
    let max_age = if claims.role == Role::Admin.as_str() {
        state.auth.admin_ttl
    } else {
        state.auth.attendee_ttl
    };
    let jar = ensure_csrf_cookie(jar, &state, max_age);
    Ok((
        jar,
        Json(serde_json::json!({
            "sub": claims.sub,
            "role": claims.role,
            "codelab_id": claims.codelab_id,
            "exp": claims.exp,
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::crypto::{decrypt_with_password, encrypt_with_password, ENCRYPTION_PREFIX};
    use std::sync::{LazyLock, Mutex};

    static GHCR_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn restore_ghcr_api_base(previous: Option<String>) {
        if let Some(value) = previous {
            std::env::set_var("GHCR_API_BASE", value);
        } else {
            std::env::remove_var("GHCR_API_BASE");
        }
    }

    #[test]
    fn test_password_encryption_round_trip() {
        let password = "admin";
        let text = "secret-api-key";
        let encrypted = encrypt_with_password(text, password).expect("encrypt");
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX));
        let decrypted = decrypt_with_password(&encrypted, password).expect("decrypt");
        assert_eq!(text, decrypted);
    }

    #[test]
    fn test_parse_semver_variants() {
        assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("2.10.7"), Some((2, 10, 7)));
        assert_eq!(parse_semver("1.2"), None);
        assert_eq!(parse_semver("1.2.x"), None);
    }

    #[test]
    fn test_pick_latest_tag_prefers_semver_then_latest_then_lexicographic() {
        assert_eq!(
            pick_latest_tag(vec![
                "v1.2.0".to_string(),
                "v1.10.0".to_string(),
                "v1.3.9".to_string(),
            ]),
            Some("v1.10.0".to_string())
        );
        assert_eq!(
            pick_latest_tag(vec!["alpha".to_string(), "latest".to_string()]),
            Some("latest".to_string())
        );
        assert_eq!(
            pick_latest_tag(vec!["beta".to_string(), "alpha".to_string()]),
            Some("beta".to_string())
        );
        assert_eq!(pick_latest_tag(vec![]), None);
    }

    #[test]
    fn test_build_update_status_covers_ok_and_error_paths() {
        let ok = build_update_status(Some("1.0.0".to_string()), Ok(Some("1.1.0".to_string())));
        assert_eq!(
            ok,
            UpdateStatus {
                current: Some("1.0.0".to_string()),
                latest: Some("1.1.0".to_string()),
                update_available: true,
                error: None,
            }
        );

        let same = build_update_status(Some("1.1.0".to_string()), Ok(Some("1.1.0".to_string())));
        assert!(!same.update_available);

        let missing = build_update_status(None, Ok(Some("1.1.0".to_string())));
        assert!(!missing.update_available);

        let err = build_update_status(Some("1.0.0".to_string()), Err("network".to_string()));
        assert_eq!(err.latest, None);
        assert!(!err.update_available);
        assert_eq!(err.error.as_deref(), Some("network"));
    }

    #[test]
    fn test_restore_ghcr_api_base_removes_var_when_previous_missing() {
        let _env_lock = GHCR_ENV_LOCK.lock().expect("ghcr env lock");
        let original = std::env::var("GHCR_API_BASE").ok();
        std::env::set_var("GHCR_API_BASE", "http://temp.example/v2");

        restore_ghcr_api_base(None);
        assert!(std::env::var("GHCR_API_BASE").is_err());

        restore_ghcr_api_base(original);
    }

    #[test]
    fn test_restore_ghcr_api_base_restores_existing_value() {
        let _env_lock = GHCR_ENV_LOCK.lock().expect("ghcr env lock");
        let original = std::env::var("GHCR_API_BASE").ok();
        std::env::set_var("GHCR_API_BASE", "http://temp.example/v2");

        restore_ghcr_api_base(Some("http://restored.example/v2".to_string()));
        assert_eq!(
            std::env::var("GHCR_API_BASE").as_deref(),
            Ok("http://restored.example/v2")
        );

        restore_ghcr_api_base(original);
    }

    #[tokio::test]
    async fn test_fetch_latest_tag_reports_request_error_for_unreachable_base() {
        let _env_lock = GHCR_ENV_LOCK.lock().expect("ghcr env lock");
        let previous = std::env::var("GHCR_API_BASE").ok();
        std::env::set_var("GHCR_API_BASE", "http://127.0.0.1:9/v2");
        let err = fetch_latest_tag("repo")
            .await
            .expect_err("request should fail");
        assert!(err.contains("request error"), "unexpected error: {err}");

        restore_ghcr_api_base(previous);
    }
}
