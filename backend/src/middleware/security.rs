use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::middleware::auth::generate_csrf_token;
use crate::utils::error::{bad_request, too_many_requests};
use crate::middleware::rate_limit::RateLimitConfig;
use crate::infrastructure::database::AppState;

#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    pub content_security_policy: String,
    pub hsts: String,
}

impl SecurityHeadersConfig {
    pub fn from_env() -> Self {
        let csp_default = [
            "default-src 'self'",
            "base-uri 'self'",
            "frame-ancestors 'none'",
            "object-src 'none'",
            "img-src 'self' data: blob:",
            "script-src 'self' 'unsafe-inline'",
            "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
            "font-src 'self' https://fonts.gstatic.com",
            "connect-src 'self' https://generativelanguage.googleapis.com ws: wss:",
            "form-action 'self'",
        ]
        .join("; ");

        let content_security_policy = std::env::var("CSP_HEADER").unwrap_or_else(|_| csp_default);
        let hsts = std::env::var("HSTS_HEADER")
            .unwrap_or_else(|_| "max-age=63072000; includeSubDomains; preload".to_string());

        Self {
            content_security_policy,
            hsts,
        }
    }
}

pub fn build_cors_layer() -> CorsLayer {
    let allow_origins = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
    if allow_origins.trim().is_empty() {
        CorsLayer::new()
            .allow_origin([
                "http://localhost:5173".parse().unwrap(),
                "http://127.0.0.1:5173".parse().unwrap(),
                "http://localhost:8080".parse().unwrap(),
                "http://127.0.0.1:8080".parse().unwrap(),
            ])
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::HeaderName::from_static("x-csrf-token"),
            ])
            .allow_credentials(true)
    } else {
        let origins: Vec<_> = allow_origins
            .split(',')
            .filter_map(|value| value.trim().parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::HeaderName::from_static("x-csrf-token"),
            ])
            .allow_credentials(true)
    }
}

pub async fn security_headers_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let is_api = path.starts_with("/api");
    let forwarded_proto = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let is_https = req.uri().scheme_str() == Some("https")
        || (state.trust_proxy && forwarded_proto.eq_ignore_ascii_case("https"));

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers
        .entry(header::X_CONTENT_TYPE_OPTIONS)
        .or_insert_with(|| header::HeaderValue::from_static("nosniff"));
    headers
        .entry(header::X_FRAME_OPTIONS)
        .or_insert_with(|| header::HeaderValue::from_static("DENY"));
    headers
        .entry(header::REFERRER_POLICY)
        .or_insert_with(|| header::HeaderValue::from_static("strict-origin-when-cross-origin"));
    headers
        .entry(header::HeaderName::from_static("permissions-policy"))
        .or_insert_with(|| {
            header::HeaderValue::from_static("camera=(), microphone=(), geolocation=()")
        });

    if is_api {
        headers
            .entry(header::CONTENT_SECURITY_POLICY)
            .or_insert_with(|| header::HeaderValue::from_static("default-src 'none'"));
    } else {
        headers
            .entry(header::CONTENT_SECURITY_POLICY)
            .or_insert_with(|| {
                header::HeaderValue::from_str(&state.security_headers.content_security_policy)
                    .unwrap_or_else(|_| header::HeaderValue::from_static("default-src 'self'"))
            });
    }

    if is_https {
        headers
            .entry(header::STRICT_TRANSPORT_SECURITY)
            .or_insert_with(|| {
                header::HeaderValue::from_str(&state.security_headers.hsts)
                    .unwrap_or_else(|_| header::HeaderValue::from_static("max-age=31536000"))
            });
    }

    response
}

pub async fn csrf_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    if matches!(
        req.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    ) {
        return Ok(next.run(req).await);
    }

    let (parts, body) = req.into_parts();
    let jar = CookieJar::from_headers(&parts.headers);

    let has_session = jar.get(&state.auth.cookie_name).is_some();
    if has_session {
        let csrf_cookie = jar
            .get(&state.auth.csrf_cookie_name)
            .map(|cookie| cookie.value().to_string());
        let csrf_header = parts
            .headers
            .get("x-csrf-token")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        if csrf_cookie.is_none() {
            return Err(bad_request("csrf token missing"));
        }
        if csrf_cookie != csrf_header {
            return Err(bad_request("csrf token invalid"));
        }
    }

    let req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}

pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let path = req.uri().path();
    let method = req.method();
    let ip = extract_client_ip(&req, state.trust_proxy).unwrap_or_else(|| "unknown".to_string());

    let (bucket, limit, window) = classify_rate_limit(path, method, &state.rate_limit_config);
    let key = format!("{}:{}", bucket, ip);
    if !state.rate_limiter.check(&key, limit, window) {
        return Err(too_many_requests());
    }

    Ok(next.run(req).await)
}

pub fn ensure_csrf_cookie(
    jar: CookieJar,
    state: &AppState,
    max_age: std::time::Duration,
) -> CookieJar {
    if jar.get(&state.auth.csrf_cookie_name).is_some() {
        return jar;
    }
    let token = generate_csrf_token();
    let cookie = crate::middleware::auth::build_csrf_cookie(&state.auth, token, max_age);
    jar.add(cookie)
}

fn classify_rate_limit<'a>(
    path: &'a str,
    method: &Method,
    config: &RateLimitConfig,
) -> (&'a str, u32, std::time::Duration) {
    if path.starts_with("/api/login") {
        return ("login", config.login_limit, config.login_window);
    }
    if path.starts_with("/api/ai/") {
        return ("ai", config.ai_limit, config.ai_window);
    }
    if path.starts_with("/api/upload") || (path.contains("/submissions") && method == Method::POST)
    {
        return ("upload", config.upload_limit, config.upload_window);
    }

    ("general", config.general_limit, config.general_window)
}

fn extract_client_ip<B>(req: &Request<B>, trust_proxy: bool) -> Option<String> {
    if trust_proxy {
        if let Some(value) = req.headers().get("x-forwarded-for") {
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

    req.extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|info| info.0.ip().to_string())
}
