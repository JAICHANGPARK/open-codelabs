use axum::extract::ConnectInfo;
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::infrastructure::database::AppState;
use crate::utils::error::unauthorized;

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
            let text = value.to_str().unwrap_or("");
            let trimmed = text.split(',').next().unwrap_or("").trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{AppState, DbKind};
    use axum::{
        body::Body,
        http::{HeaderValue, Request},
    };
    use sqlx::any::AnyPoolOptions;

    async fn make_state(trust_proxy: bool) -> Arc<AppState> {
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
            trust_proxy,
        ))
    }

    #[test]
    fn extract_ip_prefers_forwarded_for_when_proxy_trusted() {
        let mut req = Request::builder()
            .uri("/api/test")
            .header("x-forwarded-for", "203.0.113.8, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo("127.0.0.1:1234".parse::<SocketAddr>().unwrap()));
        let (parts, _) = req.into_parts();

        assert_eq!(extract_ip(&parts, true).as_deref(), Some("203.0.113.8"));
    }

    #[test]
    fn extract_ip_uses_connect_info_when_proxy_untrusted() {
        let mut req = Request::builder()
            .uri("/api/test")
            .header("x-forwarded-for", "203.0.113.8")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo("127.0.0.1:1234".parse::<SocketAddr>().unwrap()));
        let (parts, _) = req.into_parts();

        assert_eq!(extract_ip(&parts, false).as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn extract_ip_uses_connect_info_when_proxy_trusted_but_no_header() {
        let mut req = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo("127.0.0.2:1234".parse::<SocketAddr>().unwrap()));
        let (parts, _) = req.into_parts();

        assert_eq!(extract_ip(&parts, true).as_deref(), Some("127.0.0.2"));
    }

    #[test]
    fn extract_ip_ignores_empty_forwarded_for() {
        let req = Request::builder()
            .uri("/api/test")
            .header("x-forwarded-for", "   ")
            .body(Body::empty())
            .unwrap();
        let (parts, _) = req.into_parts();
        assert_eq!(extract_ip(&parts, true), None);
    }

    #[test]
    fn extract_ip_handles_non_utf8_forwarded_for() {
        let req = Request::builder()
            .uri("/api/test")
            .header(
                "x-forwarded-for",
                HeaderValue::from_bytes(&[0xFF]).expect("header"),
            )
            .body(Body::empty())
            .unwrap();
        let (parts, _) = req.into_parts();
        assert_eq!(extract_ip(&parts, true), None);
    }

    #[tokio::test]
    async fn from_request_parts_populates_fields() {
        let state = make_state(false).await;
        let req = Request::builder()
            .uri("/api/test")
            .header(axum::http::header::USER_AGENT, "curl/8.0")
            .body(Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();

        let info = RequestInfo::from_request_parts(&mut parts, &state)
            .await
            .expect("extract request info");
        assert_eq!(info.ip, "unknown");
        assert_eq!(info.user_agent.as_deref(), Some("curl/8.0"));
    }
}
