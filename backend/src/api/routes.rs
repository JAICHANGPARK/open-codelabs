use crate::api::handlers::codelabs::get_reference_codelabs;
use crate::api::handlers::{
    admin::{check_updates, get_session, login, logout, update_settings},
    ai::{
        add_ai_message, create_ai_thread, delete_ai_thread, get_ai_conversations, get_ai_messages,
        get_ai_threads, proxy_gemini_stream, save_ai_conversation,
    },
    attendees::{
        complete_codelab, get_attendees, get_certificate, get_help_requests, register_attendee,
        request_help, resolve_help_request,
    },
    audit::get_audit_logs,
    backup::{export_backup, inspect_backup, restore_backup},
    codelabs::{
        copy_codelab, create_codelab, delete_codelab, export_codelab, get_chat_history,
        get_codelab, import_codelab, list_codelabs, update_codelab_info, update_codelab_steps,
    },
    codeserver::{
        create_branch, create_codeserver, create_folder, delete_codeserver, download_workspace,
        get_codeserver_info, list_branches, list_files, list_folder_files, list_folders, read_file,
        read_folder_file, update_branch_files, update_folder_files,
    },
    feedback::{get_feedback, submit_feedback},
    inline_comments::{
        create_inline_comment, delete_inline_comment, get_inline_comments, reply_inline_comment,
    },
    materials::{add_material, delete_material, get_materials, upload_material_file},
    quizzes::{get_quiz_submissions, get_quizzes, submit_quiz, update_quizzes},
    submissions::{delete_submission, get_submissions, submit_file, submit_link},
    upload::upload_image,
    websocket::ws_handler,
};
use crate::infrastructure::AppState;
use crate::middleware::{
    build_cors_layer, csrf_middleware, rate_limit_middleware, security_headers_middleware,
};
use axum::middleware;
use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/session", get(get_session))
}

fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/settings", post(update_settings))
        .route("/api/admin/updates", get(check_updates))
        .route("/api/admin/audit-logs", get(get_audit_logs))
        .route("/api/admin/backup/export", get(export_backup))
        .route("/api/admin/backup/inspect", post(inspect_backup))
        .route("/api/admin/backup/restore", post(restore_backup))
}

fn codelab_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/codelabs/reference", get(get_reference_codelabs))
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
            "/api/codelabs/{id}/attendees/{attendee_id}/submissions/link",
            post(submit_link),
        )
        .route(
            "/api/codelabs/{id}/attendees/{attendee_id}/submissions/{submission_id}",
            delete(delete_submission),
        )
        .route("/api/codelabs/{id}/chat", get(get_chat_history))
        .route(
            "/api/codelabs/{id}/inline-comments",
            get(get_inline_comments).post(create_inline_comment),
        )
        .route(
            "/api/codelabs/{id}/inline-comments/{thread_id}/comments",
            post(reply_inline_comment),
        )
        .route(
            "/api/codelabs/{id}/inline-comments/{thread_id}/comments/{comment_id}",
            delete(delete_inline_comment),
        )
        .route(
            "/api/codelabs/{id}/ai/conversations",
            get(get_ai_conversations),
        )
}

fn upload_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/upload/image", post(upload_image))
        .route("/api/upload/material", post(upload_material_file))
}

fn ai_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ai/stream", post(proxy_gemini_stream))
        .route("/api/ai/conversations", post(save_ai_conversation))
        .route(
            "/api/ai/threads",
            get(get_ai_threads).post(create_ai_thread),
        )
        .route(
            "/api/ai/threads/{thread_id}",
            get(get_ai_messages)
                .post(add_ai_message)
                .delete(delete_ai_thread),
        )
}

fn websocket_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/ws/{id}", get(ws_handler))
}

fn codeserver_routes() -> Router<Arc<AppState>> {
    Router::new()
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
        .route(
            "/api/codeserver/{codelab_id}/branches/{branch}/files",
            get(list_files).post(update_branch_files),
        )
        .route(
            "/api/codeserver/{codelab_id}/branches/{branch}/file",
            get(read_file),
        )
        .route("/api/codeserver/{codelab_id}/folders", get(list_folders))
        .route(
            "/api/codeserver/{codelab_id}/folders/{folder}/files",
            get(list_folder_files).post(update_folder_files),
        )
        .route(
            "/api/codeserver/{codelab_id}/folders/{folder}/file",
            get(read_folder_file),
        )
}

pub fn create_router(state: Arc<AppState>) -> Router {
    auth_routes()
        .merge(admin_routes())
        .merge(codelab_routes())
        .merge(upload_routes())
        .merge(ai_routes())
        .merge(websocket_routes())
        .merge(codeserver_routes())
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback_service(ServeDir::new("static"))
        .layer(DefaultBodyLimit::max(200 * 1024 * 1024))
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
