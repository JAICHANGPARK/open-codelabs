use axum::http::StatusCode;

pub fn bad_request(message: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message.to_string())
}

pub fn unauthorized() -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
}

pub fn forbidden() -> (StatusCode, String) {
    (StatusCode::FORBIDDEN, "Forbidden".to_string())
}

pub fn too_many_requests() -> (StatusCode, String) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests".to_string(),
    )
}

pub fn internal_error<E: std::fmt::Display>(error: E) -> (StatusCode, String) {
    tracing::error!("Internal error: {}", error);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
    )
}
