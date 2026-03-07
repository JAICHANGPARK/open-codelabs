//! Request and response DTOs exposed by the HTTP API.

/// Admin-facing request payloads.
pub mod admin;
/// AI proxy request payloads.
pub mod ai;
/// Audit log query parameters.
pub mod audit;
/// CLI runtime metadata and capability DTOs.
pub mod cli;
/// Code-server and workspace management DTOs.
pub mod codeserver;

#[doc(inline)]
pub use admin::*;
#[doc(inline)]
pub use ai::*;
#[doc(inline)]
pub use audit::*;
#[doc(inline)]
pub use cli::*;
#[doc(inline)]
pub use codeserver::*;
