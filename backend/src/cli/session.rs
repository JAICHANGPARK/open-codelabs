//! Session persistence helpers for the administrative CLI.

use crate::cli::paths::home_dir;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Persisted administrator session stored on disk for subsequent CLI invocations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    /// Base URL used when the session was created.
    pub base_url: String,
    /// Flattened `Cookie` header value to replay on later requests.
    pub cookie_header: String,
    /// CSRF token mirrored from the issued cookie when available.
    pub csrf_token: Option<String>,
    /// Cached subject from `/api/session`.
    pub sub: Option<String>,
    /// Cached role from `/api/session`.
    pub role: Option<String>,
    /// Cached session expiry in epoch seconds.
    pub exp: Option<usize>,
    /// Optional codelab scope for attendee sessions or future extensions.
    pub codelab_id: Option<String>,
}

impl StoredSession {
    /// Applies the latest session snapshot returned by the backend.
    pub fn apply_snapshot(&mut self, snapshot: &SessionSnapshot) {
        self.sub = Some(snapshot.sub.clone());
        self.role = Some(snapshot.role.clone());
        self.exp = Some(snapshot.exp);
        self.codelab_id = snapshot.codelab_id.clone();
    }
}

/// Minimal session metadata returned by `GET /api/session`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Stable subject identifier.
    pub sub: String,
    /// Session role such as `admin`.
    pub role: String,
    /// Optional codelab scope.
    pub codelab_id: Option<String>,
    /// Expiration timestamp in epoch seconds.
    pub exp: usize,
}

/// Returns the default path used for CLI session persistence.
pub fn default_session_path() -> PathBuf {
    let config_root = env::var_os("OPEN_CODELABS_SESSION_FILE")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|dir| dir.join(".open-codelabs").join("session.json")))
        .unwrap_or_else(|| PathBuf::from(".open-codelabs-session.json"));
    config_root
}

/// Loads a previously saved CLI session from disk.
pub fn load_session(path: &Path) -> Result<StoredSession> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read session file {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse session file {}", path.display()))
}

/// Saves a CLI session to disk, creating parent directories as needed.
pub fn save_session(path: &Path, session: &StoredSession) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create session directory {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(session).context("Failed to serialize session")?;
    fs::write(path, raw)
        .with_context(|| format!("Failed to write session file {}", path.display()))?;
    set_owner_only_permissions(path)?;
    Ok(())
}

/// Removes the saved CLI session file if it exists.
pub fn clear_session(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path)
        .with_context(|| format!("Failed to remove session file {}", path.display()))?;
    Ok(())
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("Failed to secure session file {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("session.json");
        let session = StoredSession {
            base_url: "http://localhost:8080".to_string(),
            cookie_header: "a=b".to_string(),
            csrf_token: Some("csrf".to_string()),
            sub: Some("admin".to_string()),
            role: Some("admin".to_string()),
            exp: Some(123),
            codelab_id: None,
        };

        save_session(&path, &session).expect("save session");
        let loaded = load_session(&path).expect("load session");
        assert_eq!(loaded.base_url, session.base_url);
        assert_eq!(loaded.cookie_header, session.cookie_header);
        assert_eq!(loaded.csrf_token, session.csrf_token);
        assert_eq!(loaded.sub, session.sub);
    }

    #[test]
    fn clear_session_removes_file() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("session.json");
        fs::write(&path, "{}").expect("write");

        clear_session(&path).expect("clear");
        assert!(!path.exists());
    }
}
