//! Service helpers for infrastructure-adjacent domain operations.

/// Workspace and git orchestration for code-server style flows.
pub mod codeserver;

#[doc(inline)]
pub use codeserver::*;
