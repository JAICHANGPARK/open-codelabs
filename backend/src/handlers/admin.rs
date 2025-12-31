use crate::models::LoginPayload;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

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

#[derive(Deserialize)]
pub struct SettingsPayload {
    pub gemini_api_key: String,
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SettingsPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut api_key = payload.gemini_api_key.trim().to_string();

    // Try to decrypt if it's not empty
    if !api_key.is_empty() {
        let mc = new_magic_crypt!(&state.admin_pw, 256);
        if let Ok(decrypted) = mc.decrypt_base64_to_string(&api_key) {
            api_key = decrypted;
        } else {
            // Decryption failed. This means the key is either:
            // 1. Not encrypted (plain text)
            // 2. Encrypted with a different password
            // Since we now enforce encryption on the client, we should reject plain text.
            tracing::error!("Failed to decrypt API key. Plain text keys are no longer accepted for security reasons.");
            return Err((StatusCode::BAD_REQUEST, "Invalid encrypted API key format".to_string()));
        }
    }

    if !api_key.is_empty() {
        state.admin_api_keys.insert("global_admin".to_string(), api_key);
    } else {
        state.admin_api_keys.remove("global_admin");
    }
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use magic_crypt::{new_magic_crypt, MagicCryptTrait};

    #[test]
    fn test_magic_crypt_compatibility() {
        let password = "admin";
        let text = "secret-api-key";
        let mc = new_magic_crypt!(password, 256);
        
        let encrypted = mc.encrypt_str_to_base64(text);
        
        // Ensure this value matches what the frontend produces
        // With password="admin" and text="secret-api-key", the result should be "URh6eeDLAKxc2nYWOrhyjg=="
        assert_eq!(encrypted, "URh6eeDLAKxc2nYWOrhyjg==");
        
        let decrypted = mc.decrypt_base64_to_string(&encrypted).unwrap();
        assert_eq!(text, decrypted);
    }
}
