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
use sqlx::any::AnyPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use tower::util::ServiceExt;

struct TestApp {
    app: axum::Router,
    state: Arc<AppState>,
}

async fn setup_test_app() -> TestApp {
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
    let login_payload = serde_json::json!({
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
async fn test_codelab_visibility() {
    let test_app = setup_test_app().await;
    let (cookie_header, csrf_token) = login_admin(&test_app.app, &test_app.state).await;

    // 1. Create a public codelab
    let public_payload = CreateCodelab {
        title: "Public Codelab".to_string(),
        description: "Public".to_string(),
        author: "Author".to_string(),
        is_public: Some(true),
        quiz_enabled: None,
        require_quiz: None,
        require_feedback: None,
        guide_markdown: None,
    };
    test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, cookie_header.clone())
                .header("x-csrf-token", csrf_token.clone())
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
        quiz_enabled: None,
        require_quiz: None,
        require_feedback: None,
        guide_markdown: None,
    };
    let res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, cookie_header.clone())
                .header("x-csrf-token", csrf_token.clone())
                .body(Body::from(serde_json::to_string(&private_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let private_created: Codelab = serde_json::from_slice(&body).unwrap();
    let private_id = private_created.id;

    // 3. List codelabs as public user (no auth)
    let response = test_app
        .app
        .clone()
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

    // Should only contain the public one
    assert_eq!(
        codelabs.len(),
        1,
        "Public user should only see 1 codelab, found {}",
        codelabs.len()
    );
    assert_eq!(codelabs[0].title, "Public Codelab");

    // 4. Try to get private codelab as public user
    let response = test_app
        .app
        .clone()
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
    let response = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/codelabs")
                .header(header::COOKIE, cookie_header)
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
    assert_eq!(
        codelabs.len(),
        2,
        "Admin should see 2 codelabs, found {}",
        codelabs.len()
    );
}
