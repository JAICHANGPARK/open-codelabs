use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{create_router, models::{Codelab, CreateCodelab}, AppState, DbKind};
use sqlx::any::AnyPoolOptions;
use std::sync::Arc;
use tower::util::ServiceExt;

async fn setup_test_app() -> axum::Router {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:?cache=shared")
        .await
        .expect("Failed to connect to in-memory sqlite");

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
async fn test_codelab_visibility() {
    let app = setup_test_app().await;

    // 1. Create a public codelab
    let public_payload = CreateCodelab {
        title: "Public Codelab".to_string(),
        description: "Public".to_string(),
        author: "Author".to_string(),
        is_public: Some(true),
    };
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&public_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 2. Create a private codelab
    let private_payload = CreateCodelab {
        title: "Private Codelab".to_string(),
        description: "Private".to_string(),
        author: "Author".to_string(),
        is_public: Some(false),
    };
    let res = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&private_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let private_created: Codelab = serde_json::from_slice(&body).unwrap();
    let private_id = private_created.id;

    // 3. List codelabs as public user (no auth)
    let response = app.clone()
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
    
    // Should only contain the public one
    assert_eq!(codelabs.len(), 1, "Public user should only see 1 codelab, found {}", codelabs.len());
    assert_eq!(codelabs[0].title, "Public Codelab");

    // 4. Try to get private codelab as public user
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}", private_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // 5. List codelabs as admin
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/api/codelabs")
                .header("Authorization", "Bearer mock-jwt-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let codelabs: Vec<Codelab> = serde_json::from_slice(&body).unwrap();
    assert_eq!(codelabs.len(), 2, "Admin should see 2 codelabs, found {}", codelabs.len());
}
