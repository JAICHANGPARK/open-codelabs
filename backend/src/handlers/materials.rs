use crate::models::{CreateMaterial, Material};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;
use axum_extra::extract::Multipart;
use tokio::fs;

pub async fn get_materials(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
) -> Result<Json<Vec<Material>>, (StatusCode, String)> {
    let materials = sqlx::query_as::<_, Material>(
        "SELECT * FROM materials WHERE codelab_id = ? ORDER BY created_at ASC"
    )
    .bind(codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(materials))
}

pub async fn add_material(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    Json(payload): Json<CreateMaterial>,
) -> Result<Json<Material>, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO materials (id, codelab_id, title, material_type, link_url, file_path) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&codelab_id)
    .bind(&payload.title)
    .bind(&payload.material_type)
    .bind(&payload.link_url)
    .bind(&payload.file_path)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let material = sqlx::query_as::<_, Material>("SELECT * FROM materials WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(material))
}

pub async fn delete_material(
    State(state): State<Arc<AppState>>,
    Path((_codelab_id, material_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    // 만약 파일이라면 물리적 파일도 삭제해야 할까요? 
    // 우선 DB 레코드만 삭제하도록 구현하겠습니다. 
    // 필요하다면 나중에 파일 삭제 로직을 추가할 수 있습니다.

    sqlx::query("DELETE FROM materials WHERE id = ?")
        .bind(material_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upload_material_file(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let filename = field.file_name().unwrap_or("file").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Generate a unique filename to avoid collisions
        let unique_id = Uuid::new_v4().to_string();
        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let new_filename = if extension.is_empty() {
            unique_id
        } else {
            format!("{}.{}", unique_id, extension)
        };

        let upload_dir = "static/uploads/materials";
        let file_path = format!("{}/{}", upload_dir, new_filename);

        fs::create_dir_all(upload_dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        fs::write(&file_path, data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok(Json(serde_json::json!({
            "url": format!("/uploads/materials/{}", new_filename),
            "original_name": filename
        })));
    }

    Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))
}
