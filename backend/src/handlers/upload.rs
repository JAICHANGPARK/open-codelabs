use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use axum_extra::extract::Multipart;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

pub async fn upload_image(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Generate a unique filename
        let new_filename = format!("{}.webp", Uuid::new_v4());
        let upload_dir = "static/uploads";
        let file_path = format!("{}/{}", upload_dir, new_filename);

        fs::create_dir_all(upload_dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        return Ok(Json(serde_json::json!({
            "url": format!("/uploads/{}", new_filename)
        })));
    }

    Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))
}
