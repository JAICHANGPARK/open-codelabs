use axum::{
    body::Body,
    http::{header, HeaderMap, Request, StatusCode},
};
use backend::{
    create_router,
    domain::models::{Codelab, CreateCodelab},
    middleware::auth::SessionClaims,
    utils::crypto::encrypt_with_password,
    AppState, DbKind,
};
use cookie::Cookie;
use futures_util::{SinkExt, StreamExt};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use serde_json::{json, Value};
use sqlx::any::AnyPoolOptions;
use std::cell::Cell;
use std::collections::HashMap;
use std::io::{Cursor as IoCursor, Write};
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
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

async fn bind_local_listener_or_skip() -> Option<tokio::net::TcpListener> {
    match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => Some(listener),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skip websocket test: bind permission denied: {err}");
            None
        }
        Err(err) => panic!("failed to bind websocket test listener: {err}"),
    }
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

async fn create_codelab_as_admin(
    app: &axum::Router,
    admin_cookie: &str,
    admin_csrf: &str,
    require_submission: bool,
) -> Codelab {
    create_codelab_with_options(app, admin_cookie, admin_csrf, true, require_submission).await
}

async fn create_codelab_with_options(
    app: &axum::Router,
    admin_cookie: &str,
    admin_csrf: &str,
    is_public: bool,
    require_submission: bool,
) -> Codelab {
    let create_payload = CreateCodelab {
        title: "Flow Codelab".to_string(),
        description: "Flow Description".to_string(),
        author: "Flow Author".to_string(),
        is_public: Some(is_public),
        quiz_enabled: Some(true),
        require_quiz: Some(false),
        require_feedback: Some(false),
        require_submission: Some(require_submission),
        guide_markdown: Some("# Guide".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie)
                .header("x-csrf-token", admin_csrf)
                .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice::<Codelab>(&body).unwrap()
}

async fn register_attendee(
    app: &axum::Router,
    state: &AppState,
    codelab_id: &str,
    name: &str,
    code: &str,
) -> (String, String, String) {
    let payload = json!({
        "name": name,
        "code": code,
        "email": "alice@example.com"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/register", codelab_id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let (cookie_header, cookies) = extract_cookies(response.headers());
    let csrf_token = cookies
        .get(&state.auth.csrf_cookie_name)
        .cloned()
        .expect("attendee csrf cookie missing");
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let attendee: Value = serde_json::from_slice(&body).unwrap();
    let attendee_id = attendee["id"].as_str().unwrap().to_string();

    (cookie_header, csrf_token, attendee_id)
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
    _lock: Option<MutexGuard<'static, ()>>,
}

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
thread_local! {
    static ENV_LOCK_DEPTH: Cell<usize> = const { Cell::new(0) };
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<str>) -> Self {
        let lock = ENV_LOCK_DEPTH.with(|depth| {
            let current = depth.get();
            depth.set(current + 1);
            if current == 0 {
                Some(ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner()))
            } else {
                None
            }
        });
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value.as_ref());
        Self {
            key,
            previous,
            _lock: lock,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.previous {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
        let should_release = ENV_LOCK_DEPTH.with(|depth| {
            let current = depth.get();
            depth.set(current.saturating_sub(1));
            current <= 1
        });
        if should_release {
            self._lock = None;
        }
    }
}

fn build_multipart_file_body(
    boundary: &str,
    field_name: &str,
    file_name: &str,
    content_type: &str,
    bytes: &[u8],
) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
            field_name, file_name
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", content_type).as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    body
}

fn build_multipart_text_and_file_body(
    boundary: &str,
    text_field_name: &str,
    text_value: &str,
    file_field_name: &str,
    file_name: &str,
    content_type: &str,
    bytes: &[u8],
) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
            text_field_name
        )
        .as_bytes(),
    );
    body.extend_from_slice(text_value.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
            file_field_name, file_name
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", content_type).as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    body
}

fn build_tiny_png() -> Vec<u8> {
    let img = RgbaImage::from_pixel(2, 2, Rgba([10, 20, 30, 255]));
    let mut out = Vec::new();
    DynamicImage::ImageRgba8(img)
        .write_to(&mut IoCursor::new(&mut out), ImageFormat::Png)
        .expect("png encode");
    out
}

fn build_backup_zip(payload: &Value, extra_files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip_bytes = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(IoCursor::new(&mut zip_bytes));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("backup.json", options)
            .expect("start backup");
        let json_bytes = serde_json::to_vec(payload).expect("backup json");
        zip.write_all(&json_bytes).expect("write backup");

        for (name, content) in extra_files {
            if name.ends_with('/') {
                zip.add_directory(*name, options).expect("add dir");
            } else {
                zip.start_file(*name, options).expect("start file");
                zip.write_all(content).expect("write file");
            }
        }

        zip.finish().expect("finish zip");
    }
    zip_bytes
}

fn websocket_request(uri: &str, cookie: &str) -> axum::http::Request<()> {
    let host = uri
        .strip_prefix("ws://")
        .or_else(|| uri.strip_prefix("wss://"))
        .and_then(|rest| rest.split('/').next())
        .unwrap_or("localhost");

    axum::http::Request::builder()
        .uri(uri)
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header(header::COOKIE, cookie)
        .body(())
        .unwrap()
}

#[tokio::test]
async fn test_full_handler_flow_materials_quizzes_feedback_submissions_attendees_audit() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, true).await;
    let (attendee_cookie, attendee_csrf, attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "Alice",
        "pass-1234",
    )
    .await;

    // attendees: admin view + attendee view
    let attendees_admin_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/attendees", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attendees_admin_res.status(), StatusCode::OK);

    let attendees_attendee_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/attendees", codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attendees_attendee_res.status(), StatusCode::OK);

    // materials: add + get
    let add_material_payload = json!({
        "title": "Docs",
        "material_type": "link",
        "link_url": "https://example.com/docs"
    });
    let add_material_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/materials", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&add_material_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(add_material_res.status(), StatusCode::OK);
    let add_material_body = axum::body::to_bytes(add_material_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let material: Value = serde_json::from_slice(&add_material_body).unwrap();
    let material_id = material["id"].as_str().unwrap().to_string();

    let get_materials_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/materials", codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_materials_res.status(), StatusCode::OK);

    let delete_material_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/materials/{}",
                    codelab.id, material_id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_material_res.status(), StatusCode::NO_CONTENT);

    // quizzes: update + get + submit + list submissions
    let update_quizzes_payload = json!([
        {
            "question": "2+2?",
            "quiz_type": "multiple_choice",
            "options": ["3", "4"],
            "correct_answer": 1,
            "correct_answers": [1]
        }
    ]);
    let update_quizzes_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/codelabs/{}/quizzes", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&update_quizzes_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_quizzes_res.status(), StatusCode::OK);

    let get_quizzes_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/quizzes", codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_quizzes_res.status(), StatusCode::OK);
    let get_quizzes_body = axum::body::to_bytes(get_quizzes_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let quizzes: Value = serde_json::from_slice(&get_quizzes_body).unwrap();
    let quiz_id = quizzes[0]["id"].as_str().unwrap().to_string();

    let submit_quiz_payload = json!({
        "submissions": [
            {
                "quiz_id": quiz_id,
                "answer": "4",
                "is_correct": true
            }
        ]
    });
    let submit_quiz_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/quizzes/submit", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&submit_quiz_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_quiz_res.status(), StatusCode::OK);

    let get_quiz_submissions_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/quizzes/submissions", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_quiz_submissions_res.status(), StatusCode::OK);

    // attendees help flow
    let request_help_payload = json!({ "step_number": 1 });
    let request_help_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/help", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&request_help_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_help_res.status(), StatusCode::OK);

    let get_help_requests_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/help", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_help_requests_res.status(), StatusCode::OK);
    let get_help_requests_body =
        axum::body::to_bytes(get_help_requests_res.into_body(), usize::MAX)
            .await
            .unwrap();
    let help_requests: Value = serde_json::from_slice(&get_help_requests_body).unwrap();
    let help_id = help_requests[0]["id"].as_str().unwrap().to_string();

    let resolve_help_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/help/{}/resolve",
                    codelab.id, help_id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_help_res.status(), StatusCode::OK);

    // feedback: submit + list
    let feedback_payload = json!({
        "difficulty": "3",
        "satisfaction": "5",
        "comment": "nice"
    });
    let submit_feedback_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/feedback", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(serde_json::to_vec(&feedback_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_feedback_res.status(), StatusCode::OK);

    let get_feedback_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/feedback", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_feedback_res.status(), StatusCode::OK);

    // complete should fail before submission
    let complete_before_submission_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/complete", codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        complete_before_submission_res.status(),
        StatusCode::BAD_REQUEST
    );

    // submissions: submit link + list + delete
    let submit_link_payload = json!({
        "url": "https://example.com/repo",
        "title": "Repo"
    });
    let submit_link_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    codelab.id, attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&submit_link_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_link_res.status(), StatusCode::OK);
    let submit_link_body = axum::body::to_bytes(submit_link_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let submission: Value = serde_json::from_slice(&submit_link_body).unwrap();
    let submission_id = submission["id"].as_str().unwrap().to_string();

    let get_submissions_admin_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/submissions", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_submissions_admin_res.status(), StatusCode::OK);

    let get_submissions_attendee_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/submissions", codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_submissions_attendee_res.status(), StatusCode::OK);

    // complete should pass after submission is present.
    let complete_after_submission_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/complete", codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(complete_after_submission_res.status(), StatusCode::OK);

    let delete_submission_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/{}",
                    codelab.id, attendee_id, submission_id
                ))
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_submission_res.status(), StatusCode::NO_CONTENT);

    // certificate
    let cert_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/certificates/{}", attendee_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cert_res.status(), StatusCode::OK);

    // audit logs - exercise all query branches
    let audit_all_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/audit-logs")
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_all_res.status(), StatusCode::OK);

    let audit_action_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/audit-logs?action=quiz_update")
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_action_res.status(), StatusCode::OK);

    let audit_codelab_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/audit-logs?codelab_id={}", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_codelab_res.status(), StatusCode::OK);

    let audit_both_res = test_app
        .app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/admin/audit-logs?codelab_id={}&action=submission_link",
                    codelab.id
                ))
                .header(header::COOKIE, admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_both_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_upload_endpoints_handle_missing_file_and_unauthorized() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let boundary = "----boundary";
    let empty_multipart = format!("--{}--\r\n", boundary);

    let upload_image_unauthorized = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/image")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(empty_multipart.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_image_unauthorized.status(), StatusCode::UNAUTHORIZED);

    let upload_image_no_file = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/image")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(empty_multipart.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_image_no_file.status(), StatusCode::BAD_REQUEST);

    let upload_material_no_file = test_app
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/material")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .header(header::COOKIE, admin_cookie)
                .header("x-csrf-token", admin_csrf)
                .body(Body::from(empty_multipart))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_material_no_file.status(), StatusCode::BAD_REQUEST);
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
        require_submission: None,
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

#[tokio::test]
async fn test_admin_session_settings_updates_and_logout() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let _frontend_tag = EnvVarGuard::set("FRONTEND_IMAGE_TAG", "v0.1.0");
    let _backend_tag = EnvVarGuard::set("BACKEND_IMAGE_TAG", "v0.1.0");

    let session_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/session")
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(session_res.status(), StatusCode::OK);
    let session_body = axum::body::to_bytes(session_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let session_json: Value = serde_json::from_slice(&session_body).unwrap();
    assert_eq!(session_json["role"], "admin");

    let settings_plaintext_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/settings")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "gemini_api_key": "plain-text-key" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settings_plaintext_res.status(), StatusCode::BAD_REQUEST);

    let encrypted_key =
        encrypt_with_password("gemini-secret", &test_app.state.admin_pw).expect("encrypt key");
    let settings_encrypted_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/settings")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "gemini_api_key": encrypted_key })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settings_encrypted_res.status(), StatusCode::OK);
    assert!(test_app.state.admin_api_keys.contains_key("global_admin"));

    let settings_remove_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/settings")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "gemini_api_key": "" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settings_remove_res.status(), StatusCode::OK);
    assert!(!test_app.state.admin_api_keys.contains_key("global_admin"));

    let settings_too_long_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/settings")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "gemini_api_key": "x".repeat(4097) })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settings_too_long_res.status(), StatusCode::BAD_REQUEST);

    let updates_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/updates")
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(updates_res.status(), StatusCode::OK);
    let updates_body = axum::body::to_bytes(updates_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let updates_json: Value = serde_json::from_slice(&updates_body).unwrap();
    assert!(updates_json.get("frontend").is_some());
    assert!(updates_json.get("backend").is_some());

    let settings_unauth_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/settings")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "gemini_api_key": "" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settings_unauth_res.status(), StatusCode::UNAUTHORIZED);

    let logout_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logout")
                .header(header::COOKIE, admin_cookie)
                .header("x-csrf-token", admin_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_res.status(), StatusCode::NO_CONTENT);

    let session_unauth_res = test_app
        .app
        .oneshot(
            Request::builder()
                .uri("/api/session")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(session_unauth_res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_codelab_management_inline_comments_and_ai_endpoints() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, true).await;

    let steps_payload = json!({
        "steps": [
            {
                "title": "Step One",
                "content_markdown": "First content"
            },
            {
                "title": "Step Two",
                "content_markdown": "Second content"
            }
        ]
    });
    let update_steps_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/codelabs/{}/steps", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(serde_json::to_vec(&steps_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_steps_res.status(), StatusCode::OK);

    let get_codelab_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}", codelab.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_codelab_res.status(), StatusCode::OK);
    let get_codelab_body = axum::body::to_bytes(get_codelab_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let codelab_data: Value = serde_json::from_slice(&get_codelab_body).unwrap();
    let step_id = codelab_data[1][0]["id"].as_str().unwrap().to_string();

    let (attendee_cookie, attendee_csrf, _attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "Bob",
        "join-code",
    )
    .await;

    let create_comment_payload = json!({
        "anchor_key": "step-1:0-4",
        "target_type": "step",
        "target_step_id": step_id,
        "start_offset": 0,
        "end_offset": 4,
        "selected_text": "Step",
        "content_hash": "hash-v1",
        "message": "first"
    });
    let create_comment_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/inline-comments", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&create_comment_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_comment_res.status(), StatusCode::OK);
    let create_comment_body = axum::body::to_bytes(create_comment_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_thread: Value = serde_json::from_slice(&create_comment_body).unwrap();
    let thread_id = created_thread["id"].as_str().unwrap().to_string();
    let first_comment_id = created_thread["messages"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let reply_payload = json!({
        "message": "admin reply",
        "content_hash": "hash-v1"
    });
    let reply_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/inline-comments/{}/comments",
                    codelab.id, thread_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(serde_json::to_vec(&reply_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reply_res.status(), StatusCode::OK);
    let reply_body = axum::body::to_bytes(reply_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let replied_thread: Value = serde_json::from_slice(&reply_body).unwrap();
    let second_comment_id = replied_thread["messages"][1]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let get_comments_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/codelabs/{}/inline-comments?target_type=step",
                    codelab.id
                ))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_comments_res.status(), StatusCode::OK);

    let delete_first_comment_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/inline-comments/{}/comments/{}",
                    codelab.id, thread_id, first_comment_id
                ))
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_first_comment_res.status(), StatusCode::OK);

    let delete_second_comment_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/inline-comments/{}/comments/{}",
                    codelab.id, thread_id, second_comment_id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_second_comment_res.status(), StatusCode::OK);

    let chat_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        &test_app
            .state
            .q("INSERT INTO chat_messages (id, codelab_id, sender_name, message, msg_type) VALUES (?, ?, ?, ?, 'chat')"),
    )
    .bind(chat_id)
    .bind(&codelab.id)
    .bind("Facilitator")
    .bind("hello room")
    .execute(&test_app.state.pool)
    .await
    .unwrap();

    let chat_history_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/chat", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(chat_history_res.status(), StatusCode::OK);

    let copy_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/copy", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(copy_res.status(), StatusCode::OK);
    let copy_body = axum::body::to_bytes(copy_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let copied_codelab: Codelab = serde_json::from_slice(&copy_body).unwrap();

    let update_info_payload = json!({
        "title": "Updated Title",
        "description": "Updated Description",
        "author": "Updated Author",
        "is_public": true,
        "quiz_enabled": true,
        "require_quiz": false,
        "require_feedback": true,
        "require_submission": true,
        "guide_markdown": "# Updated"
    });
    let update_info_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/codelabs/{}", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&update_info_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_info_res.status(), StatusCode::OK);

    let export_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/export", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_res.status(), StatusCode::OK);
    let export_zip = axum::body::to_bytes(export_res.into_body(), usize::MAX)
        .await
        .unwrap();
    {
        let mut archive =
            zip::ZipArchive::new(IoCursor::new(export_zip.clone().to_vec())).expect("zip archive");
        let mut has_guide = false;
        for i in 0..archive.len() {
            let file = archive.by_index(i).expect("zip entry");
            if file.name() == "preparation_guide.md" {
                has_guide = true;
                break;
            }
        }
        assert!(has_guide);
    }

    let boundary = "----codelab-import-boundary";
    let import_body = build_multipart_text_and_file_body(
        boundary,
        "note",
        "import-request",
        "file",
        "codelab.zip",
        "application/zip",
        &export_zip,
    );
    let import_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs/import")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(import_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(import_res.status(), StatusCode::OK);
    let import_body = axum::body::to_bytes(import_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let imported_codelab: Codelab = serde_json::from_slice(&import_body).unwrap();

    let update_blank_guide_payload = json!({
        "title": "Updated Title",
        "description": "Updated Description",
        "author": "Updated Author",
        "is_public": true,
        "quiz_enabled": true,
        "require_quiz": false,
        "require_feedback": true,
        "require_submission": true,
        "guide_markdown": "   "
    });
    let update_blank_guide_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/codelabs/{}", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&update_blank_guide_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_blank_guide_res.status(), StatusCode::OK);
    let export_blank_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/export", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_blank_res.status(), StatusCode::OK);
    let export_blank_zip = axum::body::to_bytes(export_blank_res.into_body(), usize::MAX)
        .await
        .unwrap();
    {
        let mut archive =
            zip::ZipArchive::new(IoCursor::new(export_blank_zip.to_vec())).expect("zip archive");
        let mut has_guide = false;
        for i in 0..archive.len() {
            let file = archive.by_index(i).expect("zip entry");
            if file.name() == "preparation_guide.md" {
                has_guide = true;
                break;
            }
        }
        assert!(!has_guide);
    }

    let reference_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/codelabs/reference")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reference_res.status(), StatusCode::OK);

    let delete_imported_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/codelabs/{}", imported_codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_imported_res.status(), StatusCode::OK);

    let delete_copied_res = test_app
        .app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/codelabs/{}", copied_codelab.id))
                .header(header::COOKIE, admin_cookie)
                .header("x-csrf-token", admin_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_copied_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_codeserver_endpoints_branch_and_folder_flow() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;

    let workspace_dir = tempfile::tempdir().unwrap();
    let _workspace_guard = EnvVarGuard::set(
        "WORKSPACE_BASE",
        workspace_dir.path().to_string_lossy().to_string(),
    );

    let create_workspace_payload = json!({
        "codelab_id": codelab.id,
        "structure_type": "branch",
        "workspace_files": [
            { "path": "README.md", "content": "# Hello" },
            { "path": "old.txt", "content": "old" }
        ]
    });
    let create_workspace_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codeserver")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&create_workspace_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_workspace_res.status(), StatusCode::OK);

    let get_workspace_info_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codeserver/{}", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_workspace_info_res.status(), StatusCode::OK);

    let create_branch_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codeserver/{}/branch", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "step_number": 1, "branch_type": "start" }))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_branch_res.status(), StatusCode::OK);

    let list_branches_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codeserver/{}/branches", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_branches_res.status(), StatusCode::OK);

    let list_files_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/codeserver/{}/branches/step-1-start/files",
                    codelab.id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_files_res.status(), StatusCode::OK);

    let read_file_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/codeserver/{}/branches/step-1-start/file?file=README.md",
                    codelab.id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read_file_res.status(), StatusCode::OK);

    let update_branch_payload = json!({
        "files": [
            { "path": "updated.txt", "content": "updated" }
        ],
        "delete_files": ["old.txt"],
        "commit_message": "update branch files"
    });
    let update_branch_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codeserver/{}/branches/step-1-start/files",
                    codelab.id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&update_branch_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_branch_res.status(), StatusCode::OK);

    let download_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codeserver/{}/download", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(download_res.status(), StatusCode::OK);

    let create_folder_payload = json!({
        "step_number": 2,
        "folder_type": "start",
        "files": [
            { "path": "main.rs", "content": "fn main() {}" },
            { "path": "delete.me", "content": "bye" }
        ]
    });
    let create_folder_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codeserver/{}/folder", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&create_folder_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_folder_res.status(), StatusCode::OK);

    let list_folders_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codeserver/{}/folders", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_folders_res.status(), StatusCode::OK);

    let list_folder_files_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/codeserver/{}/folders/step-2-start/files",
                    codelab.id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_folder_files_res.status(), StatusCode::OK);

    let read_folder_file_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/codeserver/{}/folders/step-2-start/file?file=main.rs",
                    codelab.id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read_folder_file_res.status(), StatusCode::OK);

    let update_folder_payload = json!({
        "files": [
            { "path": "lib.rs", "content": "pub fn x() {}" }
        ],
        "delete_files": ["delete.me"],
        "commit_message": null
    });
    let update_folder_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codeserver/{}/folders/step-2-start/files",
                    codelab.id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&update_folder_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_folder_res.status(), StatusCode::OK);

    let delete_workspace_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/codeserver/{}", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_workspace_res.status(), StatusCode::OK);

    let get_workspace_missing_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codeserver/{}", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_workspace_missing_res.status(), StatusCode::BAD_REQUEST);

    let create_workspace_unauth_res = test_app
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codeserver")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(
                        &json!({ "codelab_id": codelab.id, "structure_type": "branch" }),
                    )
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        create_workspace_unauth_res.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_backup_export_inspect_restore_and_upload_success_paths() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, true).await;
    let (attendee_cookie, attendee_csrf, attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "Carol",
        "backup-code",
    )
    .await;

    let workspace_dir = tempfile::tempdir().unwrap();
    let _workspace_guard = EnvVarGuard::set(
        "WORKSPACE_BASE",
        workspace_dir.path().to_string_lossy().to_string(),
    );

    let _gemini_guard = EnvVarGuard::set("GEMINI_API_KEY", "dummy-key");
    let stream_bad_payload_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/stream")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "prompt": "hello", "model": "BadModel!" }))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stream_bad_payload_res.status(), StatusCode::BAD_REQUEST);

    let stream_network_payload_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/stream")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(
                        &json!({ "prompt": "hello", "model": "gemini-3-flash-preview" }),
                    )
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        stream_network_payload_res.status().is_server_error()
            || stream_network_payload_res.status() == StatusCode::OK
    );

    let save_conversation_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/conversations")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "codelab_id": codelab.id,
                        "step_number": 1,
                        "question": "Q?",
                        "answer": "A!",
                        "model": "gemini-3-flash-preview"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_conversation_res.status(), StatusCode::OK);

    let get_conversations_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/ai/conversations", codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_conversations_res.status(), StatusCode::OK);

    let create_thread_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/threads")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "title": "Thread 1", "codelab_id": codelab.id }))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_thread_res.status(), StatusCode::OK);
    let create_thread_body = axum::body::to_bytes(create_thread_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let thread_json: Value = serde_json::from_slice(&create_thread_body).unwrap();
    let thread_id = thread_json["id"].as_str().unwrap().to_string();

    let add_message_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/ai/threads/{}", thread_id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "role": "assistant",
                        "content": "hello",
                        "grounding_metadata": {"source":"test"}
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(add_message_res.status(), StatusCode::OK);

    let get_messages_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/ai/threads/{}", thread_id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_messages_res.status(), StatusCode::OK);

    let get_threads_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/ai/threads")
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_threads_res.status(), StatusCode::OK);

    let delete_thread_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/ai/threads/{}", thread_id))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_thread_res.status(), StatusCode::NO_CONTENT);

    let create_workspace_payload = json!({
        "codelab_id": codelab.id,
        "structure_type": "folder",
        "workspace_files": [
            { "path": "data.txt", "content": "backup data" }
        ]
    });
    let create_workspace_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codeserver")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&create_workspace_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_workspace_res.status(), StatusCode::OK);

    let png = build_tiny_png();
    let image_boundary = "----upload-image-boundary";
    let image_body =
        build_multipart_file_body(image_boundary, "file", "avatar.png", "image/png", &png);
    let upload_image_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/image")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", image_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(image_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_image_res.status(), StatusCode::OK);

    let material_boundary = "----upload-material-boundary";
    let material_body = build_multipart_file_body(
        material_boundary,
        "file",
        "slides.pdf",
        "application/pdf",
        b"pdf-bytes",
    );
    let upload_material_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/material")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", material_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(material_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_material_res.status(), StatusCode::OK);

    let submit_file_boundary = "----submit-file-boundary";
    let submit_file_body = build_multipart_file_body(
        submit_file_boundary,
        "file",
        "solution.png",
        "image/png",
        &png,
    );
    let submit_file_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", submit_file_boundary),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(submit_file_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_file_res.status(), StatusCode::OK);
    let submit_file_body = axum::body::to_bytes(submit_file_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let submission_json: Value = serde_json::from_slice(&submit_file_body).unwrap();
    let submission_id = submission_json["id"].as_str().unwrap().to_string();

    let delete_submission_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/{}",
                    codelab.id, attendee_id, submission_id
                ))
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_submission_res.status(), StatusCode::NO_CONTENT);

    let backup_export_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/backup/export")
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_export_res.status(), StatusCode::OK);
    let backup_zip = axum::body::to_bytes(backup_export_res.into_body(), usize::MAX)
        .await
        .unwrap();

    let inspect_boundary = "----backup-inspect-boundary";
    let inspect_body = build_multipart_file_body(
        inspect_boundary,
        "file",
        "backup.zip",
        "application/zip",
        &backup_zip,
    );
    let inspect_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/inspect")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", inspect_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(inspect_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inspect_res.status(), StatusCode::OK);

    let restore_boundary = "----backup-restore-boundary";
    let restore_body = build_multipart_file_body(
        restore_boundary,
        "file",
        "backup.zip",
        "application/zip",
        &backup_zip,
    );
    let restore_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/restore")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", restore_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(restore_body))
                .unwrap(),
        )
        .await
        .unwrap();
    if restore_res.status() != StatusCode::OK {
        let status = restore_res.status();
        let body = axum::body::to_bytes(restore_res.into_body(), usize::MAX)
            .await
            .unwrap();
        panic!(
            "custom restore failed: {} - {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    let codelabs_after_restore_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/codelabs")
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(codelabs_after_restore_res.status(), StatusCode::OK);

    let second_codelab =
        create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let ws_unauthorized = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/ws/{}", codelab.id))
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        ws_unauthorized.status() == StatusCode::UNAUTHORIZED
            || ws_unauthorized.status() == StatusCode::UPGRADE_REQUIRED
    );

    let ws_forbidden = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/ws/{}", second_codelab.id))
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        ws_forbidden.status() == StatusCode::FORBIDDEN
            || ws_forbidden.status() == StatusCode::UPGRADE_REQUIRED
    );

    let ws_admin_upgrade = test_app
        .app
        .oneshot(
            Request::builder()
                .uri(format!("/api/ws/{}", codelab.id))
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                .header(header::COOKIE, admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        ws_admin_upgrade.status() == StatusCode::SWITCHING_PROTOCOLS
            || ws_admin_upgrade.status() == StatusCode::UPGRADE_REQUIRED
    );
}

#[tokio::test]
async fn test_websocket_live_message_paths() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let other_codelab =
        create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let (_attendee_cookie, _attendee_csrf, attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "WebSocket User",
        "ws-code",
    )
    .await;

    let attendee_claims = SessionClaims {
        sub: attendee_id.clone(),
        role: "attendee".to_string(),
        codelab_id: Some(codelab.id.clone()),
        iss: test_app.state.auth.issuer.clone(),
        aud: test_app.state.auth.audience.clone(),
        iat: 1,
        exp: usize::MAX / 2,
    };
    let admin_claims = SessionClaims {
        sub: test_app.state.admin_id.clone(),
        role: "admin".to_string(),
        codelab_id: None,
        iss: test_app.state.auth.issuer.clone(),
        aud: test_app.state.auth.audience.clone(),
        iat: 1,
        exp: usize::MAX / 2,
    };

    let attendee_token = test_app.state.auth.issue_token(&attendee_claims).unwrap();
    let admin_token = test_app.state.auth.issue_token(&admin_claims).unwrap();
    let attendee_cookie = format!(
        "{}={}",
        test_app.state.auth.attendee_cookie_name, attendee_token
    );
    let admin_cookie = format!("{}={}", test_app.state.auth.cookie_name, admin_token);

    let Some(listener) = bind_local_listener_or_skip().await else {
        return;
    };
    let addr = listener.local_addr().unwrap();
    let ws_app = create_router(test_app.state.clone());
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            ws_app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let ws_url = format!("ws://{}/api/ws/{}", addr, codelab.id);
    let mismatch_ws_url = format!("ws://{}/api/ws/{}", addr, other_codelab.id);
    let mismatch_conn = connect_async(websocket_request(&mismatch_ws_url, &attendee_cookie)).await;
    assert!(mismatch_conn.is_err());
    if let Err(tokio_tungstenite::tungstenite::Error::Http(res)) = mismatch_conn {
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }
    let (admin_ws, _) = connect_async(websocket_request(&ws_url, &admin_cookie))
        .await
        .unwrap();
    let (attendee_ws, _) = connect_async(websocket_request(&ws_url, &attendee_cookie))
        .await
        .unwrap();

    let (mut admin_write, mut admin_read) = admin_ws.split();
    let (mut attendee_write, mut attendee_read) = attendee_ws.split();

    attendee_write
        .send(WsMessage::Text(
            json!({ "type": "chat", "message": "hello ws" })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "dm",
                "target_id": "facilitator",
                "message": "direct hi"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({ "type": "step_progress", "step_number": 2 })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "webrtc_signal",
                "target_id": "facilitator",
                "signal": {"sdp":"dummy"},
                "stream_type": "camera"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    admin_write
        .send(WsMessage::Text(
            json!({ "type": "screen_share_status", "status": "facilitator_started" })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({ "type": "attendee_screen_status", "status": "started" })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();

    // Drain a few messages so both receive loops are exercised.
    for _ in 0..4 {
        let _ = tokio::time::timeout(Duration::from_millis(300), admin_read.next()).await;
        let _ = tokio::time::timeout(Duration::from_millis(300), attendee_read.next()).await;
    }

    sleep(Duration::from_millis(200)).await;

    let chat_count: i64 = sqlx::query_scalar(
        &test_app
            .state
            .q("SELECT COUNT(*) FROM chat_messages WHERE codelab_id = ?"),
    )
    .bind(&codelab.id)
    .fetch_one(&test_app.state.pool)
    .await
    .unwrap();
    assert!(chat_count >= 2); // chat + dm

    let current_step: i32 = sqlx::query_scalar(
        &test_app
            .state
            .q("SELECT current_step FROM attendees WHERE id = ?"),
    )
    .bind(&attendee_id)
    .fetch_one(&test_app.state.pool)
    .await
    .unwrap();
    assert_eq!(current_step, 2);

    assert_eq!(
        test_app
            .state
            .active_screen_shares
            .get(&codelab.id)
            .map(|v| *v),
        Some(true)
    );
    assert_eq!(
        test_app
            .state
            .attendee_sharing
            .get(&(codelab.id.clone(), attendee_id.clone()))
            .map(|v| *v),
        Some(true)
    );

    let (late_cookie, _late_csrf, _late_attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "LateUser",
        "late-code",
    )
    .await;
    let (mut late_ws, _) = connect_async(websocket_request(&ws_url, &late_cookie))
        .await
        .expect("late attendee websocket connect");
    let late_msg = tokio::time::timeout(Duration::from_secs(2), late_ws.next())
        .await
        .expect("late attendee message timeout")
        .expect("late attendee stream ended")
        .expect("late attendee websocket error");
    match late_msg {
        WsMessage::Text(text) => {
            let payload: Value = serde_json::from_str(text.as_ref()).unwrap();
            assert_eq!(
                payload,
                json!({ "type": "screen_share_status", "status": "facilitator_started" })
            );
        }
        other => panic!("unexpected late attendee message: {other:?}"),
    }
    late_ws.close(None).await.unwrap();

    admin_write.send(WsMessage::Close(None)).await.unwrap();
    attendee_write.send(WsMessage::Close(None)).await.unwrap();
    sleep(Duration::from_millis(300)).await;

    server.abort();
}

#[tokio::test]
async fn test_admin_and_attendee_session_edge_paths() {
    let test_app = setup_test_app().await;

    let bad_login_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "admin_id": " ", "admin_pw": "" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad_login_res.status(), StatusCode::BAD_REQUEST);

    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let (attendee_cookie, attendee_csrf, _) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "SessionUser",
        "session-code",
    )
    .await;

    let attendee_session_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/session")
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attendee_session_res.status(), StatusCode::OK);
    let attendee_session_body = axum::body::to_bytes(attendee_session_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let attendee_session_json: Value = serde_json::from_slice(&attendee_session_body).unwrap();
    assert_eq!(attendee_session_json["role"], "attendee");

    let attendee_logout_res = test_app
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logout")
                .header(header::COOKIE, attendee_cookie)
                .header("x-csrf-token", attendee_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attendee_logout_res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_attendee_registration_and_help_edge_paths() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let public_codelab =
        create_codelab_with_options(&test_app.app, &admin_cookie, &admin_csrf, true, false).await;
    let another_codelab =
        create_codelab_with_options(&test_app.app, &admin_cookie, &admin_csrf, true, false).await;
    let private_codelab =
        create_codelab_with_options(&test_app.app, &admin_cookie, &admin_csrf, false, false).await;

    let register_private_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/register", private_codelab.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Private User",
                        "code": "private-code",
                        "email": Value::Null
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(register_private_res.status(), StatusCode::FORBIDDEN);

    let first_register_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/register", public_codelab.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Dup User",
                        "code": "dup-code",
                        "email": Value::Null
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_register_res.status(), StatusCode::OK);
    let (dup_cookie, dup_cookies) = extract_cookies(first_register_res.headers());
    let dup_csrf = dup_cookies
        .get(&test_app.state.auth.csrf_cookie_name)
        .cloned()
        .expect("dup csrf missing");
    let first_register_body = axum::body::to_bytes(first_register_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let first_attendee: Value = serde_json::from_slice(&first_register_body).unwrap();
    let dup_attendee_id = first_attendee["id"].as_str().unwrap().to_string();

    let duplicate_conflict_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/register", public_codelab.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Dup User",
                        "code": "wrong-code",
                        "email": "dup@example.com"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate_conflict_res.status(), StatusCode::CONFLICT);

    let duplicate_rejoin_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/register", public_codelab.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Dup User",
                        "code": "dup-code",
                        "email": "dup@example.com"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate_rejoin_res.status(), StatusCode::OK);
    let duplicate_rejoin_body = axum::body::to_bytes(duplicate_rejoin_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let rejoined_attendee: Value = serde_json::from_slice(&duplicate_rejoin_body).unwrap();
    assert_eq!(
        rejoined_attendee["id"].as_str().unwrap(),
        dup_attendee_id.as_str()
    );
    let stored_email: Option<String> =
        sqlx::query_scalar(&test_app.state.q("SELECT email FROM attendees WHERE id = ?"))
            .bind(&dup_attendee_id)
            .fetch_one(&test_app.state.pool)
            .await
            .unwrap();
    assert_eq!(stored_email.as_deref(), Some("dup@example.com"));

    let duplicate_rejoin_without_email_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/register", public_codelab.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Dup User",
                        "code": "dup-code",
                        "email": Value::Null
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate_rejoin_without_email_res.status(), StatusCode::OK);

    test_app
        .state
        .attendee_sharing
        .insert((public_codelab.id.clone(), dup_attendee_id.clone()), true);
    let attendees_admin_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/attendees", public_codelab.id))
                .header(header::COOKIE, admin_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attendees_admin_res.status(), StatusCode::OK);
    let attendees_admin_body = axum::body::to_bytes(attendees_admin_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let attendees_admin: Vec<Value> = serde_json::from_slice(&attendees_admin_body).unwrap();
    let dup_row = attendees_admin
        .iter()
        .find(|row| row["id"] == dup_attendee_id)
        .expect("dup attendee in list");
    assert_eq!(dup_row["is_sharing_screen"], true);

    let attendees_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/attendees", another_codelab.id))
                .header(header::COOKIE, dup_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attendees_forbidden_res.status(), StatusCode::FORBIDDEN);

    let help_admin_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/help", public_codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "step_number": 1 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(help_admin_res.status(), StatusCode::BAD_REQUEST);

    let help_unauth_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/help", public_codelab.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "step_number": 1 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(help_unauth_res.status(), StatusCode::UNAUTHORIZED);

    let help_mismatch_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/help", another_codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, dup_cookie.clone())
                .header("x-csrf-token", dup_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "step_number": 1 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(help_mismatch_res.status(), StatusCode::FORBIDDEN);

    let help_invalid_step_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/help", public_codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, dup_cookie.clone())
                .header("x-csrf-token", dup_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "step_number": 0 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(help_invalid_step_res.status(), StatusCode::BAD_REQUEST);

    let (tx, mut rx) = tokio::sync::broadcast::channel(8);
    test_app
        .state
        .channels
        .insert(public_codelab.id.clone(), tx);

    let help_ok_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/help", public_codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, dup_cookie.clone())
                .header("x-csrf-token", dup_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "step_number": 2 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(help_ok_res.status(), StatusCode::OK);
    let help_msg = tokio::time::timeout(Duration::from_millis(500), rx.recv())
        .await
        .expect("help broadcast timeout")
        .expect("help broadcast message");
    let help_json: Value = serde_json::from_str(&help_msg).unwrap();
    assert_eq!(help_json["type"], "help_request");

    let help_list_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/help", another_codelab.id))
                .header(header::COOKIE, dup_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(help_list_forbidden_res.status(), StatusCode::FORBIDDEN);

    let help_list_ok_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/help", public_codelab.id))
                .header(header::COOKIE, dup_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(help_list_ok_res.status(), StatusCode::OK);

    let complete_without_required_submission_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/complete", public_codelab.id))
                .header(header::COOKIE, dup_cookie.clone())
                .header("x-csrf-token", dup_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        complete_without_required_submission_res.status(),
        StatusCode::OK
    );

    let require_submission_codelab =
        create_codelab_with_options(&test_app.app, &admin_cookie, &admin_csrf, true, true).await;
    let (submit_cookie, submit_csrf, submit_attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &require_submission_codelab.id,
        "SubmitUser",
        "submit-code",
    )
    .await;
    sqlx::query(&test_app.state.q(
        "INSERT INTO submissions (id, codelab_id, attendee_id, file_path, file_name, file_size, submission_type, link_url) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    ))
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&require_submission_codelab.id)
    .bind(&submit_attendee_id)
    .bind("/uploads/submissions/ok.webp")
    .bind("ok.webp")
    .bind(1_i64)
    .bind("file")
    .bind(Option::<String>::None)
    .execute(&test_app.state.pool)
    .await
    .unwrap();
    let complete_with_submission_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/complete",
                    require_submission_codelab.id
                ))
                .header(header::COOKIE, submit_cookie)
                .header("x-csrf-token", submit_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(complete_with_submission_res.status(), StatusCode::OK);

    let complete_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/complete", another_codelab.id))
                .header(header::COOKIE, dup_cookie.clone())
                .header("x-csrf-token", dup_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(complete_forbidden_res.status(), StatusCode::FORBIDDEN);

    let unfinished_register_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/register", public_codelab.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "Unfinished User",
                        "code": "unfinished-code",
                        "email": Value::Null
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unfinished_register_res.status(), StatusCode::OK);
    let unfinished_register_body =
        axum::body::to_bytes(unfinished_register_res.into_body(), usize::MAX)
            .await
            .unwrap();
    let unfinished_attendee: Value = serde_json::from_slice(&unfinished_register_body).unwrap();
    let cert_not_completed_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/certificates/{}",
                    unfinished_attendee["id"].as_str().unwrap()
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cert_not_completed_res.status(), StatusCode::FORBIDDEN);

    let cert_missing_res = test_app
        .app
        .oneshot(
            Request::builder()
                .uri("/api/certificates/missing-attendee")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cert_missing_res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_ai_edge_paths_and_thread_guards() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let another_codelab =
        create_codelab_with_options(&test_app.app, &admin_cookie, &admin_csrf, true, false).await;
    let (attendee_cookie, attendee_csrf, _) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "AiUser",
        "ai-code",
    )
    .await;

    {
        let _empty_key_guard = EnvVarGuard::set("GEMINI_API_KEY", "");

        let stream_unauth_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "prompt": "hello" })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stream_unauth_res.status(), StatusCode::UNAUTHORIZED);

        let stream_forbidden_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, attendee_cookie.clone())
                    .header("x-csrf-token", attendee_csrf.clone())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "prompt": "hello",
                            "codelab_id": another_codelab.id
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stream_forbidden_res.status(), StatusCode::FORBIDDEN);

        let stream_missing_prompt_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, attendee_cookie.clone())
                    .header("x-csrf-token", attendee_csrf.clone())
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "codelab_id": codelab.id })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stream_missing_prompt_res.status(), StatusCode::BAD_REQUEST);

        let stream_invalid_prompt_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, attendee_cookie.clone())
                    .header("x-csrf-token", attendee_csrf.clone())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "prompt": " ",
                            "codelab_id": codelab.id
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stream_invalid_prompt_res.status(), StatusCode::BAD_REQUEST);

        let stream_invalid_api_key_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, admin_cookie.clone())
                    .header("x-csrf-token", admin_csrf.clone())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "prompt": "hello",
                            "api_key": "not-encrypted",
                            "model": "gemini-3-flash-preview"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stream_invalid_api_key_res.status(), StatusCode::BAD_REQUEST);

        let stream_missing_api_key_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, admin_cookie.clone())
                    .header("x-csrf-token", admin_csrf.clone())
                    .body(Body::from(
                        serde_json::to_vec(
                            &json!({ "prompt": "hello", "model": "gemini-3-flash-preview" }),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stream_missing_api_key_res.status(), StatusCode::BAD_REQUEST);
    }

    {
        let _dummy_key_guard = EnvVarGuard::set("GEMINI_API_KEY", "dummy-key");
        let _api_base_guard = EnvVarGuard::set("GEMINI_API_BASE", "http://127.0.0.1:1");
        test_app
            .state
            .admin_api_keys
            .insert("global_admin".to_string(), "stored-key".to_string());

        let (ghost_cookie, ghost_csrf, ghost_attendee_id) = register_attendee(
            &test_app.app,
            &test_app.state,
            &codelab.id,
            "GhostAiUser",
            "ghost-code",
        )
        .await;
        sqlx::query(&test_app.state.q("DELETE FROM attendees WHERE id = ?"))
            .bind(&ghost_attendee_id)
            .execute(&test_app.state.pool)
            .await
            .unwrap();

        let stream_ghost_attendee_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, ghost_cookie)
                    .header("x-csrf-token", ghost_csrf)
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "prompt": "hello",
                            "model": "gemini-3-flash-preview",
                            "codelab_id": codelab.id
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            stream_ghost_attendee_res.status().is_server_error()
                || stream_ghost_attendee_res.status() == StatusCode::OK
        );

        test_app.state.admin_api_keys.remove("global_admin");
        let encrypted_payload_key =
            encrypt_with_password("payload-key", &test_app.state.admin_pw).expect("encrypt");
        let stream_encrypted_key_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, attendee_cookie.clone())
                    .header("x-csrf-token", attendee_csrf.clone())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "prompt": "hello",
                            "api_key": encrypted_payload_key,
                            "model": "gemini-3-flash-preview",
                            "codelab_id": codelab.id
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            stream_encrypted_key_res.status().is_server_error()
                || stream_encrypted_key_res.status() == StatusCode::OK
        );

        let stream_with_contents_res = test_app
            .app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ai/stream")
                    .header("Content-Type", "application/json")
                    .header(header::COOKIE, admin_cookie.clone())
                    .header("x-csrf-token", admin_csrf.clone())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "model": "gemini-3-flash-preview",
                            "contents": [{
                                "role": "user",
                                "parts": [{ "text": "hello" }]
                            }],
                            "system_instruction": "system",
                            "generation_config": { "temperature": 0.2 },
                            "tools": [{ "codeExecution": {} }]
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            stream_with_contents_res.status().is_server_error()
                || stream_with_contents_res.status() == StatusCode::OK
        );
    }

    let save_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/conversations")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "codelab_id": another_codelab.id,
                        "question": "Q",
                        "answer": "A"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_forbidden_res.status(), StatusCode::FORBIDDEN);

    let save_unauth_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/conversations")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "codelab_id": codelab.id,
                        "question": "Q",
                        "answer": "A"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_unauth_res.status(), StatusCode::UNAUTHORIZED);

    let save_attendee_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/conversations")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "codelab_id": codelab.id,
                        "question": "attendee question",
                        "answer": "attendee answer"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_attendee_res.status(), StatusCode::OK);

    let create_thread_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/threads")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(
                        &json!({ "title": "Guard Thread", "codelab_id": codelab.id }),
                    )
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_thread_res.status(), StatusCode::OK);

    let add_missing_thread_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/threads/missing-thread")
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "role": "assistant",
                        "content": "hello"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(add_missing_thread_res.status(), StatusCode::FORBIDDEN);

    let get_missing_thread_res = test_app
        .app
        .oneshot(
            Request::builder()
                .uri("/api/ai/threads/missing-thread")
                .header(header::COOKIE, admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_missing_thread_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_upload_material_and_submission_edge_paths() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let another_codelab =
        create_codelab_with_options(&test_app.app, &admin_cookie, &admin_csrf, true, false).await;
    let (attendee_cookie, attendee_csrf, attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "UploadUser",
        "upload-code",
    )
    .await;

    let png = build_tiny_png();
    let attendee_image_boundary = "----attendee-image-boundary";
    let attendee_image_body = build_multipart_file_body(
        attendee_image_boundary,
        "file",
        "avatar.png",
        "image/png",
        &png,
    );
    let attendee_upload_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/image")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", attendee_image_boundary),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(attendee_image_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attendee_upload_res.status(), StatusCode::OK);

    let huge_image = vec![0u8; 5 * 1024 * 1024 + 1];
    let huge_image_boundary = "----huge-image-boundary";
    let huge_image_body = build_multipart_file_body(
        huge_image_boundary,
        "file",
        "huge.png",
        "image/png",
        &huge_image,
    );
    let huge_image_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/image")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", huge_image_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(huge_image_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(huge_image_res.status(), StatusCode::BAD_REQUEST);

    let invalid_material_boundary = "----invalid-material-boundary";
    let invalid_material_body = build_multipart_file_body(
        invalid_material_boundary,
        "file",
        "???",
        "application/octet-stream",
        b"abc",
    );
    let invalid_material_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/material")
                .header(
                    "Content-Type",
                    format!(
                        "multipart/form-data; boundary={}",
                        invalid_material_boundary
                    ),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(invalid_material_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_material_res.status(), StatusCode::BAD_REQUEST);

    let no_ext_material_boundary = "----no-ext-material-boundary";
    let no_ext_material_body = build_multipart_file_body(
        no_ext_material_boundary,
        "file",
        "README",
        "text/plain",
        b"material",
    );
    let no_ext_material_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/material")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", no_ext_material_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(no_ext_material_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(no_ext_material_res.status(), StatusCode::OK);

    let huge_material = vec![0u8; 10 * 1024 * 1024 + 1];
    let huge_material_boundary = "----huge-material-boundary";
    let huge_material_body = build_multipart_file_body(
        huge_material_boundary,
        "file",
        "large.pdf",
        "application/pdf",
        &huge_material,
    );
    let huge_material_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload/material")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", huge_material_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(huge_material_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(huge_material_res.status(), StatusCode::BAD_REQUEST);

    let submit_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, "another-attendee"
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", attendee_image_boundary),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(build_multipart_file_body(
                    attendee_image_boundary,
                    "file",
                    "ok.png",
                    "image/png",
                    &png,
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_forbidden_res.status(), StatusCode::FORBIDDEN);

    let submit_empty_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    "multipart/form-data; boundary=----empty-submission-boundary",
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from("------empty-submission-boundary--\r\n"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_empty_res.status(), StatusCode::BAD_REQUEST);

    let heic_boundary = "----heic-boundary";
    let heic_body = build_multipart_file_body(
        heic_boundary,
        "file",
        "photo.heic",
        "application/octet-stream",
        b"heic-data",
    );
    let heic_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", heic_boundary),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(heic_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(heic_res.status(), StatusCode::BAD_REQUEST);

    let huge_submission = vec![0u8; 5 * 1024 * 1024 + 1];
    let huge_submission_boundary = "----huge-submission-boundary";
    let huge_submission_body = build_multipart_file_body(
        huge_submission_boundary,
        "file",
        "big.png",
        "image/png",
        &huge_submission,
    );
    let huge_submission_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", huge_submission_boundary),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(huge_submission_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(huge_submission_res.status(), StatusCode::BAD_REQUEST);

    let mismatch_codelab_boundary = "----mismatch-codelab-boundary";
    let mismatch_codelab_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    another_codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    format!(
                        "multipart/form-data; boundary={}",
                        mismatch_codelab_boundary
                    ),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(build_multipart_file_body(
                    mismatch_codelab_boundary,
                    "file",
                    "ok.png",
                    "image/png",
                    &png,
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mismatch_codelab_res.status(), StatusCode::FORBIDDEN);

    let invalid_name_boundary = "----invalid-submission-name-boundary";
    let invalid_name_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", invalid_name_boundary),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(build_multipart_file_body(
                    invalid_name_boundary,
                    "file",
                    "???",
                    "application/octet-stream",
                    b"abc",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_name_res.status(), StatusCode::BAD_REQUEST);

    let text_boundary = "----text-submission-boundary";
    let text_upload_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", text_boundary),
                )
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(build_multipart_file_body(
                    text_boundary,
                    "file",
                    "note.txt",
                    "text/plain",
                    b"plain text",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(text_upload_res.status(), StatusCode::OK);
    let text_upload_body = axum::body::to_bytes(text_upload_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let text_upload_json: Value = serde_json::from_slice(&text_upload_body).unwrap();
    assert_eq!(text_upload_json["file_name"], "note.txt");

    sqlx::query(&test_app.state.q(
        "INSERT INTO submissions (id, codelab_id, attendee_id, file_path, file_name, file_size, submission_type, link_url) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    ))
    .bind("filler-submission")
    .bind(&codelab.id)
    .bind(&attendee_id)
    .bind("/uploads/submissions/filler.bin")
    .bind("filler.bin")
    .bind((10 * 1024 * 1024 - 1) as i64)
    .bind("file")
    .bind::<Option<String>>(None)
    .execute(&test_app.state.pool)
    .await
    .unwrap();

    let exceed_total_boundary = "----exceed-total-boundary";
    let exceed_total_res = test_app
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions",
                    codelab.id, attendee_id
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", exceed_total_boundary),
                )
                .header(header::COOKIE, attendee_cookie)
                .header("x-csrf-token", attendee_csrf)
                .body(Body::from(build_multipart_file_body(
                    exceed_total_boundary,
                    "file",
                    "tiny.bin",
                    "application/octet-stream",
                    b"0123456789",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(exceed_total_res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_backup_custom_restore_and_empty_upload_errors() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let workspace_dir = tempfile::tempdir().unwrap();
    let _workspace_guard = EnvVarGuard::set(
        "WORKSPACE_BASE",
        workspace_dir.path().to_string_lossy().to_string(),
    );

    let empty_boundary = "----empty-backup-boundary";
    let empty_body = format!("--{}--\r\n", empty_boundary);
    let inspect_empty_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/inspect")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", empty_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(empty_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inspect_empty_res.status(), StatusCode::BAD_REQUEST);

    let restore_empty_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/restore")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", empty_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(empty_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restore_empty_res.status(), StatusCode::BAD_REQUEST);

    let _backup_size_guard = EnvVarGuard::set("BACKUP_MAX_BYTES", "16");
    let oversized_boundary = "----oversized-backup-boundary";
    let oversized_body = build_multipart_file_body(
        oversized_boundary,
        "file",
        "oversized.bin",
        "application/octet-stream",
        &[1u8; 32],
    );
    let inspect_oversized_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/inspect")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", oversized_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(oversized_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inspect_oversized_res.status(), StatusCode::BAD_REQUEST);

    let restore_oversized_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/restore")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", oversized_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(oversized_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restore_oversized_res.status(), StatusCode::BAD_REQUEST);
    drop(_backup_size_guard);

    let timestamp = "2026-01-01T00:00:00Z";
    let backup_payload = json!({
        "version": 1,
        "created_at": timestamp,
        "data": {
            "codelabs": [{
                "id": "lab-restore",
                "title": "Restored Lab",
                "description": "desc",
                "author": "author",
                "is_public": true,
                "quiz_enabled": true,
                "require_quiz": false,
                "require_feedback": false,
                "require_submission": false,
                "guide_markdown": "# guide",
                "created_at": timestamp
            }],
            "steps": [{
                "id": "step-restore",
                "codelab_id": "lab-restore",
                "step_number": 1,
                "title": "Step",
                "content_markdown": "content"
            }],
            "attendees": [{
                "id": "att-restore",
                "codelab_id": "lab-restore",
                "name": "Attendee",
                "code": "ENC::dummy",
                "email": "att@example.com",
                "current_step": 1,
                "is_completed": false,
                "completed_at": Value::Null,
                "created_at": timestamp
            }],
            "help_requests": [{
                "id": "help-restore",
                "codelab_id": "lab-restore",
                "attendee_id": "att-restore",
                "attendee_name": "Attendee",
                "step_number": 1,
                "status": "pending",
                "created_at": timestamp
            }],
            "chat_messages": [{
                "id": "chat-restore",
                "codelab_id": "lab-restore",
                "sender_name": "Attendee",
                "message": "hello",
                "msg_type": "chat",
                "target_id": Value::Null,
                "sender_id": Value::Null,
                "created_at": timestamp
            }],
            "feedback": [{
                "id": "fb-restore",
                "codelab_id": "lab-restore",
                "attendee_id": "att-restore",
                "difficulty": "3",
                "satisfaction": "4",
                "comment": "good",
                "created_at": timestamp
            }],
            "materials": [{
                "id": "mat-restore",
                "codelab_id": "lab-restore",
                "title": "Material",
                "material_type": "link",
                "link_url": "https://example.com",
                "file_path": Value::Null,
                "created_at": timestamp
            }],
            "quizzes": [{
                "id": "quiz-restore",
                "codelab_id": "lab-restore",
                "question": "2+2?",
                "quiz_type": "multiple_choice",
                "options": "[\"3\",\"4\"]",
                "correct_answer": 1,
                "correct_answers": "[1]",
                "created_at": timestamp
            }],
            "quiz_submissions": [{
                "id": "quiz-sub-restore",
                "codelab_id": "lab-restore",
                "attendee_id": "att-restore",
                "quiz_id": "quiz-restore",
                "answer": "4",
                "is_correct": true,
                "created_at": timestamp
            }],
            "submissions": [{
                "id": "submission-restore",
                "codelab_id": "lab-restore",
                "attendee_id": "att-restore",
                "file_path": "/uploads/submissions/file.webp",
                "file_name": "file.webp",
                "file_size": 12,
                "submission_type": "file",
                "link_url": Value::Null,
                "created_at": timestamp
            }],
            "audit_logs": [{
                "id": "audit-restore",
                "action": "restore_action",
                "actor_type": "admin",
                "actor_id": "admin",
                "target_id": Value::Null,
                "codelab_id": "lab-restore",
                "ip": "127.0.0.1",
                "user_agent": "test-agent",
                "metadata": Value::Null,
                "created_at": timestamp
            }],
            "codeserver_workspaces": [{
                "id": "workspace-restore",
                "codelab_id": "lab-restore",
                "url": "http://codeserver",
                "structure_type": "folder",
                "created_at": timestamp
            }],
            "ai_conversations": [{
                "id": "ai-conv-restore",
                "codelab_id": "lab-restore",
                "user_id": "admin",
                "user_type": "admin",
                "user_name": "Admin",
                "step_number": 1,
                "question": "q",
                "answer": "a",
                "model": "gemini-3-flash-preview",
                "usage_metadata": "{}",
                "created_at": timestamp
            }],
            "ai_threads": [{
                "id": "ai-thread-restore",
                "title": "Thread",
                "user_id": "admin",
                "user_type": "admin",
                "codelab_id": "lab-restore",
                "created_at": timestamp,
                "updated_at": timestamp
            }],
            "ai_messages": [{
                "id": "ai-msg-restore",
                "thread_id": "ai-thread-restore",
                "role": "assistant",
                "content": "hello",
                "grounding_metadata": "{}",
                "usage_metadata": "{}",
                "created_at": timestamp
            }],
            "inline_comment_threads": [{
                "id": "thread-restore",
                "codelab_id": "lab-restore",
                "anchor_key": "step-1:0-4",
                "target_type": "step",
                "target_step_id": "step-restore",
                "start_offset": 0,
                "end_offset": 4,
                "selected_text": "Step",
                "content_hash": "hash",
                "created_by_attendee_id": "att-restore",
                "created_at": timestamp
            }],
            "inline_comment_messages": [{
                "id": "msg-restore",
                "thread_id": "thread-restore",
                "codelab_id": "lab-restore",
                "author_role": "attendee",
                "author_id": "att-restore",
                "author_name": "Attendee",
                "message": "hello",
                "created_at": timestamp
            }]
        }
    });

    let backup_zip = build_backup_zip(
        &backup_payload,
        &[
            ("uploads/", b""),
            ("uploads/deep/file.txt", b"upload"),
            ("workspaces/lab-restore/file.txt", b"workspace"),
        ],
    );

    let inspect_boundary = "----inspect-custom-boundary";
    let inspect_body = build_multipart_text_and_file_body(
        inspect_boundary,
        "meta",
        "custom-inspect",
        "file",
        "custom-backup.zip",
        "application/zip",
        &backup_zip,
    );
    let inspect_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/inspect")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", inspect_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(inspect_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inspect_res.status(), StatusCode::OK);

    let restore_boundary = "----restore-custom-boundary";
    let restore_body = build_multipart_text_and_file_body(
        restore_boundary,
        "meta",
        "custom-restore",
        "file",
        "custom-backup.zip",
        "application/zip",
        &backup_zip,
    );
    let restore_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/restore")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", restore_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(restore_body))
                .unwrap(),
        )
        .await
        .unwrap();
    if restore_res.status() != StatusCode::OK {
        let status = restore_res.status();
        let body = axum::body::to_bytes(restore_res.into_body(), usize::MAX)
            .await
            .unwrap();
        panic!(
            "custom restore failed: {} - {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    let mut reordered_backup_zip = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(IoCursor::new(&mut reordered_backup_zip));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("uploads/before-backup.txt", options)
            .unwrap();
        zip.write_all(b"before").unwrap();
        zip.start_file("backup.json", options).unwrap();
        zip.write_all(
            serde_json::to_string_pretty(&backup_payload)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        zip.finish().unwrap();
    }
    let reordered_restore_boundary = "----reordered-restore-boundary";
    let reordered_restore_body = build_multipart_text_and_file_body(
        reordered_restore_boundary,
        "meta",
        "reordered",
        "file",
        "reordered-backup.zip",
        "application/zip",
        &reordered_backup_zip,
    );
    let reordered_restore_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/backup/restore")
                .header(
                    "Content-Type",
                    format!(
                        "multipart/form-data; boundary={}",
                        reordered_restore_boundary
                    ),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(reordered_restore_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reordered_restore_res.status(), StatusCode::OK);

    let codelab_count: i64 = sqlx::query_scalar(&test_app.state.q("SELECT COUNT(*) FROM codelabs"))
        .fetch_one(&test_app.state.pool)
        .await
        .unwrap();
    let step_count: i64 = sqlx::query_scalar(&test_app.state.q("SELECT COUNT(*) FROM steps"))
        .fetch_one(&test_app.state.pool)
        .await
        .unwrap();
    let help_count: i64 =
        sqlx::query_scalar(&test_app.state.q("SELECT COUNT(*) FROM help_requests"))
            .fetch_one(&test_app.state.pool)
            .await
            .unwrap();
    let thread_count: i64 = sqlx::query_scalar(
        &test_app
            .state
            .q("SELECT COUNT(*) FROM inline_comment_threads"),
    )
    .fetch_one(&test_app.state.pool)
    .await
    .unwrap();
    let ai_message_count: i64 =
        sqlx::query_scalar(&test_app.state.q("SELECT COUNT(*) FROM ai_messages"))
            .fetch_one(&test_app.state.pool)
            .await
            .unwrap();
    assert_eq!(codelab_count, 1);
    assert_eq!(step_count, 1);
    assert_eq!(help_count, 1);
    assert_eq!(thread_count, 1);
    assert_eq!(ai_message_count, 1);
}

#[tokio::test]
async fn test_websocket_query_token_and_extra_message_paths() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let (_attendee_cookie, _attendee_csrf, attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "WsQueryUser",
        "ws-query-code",
    )
    .await;

    let attendee_claims = SessionClaims {
        sub: attendee_id.clone(),
        role: "attendee".to_string(),
        codelab_id: Some(codelab.id.clone()),
        iss: test_app.state.auth.issuer.clone(),
        aud: test_app.state.auth.audience.clone(),
        iat: 1,
        exp: usize::MAX / 2,
    };
    let admin_claims = SessionClaims {
        sub: test_app.state.admin_id.clone(),
        role: "admin".to_string(),
        codelab_id: None,
        iss: test_app.state.auth.issuer.clone(),
        aud: test_app.state.auth.audience.clone(),
        iat: 1,
        exp: usize::MAX / 2,
    };
    let attendee_token = test_app.state.auth.issue_token(&attendee_claims).unwrap();
    let admin_token = test_app.state.auth.issue_token(&admin_claims).unwrap();

    let Some(listener) = bind_local_listener_or_skip().await else {
        return;
    };
    let addr = listener.local_addr().unwrap();
    let ws_app = create_router(test_app.state.clone());
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            ws_app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let admin_ws_url = format!(
        "ws://{}/api/ws/{}?as=admin&token={}",
        addr, codelab.id, admin_token
    );
    let attendee_ws_url = format!(
        "ws://{}/api/ws/{}?as=attendee&token={}",
        addr, codelab.id, attendee_token
    );
    let bad_ws_url = format!(
        "ws://{}/api/ws/{}?as=attendee&token={}",
        addr, codelab.id, admin_token
    );
    let mismatch_ws_url = format!(
        "ws://{}/api/ws/{}?as=attendee&token={}",
        addr, "different-codelab", attendee_token
    );
    let weird_role_ws_url = format!(
        "ws://{}/api/ws/{}?as=weird&token={}",
        addr, codelab.id, attendee_token
    );

    let bad_conn = connect_async(&bad_ws_url).await;
    assert!(bad_conn.is_err());
    if let Err(tokio_tungstenite::tungstenite::Error::Http(res)) = bad_conn {
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    let mismatch_conn = connect_async(&mismatch_ws_url).await;
    assert!(mismatch_conn.is_err());
    if let Err(tokio_tungstenite::tungstenite::Error::Http(res)) = mismatch_conn {
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    let (admin_ws, _) = connect_async(&admin_ws_url).await.unwrap();
    let (attendee_ws, _) = connect_async(&attendee_ws_url).await.unwrap();
    let (weird_ws, _) = connect_async(&weird_role_ws_url).await.unwrap();
    let (mut admin_write, mut admin_read) = admin_ws.split();
    let (mut attendee_write, mut attendee_read) = attendee_ws.split();
    let (mut weird_write, _) = weird_ws.split();

    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "chat",
                "message": "x".repeat(2101)
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "dm",
                "target_id": "facilitator",
                "message": "x".repeat(2101)
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "dm",
                "target_id": "facilitator",
                "message": "short dm"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "dm",
                "message": "no-target"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "dm",
                "target_id": "missing-user",
                "message": "no-session-user"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({ "type": "step_progress", "step_number": 0 })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({ "type": "step_progress", "step_number": 2 })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({ "type": "step_progress", "step_number": "bad" })
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "webrtc_signal",
                "target_id": "facilitator",
                "signal": { "kind": "direct" },
                "stream_type": "screen"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "webrtc_signal",
                "target_id": "missing-user",
                "signal": { "kind": "missing-direct" },
                "stream_type": "screen"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "attendee_screen_status",
                "status": "started"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    admin_write
        .send(WsMessage::Text(
            json!({
                "type": "webrtc_signal",
                "signal": { "kind": "broadcast" },
                "stream_type": "screen"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    admin_write
        .send(WsMessage::Text(
            json!({
                "type": "step_progress",
                "step_number": 3
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    admin_write
        .send(WsMessage::Text(
            json!({
                "type": "attendee_screen_status",
                "status": "started"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    weird_write
        .send(WsMessage::Text(
            json!({
                "type": "attendee_screen_status",
                "status": "started"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    admin_write
        .send(WsMessage::Text(
            json!({
                "type": "screen_share_status",
                "status": "facilitator_stopped"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();
    admin_write
        .send(WsMessage::Text(
            json!({ "type": "unknown_type", "x": 1 }).to_string().into(),
        ))
        .await
        .unwrap();
    attendee_write
        .send(WsMessage::Text("not-json".into()))
        .await
        .unwrap();
    admin_write.send(WsMessage::Close(None)).await.unwrap();
    sleep(Duration::from_millis(150)).await;
    attendee_write
        .send(WsMessage::Text(
            json!({
                "type": "attendee_screen_status",
                "status": "stopped"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

    for _ in 0..8 {
        let _ = tokio::time::timeout(Duration::from_millis(300), admin_read.next()).await;
        let _ = tokio::time::timeout(Duration::from_millis(300), attendee_read.next()).await;
    }

    sleep(Duration::from_millis(200)).await;

    let chat_count: i64 = sqlx::query_scalar(
        &test_app
            .state
            .q("SELECT COUNT(*) FROM chat_messages WHERE codelab_id = ?"),
    )
    .bind(&codelab.id)
    .fetch_one(&test_app.state.pool)
    .await
    .unwrap();
    assert_eq!(chat_count, 2);
    assert_eq!(
        test_app
            .state
            .active_screen_shares
            .get(&codelab.id)
            .map(|v| *v),
        Some(false)
    );

    attendee_write.send(WsMessage::Close(None)).await.unwrap();
    weird_write.send(WsMessage::Close(None)).await.unwrap();
    sleep(Duration::from_millis(300)).await;
    server.abort();
}

#[tokio::test]
async fn test_quiz_feedback_submission_and_codelab_error_paths() {
    let test_app = setup_test_app().await;
    let (admin_cookie, admin_csrf) = login_admin(&test_app.app, &test_app.state).await;
    let codelab = create_codelab_as_admin(&test_app.app, &admin_cookie, &admin_csrf, false).await;
    let another_codelab =
        create_codelab_with_options(&test_app.app, &admin_cookie, &admin_csrf, true, false).await;
    let (attendee_cookie, attendee_csrf, attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &codelab.id,
        "ErrorUser",
        "error-code",
    )
    .await;
    let (other_attendee_cookie, other_attendee_csrf, other_attendee_id) = register_attendee(
        &test_app.app,
        &test_app.state,
        &another_codelab.id,
        "OtherUser",
        "other-code",
    )
    .await;

    let materials_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/materials", another_codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(materials_forbidden_res.status(), StatusCode::FORBIDDEN);

    let quizzes_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/quizzes", another_codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(quizzes_forbidden_res.status(), StatusCode::FORBIDDEN);

    let update_quizzes_empty_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/codelabs/{}/quizzes", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(serde_json::to_vec(&json!([])).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_quizzes_empty_res.status(), StatusCode::BAD_REQUEST);

    let update_quizzes_default_answers_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/codelabs/{}/quizzes", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!([{
                        "question": "Q?",
                        "quiz_type": "multiple_choice",
                        "options": ["a", "b"],
                        "correct_answer": 0
                    }]))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_quizzes_default_answers_res.status(), StatusCode::OK);

    let submit_quiz_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/quizzes/submit",
                    another_codelab.id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "submissions": [{
                            "quiz_id": "dummy",
                            "answer": "a",
                            "is_correct": false
                        }]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_quiz_forbidden_res.status(), StatusCode::FORBIDDEN);

    let submit_quiz_empty_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/quizzes/submit", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "submissions": [] })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_quiz_empty_res.status(), StatusCode::BAD_REQUEST);

    let feedback_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/feedback", another_codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "difficulty": "3",
                        "satisfaction": "3",
                        "comment": "nope"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feedback_forbidden_res.status(), StatusCode::FORBIDDEN);

    let feedback_ok_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/feedback", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "difficulty": "4",
                        "satisfaction": "4",
                        "comment": "ok"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feedback_ok_res.status(), StatusCode::OK);

    let feedback_duplicate_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/feedback", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "difficulty": "5",
                        "satisfaction": "5",
                        "comment": "duplicate"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feedback_duplicate_res.status(), StatusCode::CONFLICT);

    sqlx::query(&test_app.state.q("DROP TABLE feedback"))
        .execute(&test_app.state.pool)
        .await
        .unwrap();
    let feedback_internal_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/codelabs/{}/feedback", codelab.id))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "difficulty": "2",
                        "satisfaction": "2",
                        "comment": "after drop"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        feedback_internal_res.status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );

    let link_empty_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    codelab.id, attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "url": " " })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(link_empty_res.status(), StatusCode::BAD_REQUEST);

    let link_long_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    codelab.id, attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(
                        &json!({ "url": format!("https://example.com/{}", "a".repeat(2050)) }),
                    )
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(link_long_res.status(), StatusCode::BAD_REQUEST);

    let link_scheme_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    codelab.id, attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "url": "ftp://example.com/file" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(link_scheme_res.status(), StatusCode::BAD_REQUEST);

    let link_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    another_codelab.id, attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "url": "https://example.com" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(link_forbidden_res.status(), StatusCode::FORBIDDEN);

    let link_attendee_mismatch_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    codelab.id, other_attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "url": "https://example.com/mismatch" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(link_attendee_mismatch_res.status(), StatusCode::FORBIDDEN);

    let long_title_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    codelab.id, attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "url": "https://example.com/very-long-title",
                        "title": "x".repeat(300)
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(long_title_res.status(), StatusCode::OK);
    let long_title_body = axum::body::to_bytes(long_title_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let long_title_json: Value = serde_json::from_slice(&long_title_body).unwrap();
    assert_eq!(
        long_title_json["file_name"]
            .as_str()
            .unwrap()
            .chars()
            .count(),
        200
    );

    let submissions_unauth_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/submissions", codelab.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submissions_unauth_res.status(), StatusCode::UNAUTHORIZED);

    let submissions_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/submissions", another_codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submissions_forbidden_res.status(), StatusCode::FORBIDDEN);

    let link_ok_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/link",
                    codelab.id, attendee_id
                ))
                .header("Content-Type", "application/json")
                .header(header::COOKIE, attendee_cookie.clone())
                .header("x-csrf-token", attendee_csrf.clone())
                .body(Body::from(
                    serde_json::to_vec(&json!({ "url": "https://example.com/repo" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(link_ok_res.status(), StatusCode::OK);
    let link_ok_body = axum::body::to_bytes(link_ok_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let link_ok_json: Value = serde_json::from_slice(&link_ok_body).unwrap();
    let submission_id = link_ok_json["id"].as_str().unwrap().to_string();

    let delete_unauth_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/{}",
                    codelab.id, attendee_id, submission_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_unauth_res.status(), StatusCode::UNAUTHORIZED);

    let delete_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/{}",
                    codelab.id, attendee_id, submission_id
                ))
                .header(header::COOKIE, other_attendee_cookie.clone())
                .header("x-csrf-token", other_attendee_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_forbidden_res.status(), StatusCode::FORBIDDEN);

    let delete_admin_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/codelabs/{}/attendees/{}/submissions/{}",
                    codelab.id, attendee_id, submission_id
                ))
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_admin_res.status(), StatusCode::NO_CONTENT);

    let import_empty_boundary = "----import-empty-boundary";
    let import_empty_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs/import")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", import_empty_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(format!("--{}--\r\n", import_empty_boundary)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(import_empty_res.status(), StatusCode::BAD_REQUEST);

    let import_too_large_boundary = "----import-too-large-boundary";
    let import_too_large_bytes = vec![0u8; 20 * 1024 * 1024 + 1];
    let import_too_large_body = build_multipart_file_body(
        import_too_large_boundary,
        "file",
        "huge.bin",
        "application/octet-stream",
        &import_too_large_bytes,
    );
    let import_too_large_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs/import")
                .header(
                    "Content-Type",
                    format!(
                        "multipart/form-data; boundary={}",
                        import_too_large_boundary
                    ),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(import_too_large_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(import_too_large_res.status(), StatusCode::BAD_REQUEST);

    let mut large_import_zip = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(IoCursor::new(&mut large_import_zip));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("codelab.json", options).unwrap();
        zip.write_all(
            serde_json::to_vec(&json!({
                "id": "legacy-id",
                "title": "Import Source",
                "description": "desc",
                "author": "author",
                "is_public": true,
                "quiz_enabled": true,
                "require_quiz": false,
                "require_feedback": false,
                "require_submission": false,
                "guide_markdown": Value::Null,
                "created_at": "2026-01-01T00:00:00Z"
            }))
            .unwrap()
            .as_slice(),
        )
        .unwrap();
        zip.start_file("step_01_big.md", options).unwrap();
        zip.write_all(&vec![b'a'; 50_001]).unwrap();
        zip.finish().unwrap();
    }
    let import_large_boundary = "----import-large-boundary";
    let import_large_body = build_multipart_file_body(
        import_large_boundary,
        "file",
        "large.zip",
        "application/zip",
        &large_import_zip,
    );
    let import_large_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/codelabs/import")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", import_large_boundary),
                )
                .header(header::COOKIE, admin_cookie.clone())
                .header("x-csrf-token", admin_csrf.clone())
                .body(Body::from(import_large_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(import_large_res.status(), StatusCode::BAD_REQUEST);

    let chat_forbidden_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/chat", another_codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(chat_forbidden_res.status(), StatusCode::FORBIDDEN);

    let chat_allowed_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/chat", codelab.id))
                .header(header::COOKIE, attendee_cookie.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(chat_allowed_res.status(), StatusCode::OK);

    sqlx::query(&test_app.state.q("DROP TABLE submissions"))
        .execute(&test_app.state.pool)
        .await
        .unwrap();
    let submissions_internal_res = test_app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/codelabs/{}/submissions", codelab.id))
                .header(header::COOKIE, admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        submissions_internal_res.status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );

    let _ = other_attendee_id;
}
