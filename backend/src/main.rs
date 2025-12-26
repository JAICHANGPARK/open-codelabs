use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use axum_extra::extract::Multipart;
use dashmap::DashMap;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::io::{Cursor, Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
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

#[derive(Debug, Serialize, Deserialize)]
struct RegistrationPayload {
    name: String,
    code: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Attendee {
    id: String,
    codelab_id: String,
    name: String,
    code: String,
    created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HelpRequestPayload {
    step_number: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct HelpRequest {
    id: String,
    codelab_id: String,
    attendee_id: String,
    attendee_name: String,
    step_number: i32,
    status: String,
    created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct ChatMessageRow {
    id: String,
    codelab_id: String,
    sender_name: String,
    message: String,
    msg_type: String,
    target_id: Option<String>,
    created_at: Option<chrono::NaiveDateTime>,
}

struct AppState {
    pool: SqlitePool,
    admin_id: String,
    admin_pw: String,
    // Map of codelab_id -> broadcast sender
    channels: Arc<DashMap<String, broadcast::Sender<String>>>,
    // Map of (codelab_id, attendee_id) -> sender for DMs
    sessions: Arc<DashMap<(String, String), tokio::sync::mpsc::UnboundedSender<Message>>>,
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

    let state = Arc::new(AppState {
        pool,
        admin_id,
        admin_pw,
        channels: Arc::new(DashMap::new()),
        sessions: Arc::new(DashMap::new()),
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
        .route("/api/codelabs/:id/chat", get(get_chat_history))
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

async fn register_attendee(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegistrationPayload>,
) -> Result<Json<Attendee>, (StatusCode, String)> {
    // Check for duplicate name in the same codelab
    let existing = sqlx::query("SELECT id FROM attendees WHERE codelab_id = ? AND name = ?")
        .bind(&id)
        .bind(&payload.name)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Nickname already taken".to_string()));
    }

    let attendee_id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO attendees (id, codelab_id, name, code) VALUES (?, ?, ?, ?)")
        .bind(&attendee_id)
        .bind(&id)
        .bind(&payload.name)
        .bind(&payload.code)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let attendee = sqlx::query_as::<_, Attendee>("SELECT * FROM attendees WHERE id = ?")
        .bind(&attendee_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(attendee))
}

async fn get_attendees(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Attendee>>, (StatusCode, String)> {
    let attendees = sqlx::query_as::<_, Attendee>(
        "SELECT * FROM attendees WHERE codelab_id = ? ORDER BY created_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(attendees))
}

async fn request_help(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<HelpRequestPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let attendee_id = headers
        .get("X-Attendee-ID")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Attendee ID".to_string()))?;

    let help_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO help_requests (id, codelab_id, attendee_id, step_number) VALUES (?, ?, ?, ?)",
    )
    .bind(&help_id)
    .bind(&id)
    .bind(attendee_id)
    .bind(payload.step_number)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Notify via WebSocket if possible
    if let Some(res) = state.channels.get(&id) {
        let msg = serde_json::json!({
            "type": "help_request",
            "attendee_id": attendee_id,
            "step_number": payload.step_number
        })
        .to_string();
        let _ = res.send(msg);
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn get_help_requests(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<HelpRequest>>, (StatusCode, String)> {
    let requests = sqlx::query_as::<_, HelpRequest>(
        "SELECT hr.*, a.name as attendee_name FROM help_requests hr 
         JOIN attendees a ON hr.attendee_id = a.id 
         WHERE hr.codelab_id = ? AND hr.status = 'pending' 
         ORDER BY hr.created_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(requests))
}

async fn resolve_help_request(
    Path((codelab_id, help_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query("UPDATE help_requests SET status = 'resolved' WHERE id = ?")
        .bind(&help_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Notify via WebSocket
    if let Some(res) = state.channels.get(&codelab_id) {
        let msg = serde_json::json!({
            "type": "help_resolved",
            "id": help_id,
        })
        .to_string();
        let _ = res.send(msg);
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn get_chat_history(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ChatMessageRow>>, (StatusCode, String)> {
    let messages = sqlx::query_as::<_, ChatMessageRow>(
        "SELECT * FROM chat_messages WHERE codelab_id = ? ORDER BY created_at ASC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

async fn ws_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
}

async fn handle_socket(socket: WebSocket, codelab_id: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx_ws, mut rx_ws) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let mut user_id = String::new();

    // Listen for the first message to identify the user
    if let Some(Ok(Message::Text(text))) = receiver.next().await {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(id) = val.get("attendee_id").and_then(|v| v.as_str()) {
                user_id = id.to_string();
                state
                    .sessions
                    .insert((codelab_id.clone(), user_id.clone()), tx_ws);
            } else if val.get("type").and_then(|v| v.as_str()) == Some("facilitator") {
                user_id = "facilitator".to_string();
                state
                    .sessions
                    .insert((codelab_id.clone(), user_id.clone()), tx_ws);
            }
        }
    }

    if user_id.is_empty() {
        return;
    }

    let tx_broadcast = state
        .channels
        .entry(codelab_id.clone())
        .or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        })
        .clone();

    let mut rx_broadcast = tx_broadcast.subscribe();

    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Incoming from broadcast channel
                Ok(msg) = rx_broadcast.recv() => {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                // Incoming from direct message channel
                Some(msg) = rx_ws.recv() => {
                    if sender.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let state_clone = state.clone();
    let codelab_id_clone = codelab_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                match val.get("type").and_then(|v| v.as_str()) {
                    Some("chat") => {
                        // Broadcast chat
                        let sender = val
                            .get("sender")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown");
                        let message = val.get("message").and_then(|v| v.as_str()).unwrap_or("");

                        // Persist to DB
                        let msg_id = uuid::Uuid::new_v4().to_string();
                        let _ = sqlx::query("INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type) VALUES (?, ?, ?, ?, 'chat')")
                            .bind(&msg_id)
                            .bind(&codelab_id_clone)
                            .bind(sender)
                            .bind(message)
                            .execute(&state_clone.pool)
                            .await;

                        let _ = tx_broadcast.send(text.to_string());
                    }
                    Some("dm") => {
                        // Direct message
                        if let Some(target_id) = val.get("target_id").and_then(|v| v.as_str()) {
                            let sender = val
                                .get("sender")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown");
                            let message = val.get("message").and_then(|v| v.as_str()).unwrap_or("");

                            // Persist to DB
                            let msg_id = uuid::Uuid::new_v4().to_string();
                            let _ = sqlx::query("INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type, target_id) VALUES (?, ?, ?, ?, 'dm', ?)")
                                .bind(&msg_id)
                                .bind(&codelab_id_clone)
                                .bind(sender)
                                .bind(message)
                                .bind(target_id)
                                .execute(&state_clone.pool)
                                .await;

                            if let Some(target_ws) = state_clone
                                .sessions
                                .get(&(codelab_id_clone.clone(), target_id.to_string()))
                            {
                                let _ = target_ws.send(Message::Text(text.into()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
            state.sessions.remove(&(codelab_id, user_id));
        },
        _ = (&mut recv_task) => {
            send_task.abort();
            state.sessions.remove(&(codelab_id, user_id));
        },
    };
}
