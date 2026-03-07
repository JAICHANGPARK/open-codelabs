//! Shared domain models used by handlers, persistence, and websocket flows.

use serde::{Deserialize, Serialize};

/// Stored codelab metadata returned by list/detail endpoints.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct Codelab {
    /// Codelab identifier.
    pub id: String,
    /// Display title shown to learners and admins.
    pub title: String,
    /// Short summary of the codelab.
    pub description: String,
    /// Author name displayed with the codelab.
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
    /// Optional guide markdown rendered alongside the steps.
    pub guide_markdown: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Stored quiz row attached to a codelab step or overall lab flow.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Quiz {
    /// Quiz identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Prompt displayed to the learner.
    pub question: String,
    /// Quiz mode such as `multiple_choice` or `descriptive`.
    pub quiz_type: Option<String>,
    /// JSON-encoded option list persisted in the database.
    pub options: String,
    /// Index of the correct answer for single-answer quizzes.
    pub correct_answer: i32,
    /// JSON-encoded array of correct answer indices for multi-answer quizzes.
    pub correct_answers: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Payload used to create or replace a quiz definition from the API.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuiz {
    /// Prompt displayed to the learner.
    pub question: String,
    /// Quiz mode such as `multiple_choice` or `descriptive`.
    pub quiz_type: Option<String>,
    /// Answer options presented to the learner.
    pub options: Vec<String>,
    /// Correct answer index for single-answer quizzes.
    pub correct_answer: i32,
    /// Correct answer indices for multi-answer quizzes.
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
    /// Step identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// 1-based step order shown in the UI.
    pub step_number: i32,
    /// Step title.
    pub title: String,
    /// Markdown body for the step.
    pub content_markdown: String,
}

/// Payload used to create or update codelab metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCodelab {
    /// Display title shown to learners and admins.
    pub title: String,
    /// Short summary of the codelab.
    pub description: String,
    /// Author name displayed with the codelab.
    pub author: String,
    /// Whether the codelab can be joined without admin access.
    pub is_public: Option<bool>,
    /// Whether quizzes are enabled for this codelab.
    pub quiz_enabled: Option<bool>,
    /// Whether quiz completion is required before completion.
    pub require_quiz: Option<bool>,
    /// Whether learner feedback is required before completion.
    pub require_feedback: Option<bool>,
    /// Whether a submission is required before completion.
    pub require_submission: Option<bool>,
    /// Optional guide markdown rendered alongside the steps.
    pub guide_markdown: Option<String>,
}

/// Login payload for the built-in administrator session.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    /// Submitted administrator identifier.
    pub admin_id: String,
    /// Submitted administrator password.
    pub admin_pw: String,
}

/// Payload used to replace the ordered step list of a codelab.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStepsPayload {
    /// Ordered list of steps that should replace the existing step set.
    pub steps: Vec<CreateStep>,
}

/// Step payload accepted by create/update endpoints.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStep {
    /// Existing step identifier when updating in place.
    pub id: Option<String>,
    /// Step title.
    pub title: String,
    /// Markdown body for the step.
    pub content_markdown: String,
}

/// Learner registration payload for joining a codelab.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationPayload {
    /// Learner display name.
    pub name: String,
    /// Learner-entered join code.
    pub code: String,
    /// Optional email address captured during registration.
    pub email: Option<String>,
}

/// Stored attendee record for a codelab registration.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attendee {
    /// Attendee identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Learner display name.
    pub name: String,
    /// Stored encrypted join code or decrypted value for admin views.
    pub code: String,
    /// Optional email address captured during registration.
    pub email: Option<String>,
    /// Current step the attendee has reached.
    pub current_step: i32,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_completed: i32,
    /// Completion timestamp when the attendee finishes the codelab.
    pub completed_at: Option<String>,
    /// Registration timestamp.
    pub created_at: Option<String>,
    /// Whether this attendee is currently sharing their screen.
    #[serde(default)]
    #[sqlx(default)]
    pub is_sharing_screen: bool,
}

/// Public attendee representation returned to clients.
#[derive(Debug, Serialize, Deserialize)]
pub struct AttendeePublic {
    /// Attendee identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Learner display name.
    pub name: String,
    /// Optional email address captured during registration.
    pub email: Option<String>,
    /// Current step the attendee has reached.
    pub current_step: i32,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_completed: i32,
    /// Completion timestamp when the attendee finishes the codelab.
    pub completed_at: Option<String>,
    /// Registration timestamp.
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
    /// Learner name printed on the certificate.
    pub attendee_name: String,
    /// Codelab title printed on the certificate.
    pub codelab_title: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Author name printed on the certificate.
    pub author: String,
    /// Completion timestamp displayed on the certificate.
    pub completed_at: String,
    /// Public verification URL for the certificate.
    pub verification_url: String,
}

/// Payload used when an attendee requests help on a specific step.
#[derive(Debug, Serialize, Deserialize)]
pub struct HelpRequestPayload {
    /// Step number where the learner needs help.
    pub step_number: i32,
}

/// Stored help request raised by an attendee.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HelpRequest {
    /// Help request identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Attendee who raised the request.
    pub attendee_id: String,
    /// Display name of the attendee.
    pub attendee_name: String,
    /// Step number where help was requested.
    pub step_number: i32,
    /// Request status such as `open` or `resolved`.
    pub status: String,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Stored chat message row used for websocket history endpoints.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatMessageRow {
    /// Chat message identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Display name shown for the sender.
    pub sender_name: String,
    /// Message body text.
    pub message: String,
    /// Message kind such as room chat, direct message, or system event.
    pub msg_type: String,
    /// Optional recipient identifier for direct messages.
    pub target_id: Option<String>,
    /// Optional sender identifier when the sender is a known attendee/admin.
    pub sender_id: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Anchor metadata for an inline comment thread in guide or step content.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct InlineCommentThread {
    /// Thread identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Stable client-side anchor key for the selected range.
    pub anchor_key: String,
    /// Anchor target kind: `step` or `guide`.
    pub target_type: String,
    /// Step identifier when the target is a specific step.
    pub target_step_id: Option<String>,
    /// Character offset of the highlighted range start.
    pub start_offset: i32,
    /// Character offset of the highlighted range end.
    pub end_offset: i32,
    /// Text that was originally selected when the thread was created.
    pub selected_text: String,
    /// Hash of the source content used to detect stale anchors.
    pub content_hash: String,
    /// Attendee who opened the thread.
    pub created_by_attendee_id: String,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Message stored inside an inline comment thread.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct InlineCommentMessage {
    /// Message identifier.
    pub id: String,
    /// Related thread identifier.
    pub thread_id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Author role, typically `attendee` or `admin`.
    pub author_role: String,
    /// Identifier of the author.
    pub author_id: String,
    /// Display name of the author.
    pub author_name: String,
    /// Message body text.
    pub message: String,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Inline comment thread plus the ordered messages it contains.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineCommentThreadWithMessages {
    #[serde(flatten)]
    /// Thread metadata.
    pub thread: InlineCommentThread,
    /// Ordered thread messages.
    pub messages: Vec<InlineCommentMessage>,
}

/// Payload used to open a new inline comment thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInlineCommentPayload {
    /// Stable client-side anchor key for the selected range.
    pub anchor_key: String,
    /// Anchor target kind: `step` or `guide`.
    pub target_type: String,
    /// Step identifier when the target is a specific step.
    pub target_step_id: Option<String>,
    /// Character offset of the highlighted range start.
    pub start_offset: i32,
    /// Character offset of the highlighted range end.
    pub end_offset: i32,
    /// Text that was selected in the UI.
    pub selected_text: String,
    /// Hash of the source content used to detect stale anchors.
    pub content_hash: String,
    /// First message to place in the new thread.
    pub message: String,
}

/// Payload used to reply to an existing inline comment thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyInlineCommentPayload {
    /// Reply body text.
    pub message: String,
    /// Hash of the source content used to detect stale anchors.
    pub content_hash: String,
}

/// Learner feedback payload accepted by the API.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedback {
    /// Learner-selected difficulty score.
    pub difficulty: String,
    /// Learner-selected satisfaction score.
    pub satisfaction: String,
    /// Optional free-form comment from the learner.
    pub comment: Option<String>,
}

/// Stored feedback record for a codelab.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Feedback {
    /// Feedback identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Attendee who submitted the feedback, when known.
    pub attendee_id: Option<String>,
    /// Learner-selected difficulty score.
    pub difficulty: String,
    /// Learner-selected satisfaction score.
    pub satisfaction: String,
    /// Optional free-form comment from the learner.
    pub comment: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Stored teaching material associated with a codelab.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Material {
    /// Material identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Material title shown in the UI.
    pub title: String,
    /// Material kind: `link` or `file`.
    pub material_type: String,
    /// Source URL when `material_type` is `link`.
    pub link_url: Option<String>,
    /// Uploaded file path when `material_type` is `file`.
    pub file_path: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Payload used to add material to a codelab.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMaterial {
    /// Material title shown in the UI.
    pub title: String,
    /// Material kind: `link` or `file`.
    pub material_type: String,
    /// Source URL when `material_type` is `link`.
    pub link_url: Option<String>,
    /// Uploaded file path when `material_type` is `file`.
    pub file_path: Option<String>,
}

/// Stored quiz submission for a specific attendee and quiz.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSubmission {
    /// Submission identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Attendee who submitted the answer.
    pub attendee_id: String,
    /// Related quiz identifier.
    pub quiz_id: String,
    /// Submitted answer payload.
    pub answer: String,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_correct: i32,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Single quiz answer submission payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuizSubmission {
    /// Related quiz identifier.
    pub quiz_id: String,
    /// Submitted answer payload.
    pub answer: String,
    /// Whether the submitted answer is correct.
    pub is_correct: bool,
}

/// Batch payload used when submitting multiple quiz answers at once.
#[derive(Debug, Serialize, Deserialize)]
pub struct QuizSubmissionPayload {
    /// Batch of quiz answers being submitted together.
    pub submissions: Vec<CreateQuizSubmission>,
}

/// Quiz submission joined with attendee metadata for admin views.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSubmissionWithAttendee {
    /// Submission identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Attendee who submitted the answer.
    pub attendee_id: String,
    /// Display name of the attendee.
    pub attendee_name: String,
    /// Related quiz identifier.
    pub quiz_id: String,
    /// Submitted answer payload.
    pub answer: String,
    /// Stored as `0`/`1` in the database, serialized as a boolean in the API.
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_correct: i32,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Stored learner submission, which may be a file or a link.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    /// Submission identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Attendee who submitted the work.
    pub attendee_id: String,
    /// Stored file path for file submissions.
    pub file_path: String,
    /// Original filename or link title.
    pub file_name: String,
    /// Stored file size in bytes.
    pub file_size: i64,
    /// Submission kind such as `file` or `link`.
    #[serde(default = "default_submission_type")]
    pub submission_type: String,
    /// External submission URL when the submission is link-based.
    pub link_url: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Submission joined with attendee metadata for admin-facing lists.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubmissionWithAttendee {
    /// Submission identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Attendee who submitted the work.
    pub attendee_id: String,
    /// Display name of the attendee.
    pub attendee_name: String,
    /// Stored file path for file submissions.
    pub file_path: String,
    /// Original filename or link title.
    pub file_name: String,
    /// Stored file size in bytes.
    pub file_size: i64,
    /// Submission kind such as `file` or `link`.
    #[serde(default = "default_submission_type")]
    pub submission_type: String,
    /// External submission URL when the submission is link-based.
    pub link_url: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Payload used when an attendee submits an external link instead of a file.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubmissionLink {
    /// External URL submitted by the learner.
    pub url: String,
    /// Optional display title for the link.
    pub title: Option<String>,
}

/// Default submission type used for older rows that predate the field.
pub fn default_submission_type() -> String {
    "file".to_string()
}

/// Stored AI conversation entry for the legacy conversation endpoint.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiConversation {
    /// Conversation identifier.
    pub id: String,
    /// Related codelab identifier.
    pub codelab_id: String,
    /// User who asked the question.
    pub user_id: String,
    /// User category such as `admin` or `attendee`.
    pub user_type: String,
    /// Display name of the user.
    pub user_name: String,
    /// Step number associated with the question, when known.
    pub step_number: Option<i32>,
    /// User question text.
    pub question: String,
    /// Model answer text.
    pub answer: String,
    /// Model name used to produce the answer.
    pub model: Option<String>,
    /// Provider-specific usage metadata serialized as JSON text.
    pub usage_metadata: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Payload used to persist an AI conversation exchange.
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveAiConversationPayload {
    /// Related codelab identifier.
    pub codelab_id: String,
    /// Step number associated with the exchange, when known.
    pub step_number: Option<i32>,
    /// User question text.
    pub question: String,
    /// Model answer text.
    pub answer: String,
    /// Model name used to produce the answer.
    pub model: Option<String>,
    #[serde(default)]
    /// Provider-specific usage metadata captured with the exchange.
    pub usage_metadata: Option<serde_json::Value>,
}

/// Stored AI thread metadata used by the threaded chat endpoints.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiThread {
    /// Thread identifier.
    pub id: String,
    /// Thread title shown in the UI.
    pub title: String,
    /// Owner of the thread.
    pub user_id: String,
    /// User category such as `admin` or `attendee`.
    pub user_type: String,
    /// Optional codelab scope for the thread.
    pub codelab_id: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
    /// Last update timestamp serialized as text.
    pub updated_at: Option<String>,
}

/// Stored message inside an AI thread.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiMessage {
    /// Message identifier.
    pub id: String,
    /// Related thread identifier.
    pub thread_id: String,
    /// Chat role such as `user`, `assistant`, or `system`.
    pub role: String,
    /// Message body text.
    pub content: String,
    /// Provider-specific grounding metadata serialized as JSON text.
    pub grounding_metadata: Option<String>,
    /// Provider-specific usage metadata serialized as JSON text.
    pub usage_metadata: Option<String>,
    /// Creation timestamp serialized as text.
    pub created_at: Option<String>,
}

/// Payload used to create a new AI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAiThreadPayload {
    /// Thread title shown in the UI.
    pub title: String,
    #[serde(default)]
    /// Optional codelab scope for the thread.
    pub codelab_id: Option<String>,
}

/// Payload used to append a message to an AI thread.
#[derive(Debug, Serialize, Deserialize)]
pub struct AddAiMessagePayload {
    /// Chat role such as `user`, `assistant`, or `system`.
    pub role: String,
    /// Message body text.
    pub content: String,
    #[serde(default)]
    /// Provider-specific grounding metadata captured for the message.
    pub grounding_metadata: Option<serde_json::Value>,
    #[serde(default)]
    /// Provider-specific usage metadata captured for the message.
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
