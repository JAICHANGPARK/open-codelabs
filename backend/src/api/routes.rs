use crate::api::handlers::{
    admin::{get_session, login, logout, update_settings},
    ai::proxy_gemini_stream,
    attendees::{
        complete_codelab, get_attendees, get_certificate, get_help_requests, register_attendee,
        request_help, resolve_help_request,
    },
    audit::get_audit_logs,
    codeserver::{
        create_branch, create_codeserver, create_folder, delete_codeserver, download_workspace,
        get_codeserver_info, list_branches, list_files, list_folder_files, list_folders,
        read_file, read_folder_file,
    },
    codelabs::{
        copy_codelab, create_codelab, delete_codelab, export_codelab, get_chat_history,
        get_codelab, import_codelab, list_codelabs, update_codelab_info, update_codelab_steps,
    },
    feedback::{get_feedback, submit_feedback},
    materials::{add_material, delete_material, get_materials, upload_material_file},
    quizzes::{get_quiz_submissions, get_quizzes, submit_quiz, update_quizzes},
    submissions::{delete_submission, get_submissions, submit_file},
    upload::upload_image,
    websocket::ws_handler,
};
use crate::infrastructure::AppState;
use crate::middleware::{
    build_cors_layer, csrf_middleware, rate_limit_middleware, security_headers_middleware,
};
use axum::middleware;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/session", get(get_session))
        .route("/api/admin/settings", post(update_settings))
        .route("/api/admin/audit-logs", get(get_audit_logs))
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
        .route(
            "/api/codelabs/{id}/quizzes",
            get(get_quizzes).put(update_quizzes),
        )
        .route("/api/codelabs/{id}/quizzes/submit", post(submit_quiz))
        .route(
            "/api/codelabs/{id}/quizzes/submissions",
            get(get_quiz_submissions),
        )
        .route("/api/codelabs/{id}/submissions", get(get_submissions))
        .route(
            "/api/codelabs/{id}/attendees/{attendee_id}/submissions",
            post(submit_file),
        )
        .route(
            "/api/codelabs/{id}/attendees/{attendee_id}/submissions/{submission_id}",
            delete(delete_submission),
        )
        .route("/api/codelabs/{id}/chat", get(get_chat_history))
        .route("/api/upload/image", post(upload_image))
        .route("/api/upload/material", post(upload_material_file))
        .route("/api/ai/stream", post(proxy_gemini_stream))
        .route("/api/ws/{id}", get(ws_handler))
        .route("/api/codeserver", post(create_codeserver))
        .route(
            "/api/codeserver/{codelab_id}",
            get(get_codeserver_info).delete(delete_codeserver),
        )
        .route("/api/codeserver/{codelab_id}/branch", post(create_branch))
        .route("/api/codeserver/{codelab_id}/folder", post(create_folder))
        .route(
            "/api/codeserver/{codelab_id}/download",
            get(download_workspace),
        )
        .route("/api/codeserver/{codelab_id}/branches", get(list_branches))
        .route("/api/codeserver/{codelab_id}/branches/{branch}/files", get(list_files))
        .route("/api/codeserver/{codelab_id}/branches/{branch}/file", get(read_file))
        .route("/api/codeserver/{codelab_id}/folders", get(list_folders))
        .route("/api/codeserver/{codelab_id}/folders/{folder}/files", get(list_folder_files))
        .route("/api/codeserver/{codelab_id}/folders/{folder}/file", get(read_folder_file))
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback_service(ServeDir::new("static"))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers_middleware,
        ))
        .layer(build_cors_layer())
        .with_state(state)
}
