use crate::infrastructure::database::AppState;
use crate::middleware::auth::{AuthSession, Role};
use crate::utils::error::forbidden;
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
    pub role_hint: Option<String>,
    pub token: Option<String>,
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

    // If session claims are missing, try the token from query string
    let claims = match claims {
        Some(c) => Some(c),
        None => query.token.as_ref().and_then(|t| {
            state.auth.verify_token(t).filter(|c| {
                // Ensure the token role matches the requested role hint if provided
                match query.role_hint.as_deref() {
                    Some("admin") => c.role == Role::Admin.as_str(),
                    Some("attendee") => {
                        c.role == Role::Attendee.as_str()
                            && c.codelab_id.as_deref() == Some(id.as_str())
                    }
                    _ => true,
                }
            })
        }),
    };

    let claims = match claims {
        Some(claims) => claims,
        None => {
            eprintln!(
                "WebSocket: Unauthorized connection attempt to codelab {}",
                id
            );
            return axum::http::StatusCode::UNAUTHORIZED.into_response();
        }
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

    // Check if screen sharing is active and notify the new connection
    if let Some(is_active) = state.active_screen_shares.get(&codelab_id) {
        if *is_active {
            let payload = serde_json::json!({
                "type": "screen_share_status",
                "status": "facilitator_started"
            })
            .to_string();
            let _ = tx_ws.send(Message::Text(payload.into()));
        }
    }

    let mut send_task = tokio::spawn(async move {
        loop {
            let outbound = tokio::select! {
                // Incoming from broadcast channel
                Ok(msg) = rx_broadcast.recv() => {
                    Message::Text(msg.into())
                }
                // Incoming from direct message channel
                Some(msg) = rx_ws.recv() => {
                    msg
                }
            };
            if sender.send(outbound).await.is_err() {
                return;
            }
        }
    });

    let state_clone = state.clone();
    let codelab_id_clone = codelab_id.clone();
    let sender_name_clone = sender_name.clone();
    let user_id_clone = user_id.clone();
    let role_clone = role.clone();
    let tx_broadcast_clone = tx_broadcast.clone();
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
                        let _ = tx_broadcast_clone.send(payload);
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
                        if role_clone != Role::Attendee {
                            continue;
                        }
                        let Some(step_number) = val.get("step_number").and_then(|v| v.as_i64())
                        else {
                            continue;
                        };
                        if step_number < 1 {
                            continue;
                        }
                        let attendee_id = user_id_clone.as_str();
                        let _ = sqlx::query(
                            &state_clone.q("UPDATE attendees SET current_step = ? WHERE id = ?"),
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
                        let _ = tx_broadcast_clone.send(payload);
                    }
                    Some("webrtc_signal") => {
                        // WebRTC signaling (Offer/Answer/ICE)
                        if let Some(target_id) = val.get("target_id").and_then(|v| v.as_str()) {
                            // Direct signaling
                            if let Some(target_sessions) = state_clone
                                .sessions
                                .get(&(codelab_id_clone.clone(), target_id.to_string()))
                            {
                                let payload = serde_json::json!({
                                    "type": "webrtc_signal",
                                    "sender_id": user_id_clone.as_str(),
                                    "sender_name": sender_name_clone.as_str(),
                                    "signal": val.get("signal"),
                                    "stream_type": val.get("stream_type")
                                })
                                .to_string();
                                for (_, target_ws) in target_sessions.iter() {
                                    let _ = target_ws.send(Message::Text(payload.clone().into()));
                                }
                            }
                        } else {
                            // Broadcast signaling (Facilitator to all)
                            let payload = serde_json::json!({
                                "type": "webrtc_signal",
                                "sender_id": user_id_clone.as_str(),
                                "sender_name": sender_name_clone.as_str(),
                                "signal": val.get("signal"),
                                "stream_type": val.get("stream_type")
                            })
                            .to_string();
                            let _ = tx_broadcast_clone.send(payload);
                        }
                    }
                    Some("screen_share_status") => {
                        // Broadcast screen share status
                        let status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

                        if status == "facilitator_started" {
                            state_clone
                                .active_screen_shares
                                .insert(codelab_id_clone.clone(), true);
                        } else if status == "facilitator_stopped" {
                            state_clone
                                .active_screen_shares
                                .insert(codelab_id_clone.clone(), false);
                        }

                        let payload = serde_json::json!({
                            "type": "screen_share_status",
                            "sender_id": user_id_clone.as_str(),
                            "status": status
                        })
                        .to_string();
                        let _ = tx_broadcast_clone.send(payload);
                    }
                    Some("attendee_screen_status") => {
                        // Attendee notifies facilitator about their screen sharing status
                        if role_clone != Role::Attendee {
                            continue;
                        }
                        let status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");
                        let is_sharing = status == "started";
                        state_clone.attendee_sharing.insert(
                            (codelab_id_clone.clone(), user_id_clone.clone()),
                            is_sharing,
                        );

                        if let Some(facilitator_sessions) = state_clone
                            .sessions
                            .get(&(codelab_id_clone.clone(), "facilitator".to_string()))
                        {
                            let payload = serde_json::json!({
                                "type": "attendee_screen_status",
                                "attendee_id": user_id_clone.as_str(),
                                "attendee_name": sender_name_clone.as_str(),
                                "status": status
                            })
                            .to_string();
                            for (_, facilitator_ws) in facilitator_sessions.iter() {
                                let _ = facilitator_ws.send(Message::Text(payload.clone().into()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    // Broadcast join event if it's an attendee
    if role == Role::Attendee {
        // Fetch latest step to be accurate
        let current_step: i32 =
            sqlx::query_scalar(&state.q("SELECT current_step FROM attendees WHERE id = ?"))
                .bind(&user_id)
                .fetch_one(&state.pool)
                .await
                .unwrap_or(0);

        let payload = serde_json::json!({
            "type": "attendee_joined",
            "attendee": {
                "id": user_id,
                "name": sender_name,
                "created_at": chrono::Utc::now().to_rfc3339(),
                "current_step": current_step
            }
        })
        .to_string();
        let _ = tx_broadcast.send(payload);
    }

    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }
    send_task.abort();
    recv_task.abort();

    // Cleanup and broadcast leave
    cleanup_session(&state, &codelab_id, &user_id, &session_id);

    // Check if this was the last session for this user before broadcasting leave
    // (To avoid "left" message on tab refresh if another tab is open, though checking session count is tricky here due to async race,
    // but we can check if sessions map is empty for this user)
    let session_key = (codelab_id.clone(), user_id.clone());
    let is_last_session = !state.sessions.contains_key(&session_key);

    if role == Role::Attendee && is_last_session {
        let payload = serde_json::json!({
            "type": "attendee_left",
            "attendee_id": user_id
        })
        .to_string();
        let _ = tx_broadcast.send(payload);
    }
}

/// Remove a specific session from the sessions map
fn cleanup_session(state: &Arc<AppState>, codelab_id: &str, user_id: &str, session_id: &str) {
    let key = (codelab_id.to_string(), user_id.to_string());
    if let Some(mut sessions) = state.sessions.get_mut(&key) {
        // Remove the specific session by session_id
        sessions.retain(|(sid, _)| sid != session_id);
        if sessions.is_empty() {
            drop(sessions);
            state.sessions.remove(&key);
            // Also cleanup sharing status
            state.attendee_sharing.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{AppState, DbKind};
    use sqlx::any::AnyPoolOptions;

    async fn make_state() -> Arc<AppState> {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("sqlite");
        Arc::new(AppState::new(
            pool,
            DbKind::Sqlite,
            "admin".to_string(),
            "pw".to_string(),
            false,
        ))
    }

    #[test]
    fn generate_session_id_returns_uuid_like_value() {
        let a = generate_session_id();
        let b = generate_session_id();
        assert_ne!(a, b);
        assert!(uuid::Uuid::parse_str(&a).is_ok());
        assert!(uuid::Uuid::parse_str(&b).is_ok());
    }

    #[tokio::test]
    async fn cleanup_session_removes_only_target_and_cleans_last_entry() {
        let state = make_state().await;
        let key = ("lab-1".to_string(), "attendee-1".to_string());
        let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();

        state.sessions.insert(
            key.clone(),
            vec![("s1".to_string(), tx1), ("s2".to_string(), tx2)],
        );
        state.attendee_sharing.insert(key.clone(), true);

        cleanup_session(&state, "lab-1", "attendee-1", "s1");
        let sessions = state.sessions.get(&key).expect("sessions remain");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].0, "s2");
        drop(sessions);

        cleanup_session(&state, "lab-1", "attendee-1", "s2");
        assert!(!state.sessions.contains_key(&key));
        assert!(!state.attendee_sharing.contains_key(&key));
    }

    #[tokio::test]
    async fn cleanup_session_noop_when_user_has_no_sessions() {
        let state = make_state().await;
        cleanup_session(&state, "missing-lab", "missing-user", "missing-session");
        assert!(state.sessions.is_empty());
    }
}
