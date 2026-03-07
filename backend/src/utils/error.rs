//! Helpers for converting common failures into HTTP responses.

use axum::http::StatusCode;

/// Returns a `400 Bad Request` response with a caller-provided message.
pub fn bad_request(message: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message.to_string())
}

/// Returns a generic `401 Unauthorized` response.
pub fn unauthorized() -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
}

/// Returns a generic `403 Forbidden` response.
pub fn forbidden() -> (StatusCode, String) {
    (StatusCode::FORBIDDEN, "Forbidden".to_string())
}

/// Returns a generic `429 Too Many Requests` response.
pub fn too_many_requests() -> (StatusCode, String) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests".to_string(),
    )
}

/// Logs an internal error and returns a sanitized `500` response.
pub fn internal_error<E: std::fmt::Display>(error: E) -> (StatusCode, String) {
    tracing::error!("Internal error: {}", error);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_request_returns_status_and_message() {
        let (status, message) = bad_request("invalid payload");
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "invalid payload");
    }

    #[test]
    fn unauthorized_returns_expected_response() {
        let (status, message) = unauthorized();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Unauthorized");
    }

    #[test]
    fn forbidden_returns_expected_response() {
        let (status, message) = forbidden();
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(message, "Forbidden");
    }

    #[test]
    fn too_many_requests_returns_expected_response() {
        let (status, message) = too_many_requests();
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(message, "Too many requests");
    }

    #[test]
    fn internal_error_masks_detail() {
        let (status, message) = internal_error("db timeout");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "Internal server error");
    }
}
