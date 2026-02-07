use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::middleware::auth::AuthSession;
use crate::utils::error::{bad_request, internal_error, unauthorized};
use crate::middleware::request_info::RequestInfo;
use crate::infrastructure::database::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use axum_extra::extract::Multipart;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

const MAX_IMAGE_UPLOAD_SIZE: usize = 5 * 1024 * 1024; // 5MB

pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (actor_type, actor_id, codelab_id) = if let Ok(admin) = session.require_admin() {
        ("admin".to_string(), Some(admin.sub), None)
    } else if let Ok(attendee) = session.require_attendee() {
        (
            "attendee".to_string(),
            Some(attendee.sub),
            attendee.codelab_id,
        )
    } else {
        return Err(unauthorized());
    };
    if let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let data = field.bytes().await.map_err(internal_error)?;
        if data.len() > MAX_IMAGE_UPLOAD_SIZE {
            return Err(bad_request("file too large"));
        }

        // Generate a unique filename
        let new_filename = format!("{}.webp", Uuid::new_v4());
        let upload_dir = "static/uploads";
        let file_path = format!("{}/{}", upload_dir, new_filename);

        fs::create_dir_all(upload_dir)
            .await
            .map_err(internal_error)?;

        // Convert to WebP in blocking thread
        let data_clone = data.clone();
        let file_path_clone = file_path.clone();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let img = image::load_from_memory(&data_clone)
                .map_err(|e| format!("Failed to load image: {}", e))?;

            img.save_with_format(&file_path_clone, image::ImageFormat::WebP)
                .map_err(|e| format!("Failed to save image: {}", e))?;

            Ok(())
        })
        .await
        .map_err(internal_error)?
        .map_err(|e| bad_request(&e))?;

        record_audit(
            &state,
            AuditEntry {
                action: "image_upload".to_string(),
                actor_type,
                actor_id,
                target_id: None,
                codelab_id,
                ip: Some(info.ip),
                user_agent: info.user_agent,
                metadata: None,
            },
        )
        .await;

        return Ok(Json(serde_json::json!({
            "url": format!("/uploads/{}", new_filename)
        })));
    }

    Err(bad_request("No file uploaded"))
}
