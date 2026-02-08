use crate::middleware::auth::{AuthSession, Role};
use crate::utils::error::forbidden;
use crate::infrastructure::database::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json;
use sqlx;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid;

/// Generate a unique session ID for each WebSocket connection
fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(serde::Deserialize, Default)]
pub struct WsQuery {
    #[serde(rename = "as")]
    role_hint: Option<String>,
}

pub async fn ws_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
    session: AuthSession,
) -> impl IntoResponse {
    let claims = match query.role_hint.as_deref() {
        Some("admin") => session.admin_claims,
        Some("attendee") => session.attendee_claims,
        _ => session.claims,
    };

    let claims = match claims {
        Some(claims) => claims,
        None => return axum::http::StatusCode::UNAUTHORIZED.into_response(),
    };
    if claims.role == Role::Attendee.as_str() {
        if claims.codelab_id.as_deref() != Some(id.as_str()) {
            return forbidden().into_response();
        }
    }
    ws.on_upgrade(move |socket| handle_socket(socket, id, state, claims))
}

async fn handle_socket(
    socket: WebSocket,
    codelab_id: String,
    state: Arc<AppState>,
    claims: crate::middleware::auth::SessionClaims,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx_ws, mut rx_ws) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let session_id = generate_session_id();

    let (user_id, sender_name, role) = if Role::from_str(&claims.role) == Some(Role::Admin) {
        (
            "facilitator".to_string(),
            "Facilitator".to_string(),
            Role::Admin,
        )
    } else {
        let attendee_id = claims.sub.clone();
        let name: Option<(String,)> =
            sqlx::query_as(&state.q("SELECT name FROM attendees WHERE id = ?"))
                .bind(&attendee_id)
                .fetch_optional(&state.pool)
                .await
                .ok()
                .flatten();
        let display_name = name
            .map(|row| row.0)
            .unwrap_or_else(|| "Attendee".to_string());
        (attendee_id, display_name, Role::Attendee)
    };

    // Add session to the list of sessions for this user (supports multiple tabs)
    let session_key = (codelab_id.clone(), user_id.clone());
    state
        .sessions
        .entry(session_key.clone())
        .and_modify(|sessions| sessions.push((session_id.clone(), tx_ws.clone())))
        .or_insert_with(|| vec![(session_id.clone(), tx_ws.clone())]);

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
    let sender_name_clone = sender_name.clone();
    let user_id_clone = user_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                match val.get("type").and_then(|v| v.as_str()) {
                    Some("chat") => {
                        // Broadcast chat
                        let message = val.get("message").and_then(|v| v.as_str()).unwrap_or("");
                        if message.len() > 2000 {
                            continue;
                        }

                        // Persist to DB
                        let msg_id = uuid::Uuid::new_v4().to_string();
                        let _ = sqlx::query(&state_clone.q("INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type) VALUES (?, ?, ?, ?, 'chat')"))
                            .bind(&msg_id)
                            .bind(&codelab_id_clone)
                            .bind(&sender_name_clone)
                            .bind(message)
                            .execute(&state_clone.pool)
                            .await;

                        let payload = serde_json::json!({
                            "type": "chat",
                            "sender": sender_name_clone.as_str(),
                            "message": message
                        })
                        .to_string();
                        let _ = tx_broadcast.send(payload);
                    }
                    Some("dm") => {
                        // Direct message
                        if let Some(target_id) = val.get("target_id").and_then(|v| v.as_str()) {
                            let message = val.get("message").and_then(|v| v.as_str()).unwrap_or("");
                            if message.len() > 2000 {
                                continue;
                            }

                            // Persist to DB with sender_id
                            let msg_id = uuid::Uuid::new_v4().to_string();
                            let _ = sqlx::query(&state_clone.q("INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type, target_id, sender_id) VALUES (?, ?, ?, ?, 'dm', ?, ?)"))
                                .bind(&msg_id)
                                .bind(&codelab_id_clone)
                                .bind(&sender_name_clone)
                                .bind(message)
                                .bind(target_id)
                                .bind(&user_id_clone)
                                .execute(&state_clone.pool)
                                .await;

                            // Send DM to all sessions of the target user (supports multiple tabs)
                            if let Some(target_sessions) = state_clone
                                .sessions
                                .get(&(codelab_id_clone.clone(), target_id.to_string()))
                            {
                                let payload = serde_json::json!({
                                    "type": "dm",
                                    "sender": sender_name_clone.as_str(),
                                    "message": message,
                                    "target_id": target_id,
                                    "sender_id": user_id_clone.as_str()
                                })
                                .to_string();
                                for (_, target_ws) in target_sessions.iter() {
                                    let _ = target_ws.send(Message::Text(payload.clone().into()));
                                }
                            }
                        }
                    }
                    Some("step_progress") => {
                        // Persist to DB
                        if role == Role::Attendee {
                            let step_number = val.get("step_number").and_then(|v| v.as_i64());
                            if let Some(step_number) = step_number {
                                if step_number < 1 {
                                    continue;
                                }
                                let attendee_id = user_id_clone.as_str();
                                let _ = sqlx::query(
                                    &state_clone
                                        .q("UPDATE attendees SET current_step = ? WHERE id = ?"),
                                )
                                .bind(step_number as i32)
                                .bind(attendee_id)
                                .execute(&state_clone.pool)
                                .await;
                                let payload = serde_json::json!({
                                    "type": "step_progress",
                                    "attendee_id": attendee_id,
                                    "step_number": step_number
                                })
                                .to_string();
                                let _ = tx_broadcast.send(payload);
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
            cleanup_session(&state, &codelab_id, &user_id, &session_id);
        },
        _ = (&mut recv_task) => {
            send_task.abort();
            cleanup_session(&state, &codelab_id, &user_id, &session_id);
        },
    };
}

/// Remove a specific session from the sessions map
fn cleanup_session(
    state: &Arc<AppState>,
    codelab_id: &str,
    user_id: &str,
    session_id: &str,
) {
    let key = (codelab_id.to_string(), user_id.to_string());
    if let Some(mut sessions) = state.sessions.get_mut(&key) {
        // Remove the specific session by session_id
        sessions.retain(|(sid, _)| sid != session_id);
        if sessions.is_empty() {
            drop(sessions);
            state.sessions.remove(&key);
        }
    }
}
