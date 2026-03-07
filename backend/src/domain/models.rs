//! Shared domain models used by handlers, persistence, and websocket flows.

use serde::{Deserialize, Serialize};

/// Stored codelab metadata returned by list/detail endpoints.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct Codelab {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_public: i32,
    /// Whether quizzes are enabled for this codelab.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub quiz_enabled: i32,
    /// Whether quiz completion is required before marking the codelab complete.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub require_quiz: i32,
    /// Whether learner feedback is required before completion.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub require_feedback: i32,
    /// Whether a submission is required before completion.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub require_submission: i32,
    pub guide_markdown: Option<String>,
    pub created_at: Option<String>,
}

/// Stored quiz row attached to a codelab step or overall lab flow.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Quiz {
    pub id: String,
    pub codelab_id: String,
    pub question: String,
    /// Quiz mode such as `multiple_choice` or `descriptive`.
    pub quiz_type: Option<String>,
    /// JSON-encoded option list persisted in the database.
    pub options: String,
    pub correct_answer: i32,
    /// JSON-encoded array of correct answer indices for multi-answer quizzes.
    pub correct_answers: Option<String>,
    pub created_at: Option<String>,
}

/// Payload used to create or replace a quiz definition from the API.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuiz {
    pub question: String,
    pub quiz_type: Option<String>,
    pub options: Vec<String>,
    pub correct_answer: i32,
    pub correct_answers: Option<Vec<i32>>,
}

fn to_bool<S>(v: &i32, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_bool(*v != 0)
}

fn from_bool<'de, D>(d: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let b = bool::deserialize(d)?;
    Ok(if b { 1 } else { 0 })
}

/// Stored codelab step content.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Step {
    pub id: String,
    pub codelab_id: String,
    pub step_number: i32,
    pub title: String,
    pub content_markdown: String,
}

/// Payload used to create or update codelab metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCodelab {
    pub title: String,
    pub description: String,
    pub author: String,
    pub is_public: Option<bool>,
    pub quiz_enabled: Option<bool>,
    pub require_quiz: Option<bool>,
    pub require_feedback: Option<bool>,
    pub require_submission: Option<bool>,
    pub guide_markdown: Option<String>,
}

/// Login payload for the built-in administrator session.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub admin_id: String,
    pub admin_pw: String,
}

/// Payload used to replace the ordered step list of a codelab.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStepsPayload {
    pub steps: Vec<CreateStep>,
}

/// Step payload accepted by create/update endpoints.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStep {
    /// Existing step identifier when updating in place.
    pub id: Option<String>,
    pub title: String,
    pub content_markdown: String,
}

/// Learner registration payload for joining a codelab.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationPayload {
    pub name: String,
    pub code: String,
    pub email: Option<String>,
}

/// Stored attendee record for a codelab registration.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attendee {
    pub id: String,
    pub codelab_id: String,
    pub name: String,
    pub code: String,
    pub email: Option<String>,
    pub current_step: i32,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_completed: i32,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
    /// Whether this attendee is currently sharing their screen.
    #[serde(default)]
    #[sqlx(default)]
    pub is_sharing_screen: bool,
}

/// Public attendee representation returned to clients.
#[derive(Debug, Serialize, Deserialize)]
pub struct AttendeePublic {
    pub id: String,
    pub codelab_id: String,
    pub name: String,
    pub email: Option<String>,
    pub current_step: i32,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_completed: i32,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
    /// Optional attendee token included when the caller is allowed to receive it.
    pub token: Option<String>,
    /// Whether this attendee is currently sharing their screen.
    #[serde(default)]
    pub is_sharing_screen: bool,
}

impl From<Attendee> for AttendeePublic {
    fn from(attendee: Attendee) -> Self {
        Self {
            id: attendee.id,
            codelab_id: attendee.codelab_id,
            name: attendee.name,
            email: attendee.email,
            current_step: attendee.current_step,
            is_completed: attendee.is_completed,
            completed_at: attendee.completed_at,
            created_at: attendee.created_at,
            token: None,
            is_sharing_screen: attendee.is_sharing_screen,
        }
    }
}

/// Certificate payload returned for a completed attendee.
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub attendee_name: String,
    pub codelab_title: String,
    pub codelab_id: String,
    pub author: String,
    pub completed_at: String,
    pub verification_url: String,
}

/// Payload used when an attendee requests help on a specific step.
#[derive(Debug, Serialize, Deserialize)]
pub struct HelpRequestPayload {
    pub step_number: i32,
}

/// Stored help request raised by an attendee.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HelpRequest {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub attendee_name: String,
    pub step_number: i32,
    pub status: String,
    pub created_at: Option<String>,
}

/// Stored chat message row used for websocket history endpoints.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatMessageRow {
    pub id: String,
    pub codelab_id: String,
    pub sender_name: String,
    pub message: String,
    /// Message kind such as room chat, direct message, or system event.
    pub msg_type: String,
    /// Optional recipient identifier for direct messages.
    pub target_id: Option<String>,
    /// Optional sender identifier when the sender is a known attendee/admin.
    pub sender_id: Option<String>,
    pub created_at: Option<String>,
}

/// Anchor metadata for an inline comment thread in guide or step content.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct InlineCommentThread {
    pub id: String,
    pub codelab_id: String,
    pub anchor_key: String,
    /// Anchor target kind: `step` or `guide`.
    pub target_type: String,
    pub target_step_id: Option<String>,
    /// Character offset of the highlighted range start.
    pub start_offset: i32,
    /// Character offset of the highlighted range end.
    pub end_offset: i32,
    pub selected_text: String,
    /// Hash of the source content used to detect stale anchors.
    pub content_hash: String,
    pub created_by_attendee_id: String,
    pub created_at: Option<String>,
}

/// Message stored inside an inline comment thread.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct InlineCommentMessage {
    pub id: String,
    pub thread_id: String,
    pub codelab_id: String,
    /// Author role, typically `attendee` or `admin`.
    pub author_role: String,
    pub author_id: String,
    pub author_name: String,
    pub message: String,
    pub created_at: Option<String>,
}

/// Inline comment thread plus the ordered messages it contains.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineCommentThreadWithMessages {
    #[serde(flatten)]
    pub thread: InlineCommentThread,
    pub messages: Vec<InlineCommentMessage>,
}

/// Payload used to open a new inline comment thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInlineCommentPayload {
    pub anchor_key: String,
    /// Anchor target kind: `step` or `guide`.
    pub target_type: String,
    pub target_step_id: Option<String>,
    pub start_offset: i32,
    pub end_offset: i32,
    pub selected_text: String,
    pub content_hash: String,
    pub message: String,
}

/// Payload used to reply to an existing inline comment thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyInlineCommentPayload {
    pub message: String,
    pub content_hash: String,
}

/// Learner feedback payload accepted by the API.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedback {
    pub difficulty: String,
    pub satisfaction: String,
    pub comment: Option<String>,
}

/// Stored feedback record for a codelab.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Feedback {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: Option<String>,
    pub difficulty: String,
    pub satisfaction: String,
    pub comment: Option<String>,
    pub created_at: Option<String>,
}

/// Stored teaching material associated with a codelab.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Material {
    pub id: String,
    pub codelab_id: String,
    pub title: String,
    /// Material kind: `link` or `file`.
    pub material_type: String,
    /// Source URL when `material_type` is `link`.
    pub link_url: Option<String>,
    /// Uploaded file path when `material_type` is `file`.
    pub file_path: Option<String>,
    pub created_at: Option<String>,
}

/// Payload used to add material to a codelab.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMaterial {
    pub title: String,
    pub material_type: String,
    pub link_url: Option<String>,
    pub file_path: Option<String>,
}

/// Stored quiz submission for a specific attendee and quiz.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSubmission {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub quiz_id: String,
    pub answer: String,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_correct: i32,
    pub created_at: Option<String>,
}

/// Single quiz answer submission payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuizSubmission {
    pub quiz_id: String,
    pub answer: String,
    pub is_correct: bool,
}

/// Batch payload used when submitting multiple quiz answers at once.
#[derive(Debug, Serialize, Deserialize)]
pub struct QuizSubmissionPayload {
    pub submissions: Vec<CreateQuizSubmission>,
}

/// Quiz submission joined with attendee metadata for admin views.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSubmissionWithAttendee {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub attendee_name: String,
    pub quiz_id: String,
    pub answer: String,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_correct: i32,
    pub created_at: Option<String>,
}

/// Stored learner submission, which may be a file or a link.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    /// Submission kind such as `file` or `link`.
    #[serde(default = "default_submission_type")]
    pub submission_type: String,
    /// External submission URL when the submission is link-based.
    pub link_url: Option<String>,
    pub created_at: Option<String>,
}

/// Submission joined with attendee metadata for admin-facing lists.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubmissionWithAttendee {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub attendee_name: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    /// Submission kind such as `file` or `link`.
    #[serde(default = "default_submission_type")]
    pub submission_type: String,
    /// External submission URL when the submission is link-based.
    pub link_url: Option<String>,
    pub created_at: Option<String>,
}

/// Payload used when an attendee submits an external link instead of a file.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubmissionLink {
    pub url: String,
    pub title: Option<String>,
}

/// Default submission type used for older rows that predate the field.
pub fn default_submission_type() -> String {
    "file".to_string()
}

/// Stored AI conversation entry for the legacy conversation endpoint.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiConversation {
    pub id: String,
    pub codelab_id: String,
    pub user_id: String,
    /// User category such as `admin` or `attendee`.
    pub user_type: String,
    pub user_name: String,
    pub step_number: Option<i32>,
    pub question: String,
    pub answer: String,
    pub model: Option<String>,
    /// Provider-specific usage metadata serialized as JSON text.
    pub usage_metadata: Option<String>,
    pub created_at: Option<String>,
}

/// Payload used to persist an AI conversation exchange.
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveAiConversationPayload {
    pub codelab_id: String,
    pub step_number: Option<i32>,
    pub question: String,
    pub answer: String,
    pub model: Option<String>,
    #[serde(default)]
    pub usage_metadata: Option<serde_json::Value>,
}

/// Stored AI thread metadata used by the threaded chat endpoints.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiThread {
    pub id: String,
    pub title: String,
    pub user_id: String,
    /// User category such as `admin` or `attendee`.
    pub user_type: String,
    pub codelab_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Stored message inside an AI thread.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiMessage {
    pub id: String,
    pub thread_id: String,
    /// Chat role such as `user`, `assistant`, or `system`.
    pub role: String,
    pub content: String,
    /// Provider-specific grounding metadata serialized as JSON text.
    pub grounding_metadata: Option<String>,
    /// Provider-specific usage metadata serialized as JSON text.
    pub usage_metadata: Option<String>,
    pub created_at: Option<String>,
}

/// Payload used to create a new AI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAiThreadPayload {
    pub title: String,
    #[serde(default)]
    pub codelab_id: Option<String>,
}

/// Payload used to append a message to an AI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct AddAiMessagePayload {
    /// Chat role such as `user`, `assistant`, or `system`.
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub grounding_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub usage_metadata: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codelab_serialization() {
        let codelab = Codelab {
            id: "test-id".to_string(),
            title: "Test Title".to_string(),
            description: "Test Description".to_string(),
            author: "Test Author".to_string(),
            is_public: 1,
            quiz_enabled: 0,
            require_quiz: 0,
            require_feedback: 0,
            require_submission: 0,
            guide_markdown: None,
            created_at: Some("2023-01-01".to_string()),
        };

        let json = serde_json::to_string(&codelab).unwrap();
        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"title\":\"Test Title\""));
    }

    #[test]
    fn test_create_codelab_deserialization() {
        let json = r#"{"title":"New Codelab","description":"Desc","author":"John"}"#;
        let create_codelab: CreateCodelab = serde_json::from_str(json).unwrap();

        assert_eq!(create_codelab.title, "New Codelab");
        assert_eq!(create_codelab.description, "Desc");
        assert_eq!(create_codelab.author, "John");
    }

    #[test]
    fn test_registration_payload_deserialization() {
        let json = r#"{"name":"Alice","code":"1234"}"#;
        let payload: RegistrationPayload = serde_json::from_str(json).unwrap();

        assert_eq!(payload.name, "Alice");
        assert_eq!(payload.code, "1234");
    }

    #[test]
    fn test_attendee_public_from_attendee() {
        let attendee = Attendee {
            id: "a1".to_string(),
            codelab_id: "c1".to_string(),
            name: "Alice".to_string(),
            code: "CODE".to_string(),
            email: Some("alice@example.com".to_string()),
            current_step: 3,
            is_completed: 1,
            completed_at: Some("2026-02-01".to_string()),
            created_at: Some("2026-01-01".to_string()),
            is_sharing_screen: true,
        };

        let public: AttendeePublic = attendee.into();
        assert_eq!(public.id, "a1");
        assert_eq!(public.codelab_id, "c1");
        assert_eq!(public.name, "Alice");
        assert_eq!(public.email.as_deref(), Some("alice@example.com"));
        assert_eq!(public.current_step, 3);
        assert_eq!(public.is_completed, 1);
        assert_eq!(public.completed_at.as_deref(), Some("2026-02-01"));
        assert_eq!(public.created_at.as_deref(), Some("2026-01-01"));
        assert_eq!(public.token, None);
        assert!(public.is_sharing_screen);
    }

    #[test]
    fn test_default_submission_type_is_file() {
        assert_eq!(default_submission_type(), "file");
    }

    #[test]
    fn test_codelab_serialization_with_false_flags() {
        let codelab = Codelab {
            id: "test-id-2".to_string(),
            title: "Title".to_string(),
            description: "Description".to_string(),
            author: "Author".to_string(),
            is_public: 0,
            quiz_enabled: 0,
            require_quiz: 0,
            require_feedback: 0,
            require_submission: 0,
            guide_markdown: None,
            created_at: None,
        };

        let json = serde_json::to_string(&codelab).unwrap();
        assert!(json.contains("\"is_public\":false"));
    }
}
