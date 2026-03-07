//! Backend library crate for the Open Codelabs server.
//!
//! The crate is organized around HTTP API wiring, shared domain models,
//! runtime infrastructure, request middleware, and small utility helpers.

/// HTTP routes, DTOs, and handlers exposed by the server.
pub mod api;
/// Shared helpers for the administrative CLI binary.
pub mod cli;
/// Shared business models and service helpers used across handlers.
pub mod domain;
/// Runtime configuration, database state, and persistence-facing models.
pub mod infrastructure;
/// Cross-cutting request middleware and extractors.
pub mod middleware;
/// Small reusable helpers such as validation and crypto utilities.
pub mod utils;

#[doc(inline)]
pub use api::create_router;
#[doc(inline)]
pub use infrastructure::{AppState, DbKind};
