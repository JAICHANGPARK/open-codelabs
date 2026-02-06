use serde::Deserialize;

#[derive(Deserialize)]
pub struct SettingsPayload {
    pub gemini_api_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_payload_deserializes() {
        let raw = r#"{ "gemini_api_key": "abc" }"#;
        let payload: SettingsPayload = serde_json::from_str(raw).expect("deserialize");
        assert_eq!(payload.gemini_api_key, "abc");
    }
}
