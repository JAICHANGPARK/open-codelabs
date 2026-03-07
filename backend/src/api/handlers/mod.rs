//! HTTP handler implementations grouped by feature area.

/// Admin authentication, settings, and update checks.
pub mod admin;
/// AI proxying, threaded chat, and conversation persistence.
pub mod ai;
/// Attendee registration, progress, help, and certificates.
pub mod attendees;
/// Audit log listing endpoints.
pub mod audit;
/// Backup export, restore, and inspection handlers.
pub mod backup;
/// CLI runtime discovery and browser-auth support handlers.
pub mod cli;
/// Codelab CRUD, import/export, and chat history handlers.
pub mod codelabs;
/// Workspace and code-server orchestration handlers.
pub mod codeserver;
/// Learner feedback submission and listing handlers.
pub mod feedback;
/// Inline comment thread handlers for guide and step content.
pub mod inline_comments;
/// Codelab material management handlers.
pub mod materials;
/// Quiz listing, updates, submissions, and results handlers.
pub mod quizzes;
/// Learner submission upload and management handlers.
pub mod submissions;
/// Standalone asset upload handlers.
pub mod upload;
/// Websocket connection and live messaging handlers.
pub mod websocket;
