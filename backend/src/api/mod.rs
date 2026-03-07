//! HTTP-facing API surface for the backend.
//!
//! This module groups request/response DTOs, handler implementations, and the
//! router builder that mounts the public API.

/// Request and response payload types used by handlers.
pub mod dto;
/// Axum handlers for the backend API.
pub mod handlers;
/// Route composition for the Axum application.
pub mod routes;

#[doc(inline)]
pub use routes::*;
