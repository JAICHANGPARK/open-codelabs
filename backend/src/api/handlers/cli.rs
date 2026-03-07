use crate::api::dto::{CliRuntimeCapabilities, CliRuntimeInfo};
use axum::Json;

/// Returns runtime metadata used by the standalone CLI to probe server support.
pub async fn get_cli_runtime() -> Json<CliRuntimeInfo> {
    Json(CliRuntimeInfo {
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
    })
}
