use axum::extract::ConnectInfo;
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::error::unauthorized;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub ip: String,
    pub user_agent: Option<String>,
}

impl<S> FromRequestParts<S> for RequestInfo
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(state) = State::<Arc<AppState>>::from_request_parts(parts, state)
            .await
            .map_err(|_| unauthorized())?;

        let user_agent = parts
            .headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let ip = extract_ip(parts, state.trust_proxy).unwrap_or_else(|| "unknown".to_string());

        Ok(Self { ip, user_agent })
    }
}

fn extract_ip(parts: &Parts, trust_proxy: bool) -> Option<String> {
    if trust_proxy {
        if let Some(value) = parts.headers.get("x-forwarded-for") {
            if let Ok(text) = value.to_str() {
                if let Some(first) = text.split(',').next() {
                    let trimmed = first.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }

    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
}
