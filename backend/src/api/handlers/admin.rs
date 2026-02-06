use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::middleware::auth::{
    build_csrf_cookie, build_session_cookie, clear_cookie, now_epoch_seconds, AuthSession, Role,
    SessionClaims,
};
use crate::utils::crypto::decrypt_with_password;
use crate::utils::error::{bad_request, internal_error, unauthorized};
use crate::api::dto::SettingsPayload;
use crate::domain::models::LoginPayload;
use crate::middleware::request_info::RequestInfo;
use crate::middleware::security::ensure_csrf_cookie;
use crate::infrastructure::database::AppState;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::CookieJar;
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
            token,
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

    Ok((jar, Json(serde_json::json!({ "status": "ok" }))))
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
    use crate::utils::crypto::{decrypt_with_password, encrypt_with_password, ENCRYPTION_PREFIX};

    #[test]
    fn test_password_encryption_round_trip() {
        let password = "admin";
        let text = "secret-api-key";
        let encrypted = encrypt_with_password(text, password).expect("encrypt");
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX));
        let decrypted = decrypt_with_password(&encrypted, password).expect("decrypt");
        assert_eq!(text, decrypted);
    }
}
