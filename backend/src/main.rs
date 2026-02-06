use backend::infrastructure::{db_kind_from_url, ensure_sqlite_directory, AppConfig};
use backend::{create_router, AppState};
use sqlx::any::AnyPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    if database_url.starts_with("mysql") {
        anyhow::bail!("MySQL is not supported in this build; use sqlite or postgres.");
    }

    // Ensure directory exists for sqlite
    ensure_sqlite_directory(&database_url).ok();

    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let db_kind = db_kind_from_url(&database_url);

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_config = AppConfig::from_env()?;

    let state = Arc::new(AppState::new_with_config(pool, db_kind, app_config));

    // Build our application with routes
    let app = create_router(state);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
