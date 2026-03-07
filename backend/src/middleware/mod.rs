//! Cross-cutting request middleware and extractors.

/// Authentication helpers and request extractors.
pub mod auth;
/// Request rate-limit configuration and middleware.
pub mod rate_limit;
/// Shared request metadata extraction helpers.
pub mod request_info;
/// Security-related middleware such as headers, CSRF, and CORS.
pub mod security;

#[doc(inline)]
pub use auth::*;
#[doc(inline)]
pub use rate_limit::*;
#[doc(inline)]
pub use request_info::*;
#[doc(inline)]
pub use security::*;
