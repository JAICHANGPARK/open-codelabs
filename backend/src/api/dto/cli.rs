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

/// Response returned when a browser-auth challenge is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliAuthStartResponse {
    /// Stable challenge identifier embedded in browser URLs.
    pub request_id: String,
    /// Secret used by the CLI while polling and exchanging the approval.
    pub poll_token: String,
    /// Relative browser path that should be opened by the CLI.
    pub verification_path: String,
    /// Absolute expiration timestamp in epoch seconds.
    pub expires_at_epoch: i64,
    /// Suggested polling cadence.
    pub poll_interval_seconds: u64,
}

/// Query parameters used while polling a browser-auth challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliAuthPollQuery {
    /// One-time secret held by the CLI.
    pub poll_token: String,
}

/// State returned while polling a browser-auth challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliAuthPollResponse {
    /// Challenge state such as `pending`, `approved`, or `expired`.
    pub status: String,
}

/// Body used when exchanging an approved browser-auth challenge for cookies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliAuthExchangeRequest {
    /// One-time secret held by the CLI.
    pub poll_token: String,
}

/// Session payload returned after a successful browser-auth exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliAuthExchangeResponse {
    /// Stable subject identifier for the issued session.
    pub sub: String,
    /// Issued role, currently `admin`.
    pub role: String,
    /// Optional codelab scope.
    pub codelab_id: Option<String>,
    /// Expiration timestamp in epoch seconds.
    pub exp: usize,
}

/// Approval payload sent from the browser challenge page.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliAuthApproveRequest {
    /// Optional admin identifier when no browser session exists.
    #[serde(default)]
    pub admin_id: String,
    /// Optional admin password when no browser session exists.
    #[serde(default)]
    pub admin_pw: String,
}

/// Minimal approval response sent back to the browser page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliAuthApproveResponse {
    /// Final approval state.
    pub status: String,
}
