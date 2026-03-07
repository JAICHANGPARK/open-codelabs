use serde::{Deserialize, Serialize};

/// Raw audit-log row returned directly from SQL queries.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: String,
    pub action: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub target_id: Option<String>,
    pub codelab_id: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

/// Submission row joined with attendee metadata before API-level normalization.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubmissionWithAttendeeRaw {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub attendee_name: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub submission_type: String,
    pub link_url: Option<String>,
    pub created_at: Option<String>,
}

/// Database row describing the workspace URL and structure mode of a codelab.
#[derive(sqlx::FromRow)]
pub struct WorkspaceRow {
    pub url: String,
    pub structure_type: String,
}
