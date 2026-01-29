use backend::middleware::auth::AuthConfig;
use backend::middleware::rate_limit::{RateLimitConfig, RateLimiter};
use backend::middleware::security::SecurityHeadersConfig;
use backend::{create_router, AppState, DbKind};
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
    let trust_proxy = std::env::var("TRUST_PROXY")
        .ok()
        .map(|value| value == "true")
        .unwrap_or(false);

    let state = Arc::new(AppState {
        pool,
        db_kind,
        admin_id,
        admin_pw,
        auth: AuthConfig::from_env(),
        rate_limit_config: RateLimitConfig::from_env(),
        rate_limiter: Arc::new(RateLimiter::new()),
        security_headers: SecurityHeadersConfig::from_env(),
        trust_proxy,
        admin_api_keys: Arc::new(dashmap::DashMap::new()),
        channels: Arc::new(dashmap::DashMap::new()),
        sessions: Arc::new(dashmap::DashMap::new()),
    });

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
