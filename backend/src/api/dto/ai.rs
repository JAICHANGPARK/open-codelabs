use serde::Deserialize;

#[derive(Deserialize)]
pub struct AiRequest {
    pub prompt: Option<String>,
    pub contents: Option<serde_json::Value>,
    pub system_instruction: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub generation_config: Option<serde_json::Value>,
    pub tools: Option<serde_json::Value>,
    pub codelab_id: Option<String>,
    pub step_number: Option<i32>,
}
