use serde::{Deserialize, Serialize};

/// Raw audit-log row returned directly from SQL queries.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    /// Audit log row identifier.
    pub id: String,
    /// Machine-readable action name.
    pub action: String,
    /// Actor category such as `admin` or `attendee`.
    pub actor_type: String,
    /// Stable identifier of the acting user when known.
    pub actor_id: Option<String>,
    /// Identifier of the entity that was changed.
    pub target_id: Option<String>,
    /// Related codelab identifier when the event is codelab-scoped.
    pub codelab_id: Option<String>,
    /// Captured client IP address when available.
    pub ip: Option<String>,
    /// Captured client user agent when available.
    pub user_agent: Option<String>,
    /// JSON metadata serialized as text.
    pub metadata: Option<String>,
    /// Creation timestamp serialized as text for API output.
    pub created_at: String,
}

/// Submission row joined with attendee metadata before API-level normalization.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubmissionWithAttendeeRaw {
    /// Submission identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Related attendee identifier.
    pub attendee_id: String,
    /// Display name of the attendee who submitted the work.
    pub attendee_name: String,
    /// Stored file path for file submissions.
    pub file_path: String,
    /// Original filename or link title.
    pub file_name: String,
    /// Stored file size in bytes.
    pub file_size: i64,
    /// Submission variant such as `file` or `link`.
    pub submission_type: String,
    /// External URL when the submission is link-based.
    pub link_url: Option<String>,
    /// Submission creation timestamp.
    pub created_at: Option<String>,
}

/// Database row describing the workspace URL and structure mode of a codelab.
#[derive(sqlx::FromRow)]
pub struct WorkspaceRow {
    /// Workspace path or URL.
    pub url: String,
    /// Workspace layout mode such as `branch` or `folder`.
    pub structure_type: String,
}
