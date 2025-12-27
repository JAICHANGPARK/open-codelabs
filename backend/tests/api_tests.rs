use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{create_router, models::{Codelab, CreateCodelab}, AppState, DbKind};
use serde_json::{json, Value};
use sqlx::any::AnyPoolOptions;
use std::sync::Arc;
use tower::util::ServiceExt; // for `oneshot`, `ready`, and `call`

async fn setup_test_app() -> axum::Router {
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

    let state = Arc::new(AppState {
        pool,
        db_kind: DbKind::Sqlite,
        admin_id: "admin".to_string(),
        admin_pw: "admin123".to_string(),
        channels: Arc::new(dashmap::DashMap::new()),
        sessions: Arc::new(dashmap::DashMap::new()),
    });

    create_router(state)
}

#[tokio::test]
async fn test_list_codelabs_empty() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/codelabs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let codelabs: Vec<Codelab> = serde_json::from_slice(&body).unwrap();
    assert_eq!(codelabs.len(), 0);
}

#[tokio::test]
async fn test_create_and_get_codelab() {
    let app = setup_test_app().await;

    // 1. Create a codelab
    let create_payload = CreateCodelab {
        title: "Test Codelab".to_string(),
        description: "Test Description".to_string(),
        author: "Test Author".to_string(),
        is_public: None,
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&create_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        panic!("Create codelab failed: {} - {}", status, String::from_utf8_lossy(&body));
    }
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: Codelab = serde_json::from_slice(&body).unwrap();
    assert_eq!(created.title, "Test Codelab");
    let codelab_id = created.id;

    // 2. Get the created codelab
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}", codelab_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let (codelab, steps): (Codelab, Vec<Value>) = serde_json::from_slice(&body).unwrap();
    assert_eq!(codelab.id, codelab_id);
    assert_eq!(codelab.title, "Test Codelab");
    assert_eq!(steps.len(), 0);
}

#[tokio::test]
async fn test_login() {
    let app = setup_test_app().await;

    let login_payload = json!({
        "admin_id": "admin",
        "admin_pw": "admin123"
    });

    let response = app
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res_json["status"], "ok");
}

#[tokio::test]
async fn test_login_failure() {
    let app = setup_test_app().await;

    let login_payload = json!({
        "admin_id": "admin",
        "admin_pw": "wrong_password"
    });

    let response = app
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
