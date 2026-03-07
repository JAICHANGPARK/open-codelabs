use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Persisted CLI configuration containing connection profiles.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    /// Selected default profile name.
    pub current_profile: Option<String>,
    /// Named connection profiles.
    #[serde(default)]
    pub profiles: BTreeMap<String, ConnectionProfile>,
}

/// A saved server target used by `oc connect`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    /// Backend or frontend base URL.
    pub base_url: String,
    /// Preferred runtime mode for the profile.
    #[serde(default)]
    pub runtime: RuntimePreference,
}

/// Runtime preference selected for a profile.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePreference {
    /// Probe the target and infer the runtime when possible.
    #[default]
    Auto,
    /// Full backend runtime served by the Rust API.
    Backend,
    /// Firebase-hosted runtime with limited CLI support.
    Firebase,
    /// Supabase-hosted runtime with limited CLI support.
    Supabase,
}

impl RuntimePreference {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "backend" => Some(Self::Backend),
            "firebase" => Some(Self::Firebase),
            "supabase" => Some(Self::Supabase),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Backend => "backend",
            Self::Firebase => "firebase",
            Self::Supabase => "supabase",
        }
    }
}

/// Returns the default config file path used by the CLI.
pub fn default_config_path() -> PathBuf {
    env::var_os("OPEN_CODELABS_CONFIG_FILE")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|dir| dir.join(".open-codelabs").join("config.json")))
        .unwrap_or_else(|| PathBuf::from(".open-codelabs-config.json"))
}

/// Returns the default profile-specific session path.
pub fn default_profile_session_path(profile_name: &str) -> PathBuf {
    home_dir()
        .map(|dir| {
            dir.join(".open-codelabs")
                .join("profiles")
                .join(sanitize_profile_name(profile_name))
                .join("session.json")
        })
        .unwrap_or_else(|| {
            PathBuf::from(format!(
                ".open-codelabs-{}.session.json",
                sanitize_profile_name(profile_name)
            ))
        })
}

/// Loads the saved CLI configuration from disk, returning an empty config when missing.
pub fn load_config(path: &Path) -> Result<CliConfig> {
    if !path.exists() {
        return Ok(CliConfig::default());
    }

    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse config file {}", path.display()))
}

/// Saves the CLI configuration to disk, creating parent directories as needed.
pub fn save_config(path: &Path, config: &CliConfig) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
    }

    let raw = serde_json::to_string_pretty(config).context("Failed to serialize CLI config")?;
    fs::write(path, raw).with_context(|| format!("Failed to write config {}", path.display()))?;
    set_owner_only_permissions(path)?;
    Ok(())
}

fn sanitize_profile_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if sanitized.is_empty() {
        "default".to_string()
    } else {
        sanitized
    }
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("Failed to secure config file {}", path.display()))?;
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
        let path = dir.path().join("config.json");
        let mut config = CliConfig::default();
        config.current_profile = Some("local".to_string());
        config.profiles.insert(
            "local".to_string(),
            ConnectionProfile {
                base_url: "http://localhost:8080".to_string(),
                runtime: RuntimePreference::Backend,
            },
        );

        save_config(&path, &config).expect("save");
        let loaded = load_config(&path).expect("load");
        assert_eq!(loaded.current_profile.as_deref(), Some("local"));
        assert_eq!(
            loaded
                .profiles
                .get("local")
                .map(|profile| profile.base_url.as_str()),
            Some("http://localhost:8080")
        );
    }

    #[test]
    fn missing_config_returns_default() {
        let dir = tempdir().expect("temp dir");
        let loaded = load_config(&dir.path().join("missing.json")).expect("load");
        assert!(loaded.current_profile.is_none());
        assert!(loaded.profiles.is_empty());
    }

    #[test]
    fn profile_session_path_is_sanitized() {
        let path = default_profile_session_path("dev/local");
        let path_text = path.display().to_string();
        assert!(path_text.contains("dev-local"));
        assert!(path_text.ends_with("session.json"));
    }
}
