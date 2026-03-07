//! Runtime infrastructure for the backend.
//!
//! The infrastructure layer owns application state, environment-backed
//! configuration, audit logging helpers, and raw database mapping structs.

/// Audit logging helpers.
pub mod audit;
/// Environment-backed runtime configuration.
pub mod config;
/// Shared application state and database helper functions.
pub mod database;
/// Low-level row-mapping structs used by SQL queries.
pub mod db_models;

#[doc(inline)]
pub use audit::*;
#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use database::*;
#[doc(inline)]
pub use db_models::*;
