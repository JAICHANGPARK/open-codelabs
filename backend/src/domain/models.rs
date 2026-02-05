use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct Codelab {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_public: i32,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub quiz_enabled: i32,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub require_quiz: i32,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub require_feedback: i32,
    pub guide_markdown: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Quiz {
    pub id: String,
    pub codelab_id: String,
    pub question: String,
    pub quiz_type: Option<String>,
    pub options: String, // JSON string
    pub correct_answer: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuiz {
    pub question: String,
    pub quiz_type: Option<String>,
    pub options: Vec<String>,
    pub correct_answer: i32,
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Step {
    pub id: String,
    pub codelab_id: String,
    pub step_number: i32,
    pub title: String,
    pub content_markdown: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCodelab {
    pub title: String,
    pub description: String,
    pub author: String,
    pub is_public: Option<bool>,
    pub quiz_enabled: Option<bool>,
    pub require_quiz: Option<bool>,
    pub require_feedback: Option<bool>,
    pub guide_markdown: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub admin_id: String,
    pub admin_pw: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStepsPayload {
    pub steps: Vec<CreateStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStep {
    pub title: String,
    pub content_markdown: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationPayload {
    pub name: String,
    pub code: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attendee {
    pub id: String,
    pub codelab_id: String,
    pub name: String,
    pub code: String,
    pub email: Option<String>,
    pub current_step: i32,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_completed: i32,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttendeePublic {
    pub id: String,
    pub codelab_id: String,
    pub name: String,
    pub email: Option<String>,
    pub current_step: i32,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_completed: i32,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub attendee_name: String,
    pub codelab_title: String,
    pub codelab_id: String,
    pub author: String,
    pub completed_at: String,
    pub verification_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelpRequestPayload {
    pub step_number: i32,
}

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatMessageRow {
    pub id: String,
    pub codelab_id: String,
    pub sender_name: String,
    pub message: String,
    pub msg_type: String,
    pub target_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedback {
    pub difficulty: String,
    pub satisfaction: String,
    pub comment: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Material {
    pub id: String,
    pub codelab_id: String,
    pub title: String,
    pub material_type: String, // "link" or "file"
    pub link_url: Option<String>,
    pub file_path: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMaterial {
    pub title: String,
    pub material_type: String,
    pub link_url: Option<String>,
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSubmission {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub quiz_id: String,
    pub answer: String,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_correct: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuizSubmission {
    pub quiz_id: String,
    pub answer: String,
    pub is_correct: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizSubmissionPayload {
    pub submissions: Vec<CreateQuizSubmission>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSubmissionWithAttendee {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub attendee_name: String,
    pub quiz_id: String,
    pub answer: String,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_correct: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubmissionWithAttendee {
    pub id: String,
    pub codelab_id: String,
    pub attendee_id: String,
    pub attendee_name: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiConversation {
    pub id: String,
    pub codelab_id: String,
    pub user_id: String,
    pub user_type: String,
    pub user_name: String,
    pub step_number: Option<i32>,
    pub question: String,
    pub answer: String,
    pub model: Option<String>,
    pub usage_metadata: Option<String>,
    pub created_at: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiThread {
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub user_type: String,
    pub codelab_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiMessage {
    pub id: String,
    pub thread_id: String,
    pub role: String,
    pub content: String,
    pub grounding_metadata: Option<String>,
    pub usage_metadata: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAiThreadPayload {
    pub title: String,
    #[serde(default)]
    pub codelab_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAiMessagePayload {
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
}
