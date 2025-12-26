use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::io::{Cursor, Read, Write};
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

#[derive(Debug, Serialize, Deserialize)]
struct LoginPayload {
    admin_id: String,
    admin_pw: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateStepsPayload {
    steps: Vec<CreateStep>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateStep {
    title: String,
    content_markdown: String,
}

struct AppState {
    pool: SqlitePool,
    admin_id: String,
    admin_pw: String,
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

    let admin_id = std::env::var("ADMIN_ID").expect("ADMIN_ID must be set");
    let admin_pw = std::env::var("ADMIN_PW").expect("ADMIN_PW must be set");

    let state = std::sync::Arc::new(AppState {
        pool,
        admin_id,
        admin_pw,
    });

    // Build our application with routes
    let app = Router::new()
        .route("/api/login", post(login))
        .route("/api/codelabs", get(list_codelabs).post(create_codelab))
        .route(
            "/api/codelabs/:id",
            get(get_codelab).put(update_codelab_info),
        )
        .route("/api/codelabs/:id/steps", put(update_codelab_steps))
        .route("/api/codelabs/:id/export", get(export_codelab))
        .route("/api/codelabs/import", post(import_codelab))
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

async fn login(
    State(state): State<std::sync::Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if payload.admin_id == state.admin_id && payload.admin_pw == state.admin_pw {
        Ok(Json(
            serde_json::json!({ "status": "ok", "token": "mock-jwt-token" }),
        ))
    } else {
        Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
    }
}

async fn update_codelab_info(
    Path(id): Path<String>,
    State(state): State<std::sync::Arc<AppState>>,
    Json(payload): Json<CreateCodelab>,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    sqlx::query("UPDATE codelabs SET title = ?, description = ?, author = ? WHERE id = ?")
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.author)
        .bind(&id)
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

async fn update_codelab_steps(
    Path(id): Path<String>,
    State(state): State<std::sync::Arc<AppState>>,
    Json(payload): Json<UpdateStepsPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete existing steps
    sqlx::query("DELETE FROM steps WHERE codelab_id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Insert new steps
    for (i, step) in payload.steps.into_iter().enumerate() {
        let step_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO steps (id, codelab_id, step_number, title, content_markdown) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&step_id)
        .bind(&id)
        .bind((i + 1) as i32)
        .bind(&step.title)
        .bind(&step.content_markdown)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn export_codelab(
    Path(id): Path<String>,
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
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

    let mut buf = Vec::new();
    let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add metadata
    zip.start_file("codelab.json", options)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let metadata = serde_json::to_string_pretty(&codelab)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    zip.write_all(metadata.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Add steps
    for step in steps {
        let filename = format!(
            "step_{:02}_{}.md",
            step.step_number,
            step.title.replace(" ", "_")
        );
        zip.start_file(filename, options)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        zip.write_all(step.content_markdown.as_bytes())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    zip.finish()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&format!("attachment; filename=\"codelab_{}.zip\"", id))
            .unwrap(),
    );

    Ok((headers, buf))
}

async fn import_codelab(
    State(state): State<std::sync::Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Codelab>, (StatusCode, String)> {
    let mut zip_data = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() == Some("file") {
            zip_data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                .to_vec();
            break;
        }
    }

    if zip_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()));
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(zip_data))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut codelab: Option<Codelab> = None;
    let mut steps_content: Vec<(i32, String, String)> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let name = file.name().to_string();

        if name == "codelab.json" {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            codelab = Some(
                serde_json::from_str(&contents)
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
            );
        } else if name.ends_with(".md") && name.starts_with("step_") {
            // format: step_01_Title.md
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let parts: Vec<&str> = name.split('_').collect();
            if parts.len() >= 3 {
                let step_num: i32 = parts[1].parse().unwrap_or(0);
                let title = parts[2..].join("_").replace(".md", "").replace("_", " ");
                steps_content.push((step_num, title, contents));
            }
        }
    }

    let mut codelab =
        codelab.ok_or((StatusCode::BAD_REQUEST, "Missing codelab.json".to_string()))?;
    codelab.id = uuid::Uuid::new_v4().to_string(); // New ID for imported codelab

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("INSERT INTO codelabs (id, title, description, author) VALUES (?, ?, ?, ?)")
        .bind(&codelab.id)
        .bind(&codelab.title)
        .bind(&codelab.description)
        .bind(&codelab.author)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    steps_content.sort_by_key(|s| s.0);

    for (step_num, title, content) in steps_content {
        let step_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO steps (id, codelab_id, step_number, title, content_markdown) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&step_id)
        .bind(&codelab.id)
        .bind(step_num)
        .bind(&title)
        .bind(&content)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(codelab))
}
