//! Domain-level models and service helpers shared across the backend.

/// Serializable models used by handlers, persistence, and websocket payloads.
pub mod models;
/// Service objects that encapsulate non-HTTP operations.
pub mod services;

#[doc(inline)]
pub use models::*;
#[doc(inline)]
pub use services::*;
