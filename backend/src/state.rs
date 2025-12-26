use axum::extract::ws::Message;
use dashmap::DashMap;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct AppState {
    pub pool: SqlitePool,
    pub admin_id: String,
    pub admin_pw: String,
    // Map of codelab_id -> broadcast sender
    pub channels: Arc<DashMap<String, broadcast::Sender<String>>>,
    // Map of (codelab_id, attendee_id) -> sender for DMs
    pub sessions: Arc<DashMap<(String, String), tokio::sync::mpsc::UnboundedSender<Message>>>,
}
