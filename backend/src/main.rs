mod handlers;
mod models;
mod state;

use axum::{
    routing::{get, post, put},
    Router,
};
use sqlx::any::AnyPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handlers::{
    admin::login,
    attendees::{
        get_attendees, get_help_requests, register_attendee, request_help, resolve_help_request,
    },
    codelabs::{
        create_codelab, delete_codelab, export_codelab, get_chat_history, get_codelab,
        import_codelab, list_codelabs, update_codelab_info, update_codelab_steps,
    },
    feedback::{get_feedback, submit_feedback},
    upload::upload_image,
    websocket::ws_handler,
};
use crate::state::{AppState, DbKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/sqlite.db?mode=rwc".to_string());

    // Ensure directory exists for sqlite
    if database_url.starts_with("sqlite:") {
        let path = database_url.replace("sqlite:", "");
        let path = path.split('?').next().unwrap_or(&path);
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).ok();
            }
        }
    }

    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let db_kind = if database_url.starts_with("postgres") {
        DbKind::Postgres
    } else if database_url.starts_with("mysql") {
        DbKind::Mysql
    } else {
        DbKind::Sqlite
    };

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    let admin_id = std::env::var("ADMIN_ID").expect("ADMIN_ID must be set");
    let admin_pw = std::env::var("ADMIN_PW").expect("ADMIN_PW must be set");

    let state = Arc::new(AppState {
        pool,
        db_kind,
        admin_id,
        admin_pw,
        channels: Arc::new(dashmap::DashMap::new()),
        sessions: Arc::new(dashmap::DashMap::new()),
    });

    // Build our application with routes
    let app = Router::new()
        .route("/api/login", post(login))
        .route("/api/codelabs", get(list_codelabs).post(create_codelab))
        .route(
            "/api/codelabs/:id",
            get(get_codelab)
                .put(update_codelab_info)
                .delete(delete_codelab),
        )
        .route("/api/codelabs/:id/steps", put(update_codelab_steps))
        .route("/api/codelabs/:id/export", get(export_codelab))
        .route("/api/codelabs/import", post(import_codelab))
        .route("/api/codelabs/:id/register", post(register_attendee))
        .route("/api/codelabs/:id/attendees", get(get_attendees))
        .route(
            "/api/codelabs/:id/help",
            post(request_help).get(get_help_requests),
        )
        .route(
            "/api/codelabs/:id/help/:help_id/resolve",
            post(resolve_help_request),
        )
        .route(
            "/api/codelabs/:id/feedback",
            post(submit_feedback).get(get_feedback),
        )
        .route("/api/codelabs/:id/chat", get(get_chat_history))
        .route("/api/upload/image", post(upload_image))
        .route("/api/ws/:id", get(ws_handler))
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback_service(ServeDir::new("static"))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
