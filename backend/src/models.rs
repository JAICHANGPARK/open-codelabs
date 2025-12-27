use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Codelab {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: Option<String>,
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
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attendee {
    pub id: String,
    pub codelab_id: String,
    pub name: String,
    pub code: String,
    pub current_step: i32,
    pub created_at: Option<String>,
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
    pub attendee_id: String,
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
