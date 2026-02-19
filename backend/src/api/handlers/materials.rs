use crate::domain::models::{Codelab, CreateMaterial, Material};
use crate::infrastructure::audit::{record_audit, AuditEntry};
use crate::infrastructure::database::AppState;
use crate::middleware::auth::AuthSession;
use crate::middleware::request_info::RequestInfo;
use crate::utils::error::{bad_request, forbidden, internal_error};
use crate::utils::validation::validate_material;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use axum_extra::extract::Multipart;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

const MAX_MATERIAL_UPLOAD_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub async fn get_materials(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    session: AuthSession,
) -> Result<Json<Vec<Material>>, (StatusCode, String)> {
    let codelab = sqlx::query_as::<_, Codelab>(&state.q("SELECT * FROM codelabs WHERE id = ?"))
        .bind(&codelab_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Codelab not found".to_string()))?;
    if !can_access_codelab(&codelab, &session) {
        return Err(forbidden());
    }

    let materials = sqlx::query_as::<_, Material>(
        "SELECT * FROM materials WHERE codelab_id = ? ORDER BY created_at ASC",
    )
    .bind(codelab_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(materials))
}

pub async fn add_material(
    State(state): State<Arc<AppState>>,
    Path(codelab_id): Path<String>,
    session: AuthSession,
    info: RequestInfo,
    Json(payload): Json<CreateMaterial>,
) -> Result<Json<Material>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    validate_material(&payload)?;
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
    .map_err(internal_error)?;

    let material = sqlx::query_as::<_, Material>("SELECT * FROM materials WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "material_add".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: Some(material.id.clone()),
            codelab_id: Some(material.codelab_id.clone()),
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(Json(material))
}

pub async fn delete_material(
    State(state): State<Arc<AppState>>,
    Path((_codelab_id, material_id)): Path<(String, String)>,
    session: AuthSession,
    info: RequestInfo,
) -> Result<StatusCode, (StatusCode, String)> {
    let admin = session.require_admin()?;
    // 만약 파일이라면 물리적 파일도 삭제해야 할까요?
    // 우선 DB 레코드만 삭제하도록 구현하겠습니다.
    // 필요하다면 나중에 파일 삭제 로직을 추가할 수 있습니다.

    sqlx::query("DELETE FROM materials WHERE id = ?")
        .bind(material_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    record_audit(
        &state,
        AuditEntry {
            action: "material_delete".to_string(),
            actor_type: "admin".to_string(),
            actor_id: Some(admin.sub),
            target_id: None,
            codelab_id: None,
            ip: Some(info.ip),
            user_agent: info.user_agent,
            metadata: None,
        },
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upload_material_file(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    info: RequestInfo,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = session.require_admin()?;
    if let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let filename = sanitize_filename(field.file_name().unwrap_or("file"));
        if filename.is_empty() {
            return Err(bad_request("invalid file name"));
        }
        let data = field.bytes().await.map_err(internal_error)?;
        if data.len() > MAX_MATERIAL_UPLOAD_SIZE {
            return Err(bad_request("file too large"));
        }

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
            .map_err(internal_error)?;

        fs::write(&file_path, data).await.map_err(internal_error)?;

        let response = serde_json::json!({
            "url": format!("/uploads/materials/{}", new_filename),
            "original_name": filename
        });

        record_audit(
            &state,
            AuditEntry {
                action: "material_upload".to_string(),
                actor_type: "admin".to_string(),
                actor_id: Some(admin.sub),
                target_id: None,
                codelab_id: None,
                ip: Some(info.ip),
                user_agent: info.user_agent,
                metadata: None,
            },
        )
        .await;

        return Ok(Json(response));
    }

    Err(bad_request("No file uploaded"))
}

fn can_access_codelab(codelab: &Codelab, session: &AuthSession) -> bool {
    match &session.claims {
        Some(claims) if claims.role == "admin" => true,
        Some(claims)
            if claims.role == "attendee"
                && claims.codelab_id.as_deref() == Some(codelab.id.as_str()) =>
        {
            true
        }
        _ => false,
    }
}

fn sanitize_filename(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-' {
            out.push(ch);
        }
    }
    if out.len() > 120 {
        out.truncate(120);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::{AuthSession, SessionClaims};

    fn session(role: &str, codelab_id: Option<&str>) -> AuthSession {
        AuthSession {
            claims: Some(SessionClaims {
                sub: "u1".to_string(),
                role: role.to_string(),
                codelab_id: codelab_id.map(|v| v.to_string()),
                iss: "test".to_string(),
                aud: "test".to_string(),
                iat: 0,
                exp: 1_000_000,
            }),
            admin_claims: None,
            attendee_claims: None,
        }
    }

    fn codelab(id: &str) -> Codelab {
        Codelab {
            id: id.to_string(),
            title: "t".to_string(),
            description: "d".to_string(),
            author: "a".to_string(),
            is_public: 1,
            quiz_enabled: 0,
            require_quiz: 0,
            require_feedback: 0,
            require_submission: 0,
            guide_markdown: None,
            created_at: None,
        }
    }

    #[test]
    fn can_access_codelab_respects_role_and_membership() {
        let lab = codelab("lab-1");
        assert!(can_access_codelab(&lab, &session("admin", None)));
        assert!(can_access_codelab(
            &lab,
            &session("attendee", Some("lab-1"))
        ));
        assert!(!can_access_codelab(
            &lab,
            &session("attendee", Some("other-lab"))
        ));
        assert!(!can_access_codelab(&lab, &session("guest", None)));
    }

    #[test]
    fn sanitize_filename_filters_and_truncates() {
        assert_eq!(sanitize_filename("a b/c?.pdf"), "abc.pdf");
        let long = "x".repeat(200);
        assert_eq!(sanitize_filename(&long).len(), 120);
    }
}
