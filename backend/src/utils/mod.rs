//! Small reusable utility helpers used across the backend.

/// Symmetric encryption helpers for secrets stored at rest.
pub mod crypto;
/// Common HTTP error response builders.
pub mod error;
/// Payload validation helpers shared by handlers.
pub mod validation;

#[doc(inline)]
pub use crypto::*;
#[doc(inline)]
pub use error::*;
#[doc(inline)]
pub use validation::*;
