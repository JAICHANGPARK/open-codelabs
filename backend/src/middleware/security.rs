use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::infrastructure::database::AppState;
use crate::middleware::auth::generate_csrf_token;
use crate::middleware::rate_limit::RateLimitConfig;
use crate::utils::error::{bad_request, too_many_requests};

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

    let has_session = jar.get(&state.auth.cookie_name).is_some()
        || jar.get(&state.auth.attendee_cookie_name).is_some();
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
            let text = value.to_str().unwrap_or("");
            let trimmed = text.split(',').next().unwrap_or("").trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    req.extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|info| info.0.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{AppState, DbKind};
    use axum::{
        body::Body,
        http::{HeaderValue, Request},
        middleware,
        response::IntoResponse,
        routing::{get, post},
        Router,
    };
    use sqlx::any::AnyPoolOptions;
    use std::time::Duration;
    use tower::util::ServiceExt;

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
    fn classify_rate_limit_selects_expected_bucket() {
        let config = RateLimitConfig {
            general_limit: 1,
            general_window: Duration::from_secs(1),
            login_limit: 2,
            login_window: Duration::from_secs(2),
            ai_limit: 3,
            ai_window: Duration::from_secs(3),
            upload_limit: 4,
            upload_window: Duration::from_secs(4),
        };

        assert_eq!(
            classify_rate_limit("/api/login", &Method::POST, &config),
            ("login", 2, Duration::from_secs(2))
        );
        assert_eq!(
            classify_rate_limit("/api/ai/stream", &Method::POST, &config),
            ("ai", 3, Duration::from_secs(3))
        );
        assert_eq!(
            classify_rate_limit("/api/upload/image", &Method::POST, &config),
            ("upload", 4, Duration::from_secs(4))
        );
        assert_eq!(
            classify_rate_limit("/api/codelabs/1/submissions", &Method::POST, &config),
            ("upload", 4, Duration::from_secs(4))
        );
        assert_eq!(
            classify_rate_limit("/api/codelabs", &Method::GET, &config),
            ("general", 1, Duration::from_secs(1))
        );
    }

    #[test]
    fn extract_client_ip_prefers_forwarded_for_when_trusted() {
        let req = Request::builder()
            .uri("/api/x")
            .header("x-forwarded-for", "203.0.113.10, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_ip(&req, true).as_deref(),
            Some("203.0.113.10")
        );
        assert_eq!(extract_client_ip(&req, false), None);
    }

    #[test]
    fn extract_client_ip_falls_back_when_forwarded_for_empty() {
        let mut req = Request::builder()
            .uri("/api/x")
            .header("x-forwarded-for", " ")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:8080"
                .parse::<std::net::SocketAddr>()
                .expect("socket"),
        ));
        assert_eq!(extract_client_ip(&req, true).as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn extract_client_ip_handles_non_utf8_forwarded_for() {
        let mut req = Request::builder()
            .uri("/api/x")
            .header(
                "x-forwarded-for",
                HeaderValue::from_bytes(&[0xFF]).expect("header"),
            )
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:9000"
                .parse::<std::net::SocketAddr>()
                .expect("socket"),
        ));

        assert_eq!(extract_client_ip(&req, true).as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn extract_client_ip_uses_connect_info_when_proxy_trusted_but_header_missing() {
        let mut req = Request::builder()
            .uri("/api/x")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.3:9000"
                .parse::<std::net::SocketAddr>()
                .expect("socket"),
        ));
        assert_eq!(extract_client_ip(&req, true).as_deref(), Some("127.0.0.3"));
    }

    #[tokio::test]
    async fn ensure_csrf_cookie_sets_cookie_once() {
        let state = make_state(false).await;
        let jar = CookieJar::new();
        let jar = ensure_csrf_cookie(jar, &state, Duration::from_secs(60));
        assert!(jar.get(&state.auth.csrf_cookie_name).is_some());

        let existing = jar
            .get(&state.auth.csrf_cookie_name)
            .map(|c| c.value().to_string())
            .unwrap();
        let jar = ensure_csrf_cookie(jar, &state, Duration::from_secs(60));
        let after = jar
            .get(&state.auth.csrf_cookie_name)
            .map(|c| c.value().to_string())
            .unwrap();
        assert_eq!(existing, after);
    }

    #[tokio::test]
    async fn csrf_middleware_rejects_missing_or_invalid_token() {
        let state = make_state(false).await;
        let app = Router::new()
            .route("/submit", post(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                csrf_middleware,
            ))
            .with_state(state.clone());

        let session_cookie = format!("{}=token", state.auth.cookie_name);

        let missing = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/submit")
                    .header("cookie", session_cookie.clone())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::BAD_REQUEST);

        let invalid = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/submit")
                    .header(
                        "cookie",
                        format!(
                            "{}; {}=cookie-token",
                            session_cookie, state.auth.csrf_cookie_name
                        ),
                    )
                    .header("x-csrf-token", "header-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn csrf_middleware_allows_safe_methods_and_matching_token() {
        let state = make_state(false).await;
        let app = Router::new()
            .route("/submit", post(|| async { "ok" }).get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                csrf_middleware,
            ))
            .with_state(state.clone());

        let get_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/submit")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);

        let token = "csrf-token";
        let post_res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/submit")
                    .header(
                        "cookie",
                        format!(
                            "{}=token; {}={}",
                            state.auth.cookie_name, state.auth.csrf_cookie_name, token
                        ),
                    )
                    .header("x-csrf-token", token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(post_res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rate_limit_middleware_blocks_when_exceeded() {
        let state = make_state(false).await;
        let app = Router::new()
            .route("/api/ping", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                rate_limit_middleware,
            ))
            .with_state(state.clone());

        for _ in 0..state.rate_limit_config.general_limit {
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/ping")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        let blocked = app
            .oneshot(
                Request::builder()
                    .uri("/api/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(blocked.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn security_headers_middleware_sets_expected_headers() {
        let state = make_state(true).await;
        let app = Router::new()
            .route("/api/ping", get(|| async { "ok".into_response() }))
            .route("/page", get(|| async { "ok".into_response() }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                security_headers_middleware,
            ))
            .with_state(state.clone());

        let api_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/ping")
                    .header("x-forwarded-proto", "https")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(api_res.status(), StatusCode::OK);
        assert_eq!(
            api_res
                .headers()
                .get(axum::http::header::CONTENT_SECURITY_POLICY)
                .unwrap(),
            "default-src 'none'"
        );
        assert!(api_res
            .headers()
            .contains_key(axum::http::header::STRICT_TRANSPORT_SECURITY));

        let page_res = app
            .oneshot(Request::builder().uri("/page").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(page_res.status(), StatusCode::OK);
        assert!(page_res
            .headers()
            .contains_key(axum::http::header::X_FRAME_OPTIONS));
        assert!(page_res
            .headers()
            .contains_key(axum::http::header::CONTENT_SECURITY_POLICY));
    }

    #[test]
    fn build_cors_layer_handles_default_and_custom_env() {
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
        let _default = build_cors_layer();

        std::env::set_var(
            "CORS_ALLOWED_ORIGINS",
            "http://localhost:3000, http://127.0.0.1:4173",
        );
        let _custom = build_cors_layer();
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }
}
