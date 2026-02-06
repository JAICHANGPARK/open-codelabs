use axum::{
    body::Body,
    http::{header, HeaderMap, Request, StatusCode},
};
use backend::{
    create_router,
    domain::models::{Codelab, CreateCodelab},
    AppState, DbKind,
};
use cookie::Cookie;
use serde_json::{json, Value};
use sqlx::any::AnyPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use tower::util::ServiceExt; // for `oneshot`, `ready`, and `call`

struct TestApp {
    app: axum::Router,
    state: Arc<AppState>,
}

async fn setup_test_app() -> TestApp {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1) // Use 1 connection for in-memory sqlite to avoid issues
        .connect("sqlite::memory:?cache=shared")
        .await
        .expect("Failed to connect to in-memory sqlite");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = Arc::new(AppState::new(
        pool,
        DbKind::Sqlite,
        "admin".to_string(),
        "admin123".to_string(),
        false,
    ));

    let app = create_router(state.clone());
    TestApp { app, state }
}

fn extract_cookies(headers: &HeaderMap) -> (String, HashMap<String, String>) {
    let mut values = HashMap::new();
    let mut pairs = Vec::new();
    for value in headers.get_all(header::SET_COOKIE) {
        if let Ok(text) = value.to_str() {
            if let Ok(cookie) = Cookie::parse(text.to_string()) {
                let name = cookie.name().to_string();
                let val = cookie.value().to_string();
                values.insert(name.clone(), val.clone());
                pairs.push(format!("{}={}", name, val));
            }
        }
    }
    (pairs.join("; "), values)
}

async fn login_admin(app: &axum::Router, state: &AppState) -> (String, String) {
    let login_payload = json!({
        "admin_id": state.admin_id.clone(),
        "admin_pw": state.admin_pw.clone()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let (cookie_header, cookies) = extract_cookies(response.headers());
    let csrf_token = cookies
        .get(&state.auth.csrf_cookie_name)
        .cloned()
        .expect("csrf cookie missing");
    (cookie_header, csrf_token)
}

#[tokio::test]
async fn test_list_codelabs_empty() {
    let test_app = setup_test_app().await;

    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .uri("/api/codelabs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let codelabs: Vec<Codelab> = serde_json::from_slice(&body).unwrap();
    assert_eq!(codelabs.len(), 0);
}

#[tokio::test]
async fn test_create_and_get_codelab() {
    let test_app = setup_test_app().await;
    let (cookie_header, csrf_token) = login_admin(&test_app.app, &test_app.state).await;

    // 1. Create a codelab
    let create_payload = CreateCodelab {
        title: "Test Codelab".to_string(),
        description: "Test Description".to_string(),
        author: "Test Author".to_string(),
        is_public: None,
        quiz_enabled: None,
        require_quiz: None,
        require_feedback: None,
        guide_markdown: None,
    };

    let response = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, cookie_header)
                .header("x-csrf-token", csrf_token)
                .body(Body::from(serde_json::to_string(&create_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        panic!(
            "Create codelab failed: {} - {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: Codelab = serde_json::from_slice(&body).unwrap();
    assert_eq!(created.title, "Test Codelab");
    let codelab_id = created.id;

    // 2. Get the created codelab
    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}", codelab_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let (codelab, steps): (Codelab, Vec<Value>) = serde_json::from_slice(&body).unwrap();
    assert_eq!(codelab.id, codelab_id);
    assert_eq!(codelab.title, "Test Codelab");
    assert_eq!(steps.len(), 0);
}

#[tokio::test]
async fn test_login() {
    let test_app = setup_test_app().await;

    let login_payload = json!({
        "admin_id": "admin",
        "admin_pw": "admin123"
    });

    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let res_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res_json["status"], "ok");
}

#[tokio::test]
async fn test_login_failure() {
    let test_app = setup_test_app().await;

    let login_payload = json!({
        "admin_id": "admin",
        "admin_pw": "wrong_password"
    });

    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
