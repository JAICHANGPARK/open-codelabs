use serde::{Deserialize, Serialize};

/// Capability matrix returned by the backend CLI runtime probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRuntimeCapabilities {
    /// Whether the backend exposes the administrative HTTP API used by the CLI.
    pub admin_api: bool,
    /// Whether backup export, inspect, and restore are available.
    pub backup: bool,
    /// Whether workspace management endpoints are available.
    pub workspace: bool,
    /// Whether audit log endpoints are available.
    pub audit: bool,
    /// Whether the backend supports browser-based CLI authentication.
    pub browser_auth: bool,
}

/// High-level metadata describing the connected runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRuntimeInfo {
    /// Runtime kind such as `backend`.
    pub runtime: String,
    /// Backend version exposed to the CLI.
    pub version: String,
    /// Supported login methods for the runtime.
    pub auth_methods: Vec<String>,
    /// Capability flags used by `oc connect status`.
    pub capabilities: CliRuntimeCapabilities,
}
