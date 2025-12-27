use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Codelab {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    #[serde(serialize_with = "to_bool", deserialize_with = "from_bool")]
    pub is_public: i32,
    pub created_at: Option<String>,
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
