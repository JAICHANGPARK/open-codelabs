use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json;
use sqlx;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid;

pub async fn ws_handler(
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
                    Some("step_progress") => {
                        // Broadcast step progress to facilitators
                        let _ = tx_broadcast.send(text.to_string());
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
