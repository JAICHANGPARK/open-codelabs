use serde::Deserialize;

/// Request payload forwarded to the AI streaming proxy endpoint.
#[derive(Deserialize)]
pub struct AiRequest {
    /// Plain-text prompt used by simpler client flows.
    pub prompt: Option<String>,
    /// Full provider-native request body when the client sends structured contents.
    pub contents: Option<serde_json::Value>,
    /// Optional system instruction prepended to the model request.
    pub system_instruction: Option<String>,
    /// Per-request API key override supplied by the caller.
    pub api_key: Option<String>,
    /// Requested model name.
    pub model: Option<String>,
    /// Provider-specific generation configuration JSON.
    pub generation_config: Option<serde_json::Value>,
    /// Provider-specific tool declarations JSON.
    pub tools: Option<serde_json::Value>,
    /// Related codelab id for audit logging and persistence.
    pub codelab_id: Option<String>,
    /// Related step number for audit logging and persistence.
    pub step_number: Option<i32>,
}
