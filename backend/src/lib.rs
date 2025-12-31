pub mod handlers;
pub mod models;
pub mod state;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;
use crate::handlers::{
    admin::login,
    attendees::{
        complete_codelab, get_attendees, get_certificate, get_help_requests, register_attendee,
        request_help, resolve_help_request,
    },
    codelabs::{
        copy_codelab, create_codelab, delete_codelab, export_codelab, get_chat_history,
        get_codelab, import_codelab, list_codelabs, update_codelab_info, update_codelab_steps,
    },
    feedback::{get_feedback, submit_feedback},
    materials::{add_material, delete_material, get_materials, upload_material_file},
    quizzes::{get_quizzes, update_quizzes, submit_quiz, get_quiz_submissions},
    submissions::{submit_file, get_submissions, delete_submission},
    upload::upload_image,
    websocket::ws_handler,
};
pub use crate::state::{AppState, DbKind};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/codelabs", get(list_codelabs).post(create_codelab))
        .route(
            "/api/codelabs/{id}",
            get(get_codelab)
                .put(update_codelab_info)
                .delete(delete_codelab),
        )
        .route("/api/codelabs/{id}/copy", post(copy_codelab))
        .route("/api/codelabs/{id}/steps", put(update_codelab_steps))
        .route("/api/codelabs/{id}/export", get(export_codelab))
        .route("/api/codelabs/import", post(import_codelab))
        .route("/api/codelabs/{id}/register", post(register_attendee))
        .route("/api/codelabs/{id}/complete", post(complete_codelab))
        .route("/api/codelabs/{id}/attendees", get(get_attendees))
        .route("/api/certificates/{id}", get(get_certificate))
        .route(
            "/api/codelabs/{id}/help",
            post(request_help).get(get_help_requests),
        )
        .route(
            "/api/codelabs/{id}/help/{help_id}/resolve",
            post(resolve_help_request),
        )
        .route(
            "/api/codelabs/{id}/feedback",
            post(submit_feedback).get(get_feedback),
        )
        .route(
            "/api/codelabs/{id}/materials",
            get(get_materials).post(add_material),
        )
        .route(
            "/api/codelabs/{id}/materials/{material_id}",
            delete(delete_material),
        )
        .route("/api/codelabs/{id}/quizzes", get(get_quizzes).put(update_quizzes))
        .route("/api/codelabs/{id}/quizzes/submit", post(submit_quiz))
        .route("/api/codelabs/{id}/quizzes/submissions", get(get_quiz_submissions))
        .route("/api/codelabs/{id}/submissions", get(get_submissions))
        .route("/api/codelabs/{id}/attendees/{attendee_id}/submissions", post(submit_file))
        .route("/api/codelabs/{id}/attendees/{attendee_id}/submissions/{submission_id}", delete(delete_submission))
        .route("/api/codelabs/{id}/chat", get(get_chat_history))
        .route("/api/upload/image", post(upload_image))
        .route("/api/upload/material", post(upload_material_file))
        .route("/api/ws/{id}", get(ws_handler))
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback_service(ServeDir::new("static"))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}
