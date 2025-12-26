use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Codelab {
    id: String,
    title: String,
    description: String,
    author: String,
    created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Step {
    id: String,
    codelab_id: String,
    step_number: i32,
    title: String,
    content_markdown: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateCodelab {
    title: String,
    description: String,
    author: String,
}

struct AppState {
    pool: SqlitePool,
}

use tower_http::services::ServeDir;

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

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = std::sync::Arc::new(AppState { pool });

    // Build our application with routes
    let app = Router::new()
        .route("/api/codelabs", get(list_codelabs).post(create_codelab))
        .route("/api/codelabs/:id", get(get_codelab))
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

async fn list_codelabs(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<Vec<Codelab>>, (StatusCode, String)> {
    let codelabs = sqlx::query_as::<_, Codelab>("SELECT * FROM codelabs")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codelabs))
}

async fn get_codelab(
    Path(id): Path<String>,
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<(Codelab, Vec<Step>)>, (StatusCode, String)> {
    let codelab = sqlx::query_as::<_, Codelab>("SELECT * FROM codelabs WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;

    let steps =
        sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE codelab_id = ? ORDER BY step_number")
            .bind(&id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json((codelab, steps)))
}

async fn create_codelab(
    State(state): State<std::sync::Arc<AppState>>,
    Json(payload): Json<CreateCodelab>,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO codelabs (id, title, description, author) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.author)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let codelab = sqlx::query_as::<_, Codelab>("SELECT * FROM codelabs WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codelab))
}
