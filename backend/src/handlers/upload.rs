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
        let file_name = field.file_name().unwrap_or("image.png").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Generate a unique filename
        let extension = std::path::Path::new(&file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png");

        // If content type is webp or user wants webp, we could convert here.
        // For simplicity, we just save it as is but with a UUID.
        let new_filename = format!("{}.{}", Uuid::new_v4(), extension);
        let upload_dir = "static/uploads";
        let file_path = format!("{}/{}", upload_dir, new_filename);

        fs::create_dir_all(upload_dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        fs::write(&file_path, data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok(Json(serde_json::json!({
            "url": format!("/uploads/{}", new_filename)
        })));
    }

    Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))
}
