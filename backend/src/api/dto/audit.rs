use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub codelab_id: Option<String>,
    pub action: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_query_deserializes() {
        let raw = r#"{ "limit": 10, "offset": 5, "action": "ai_query" }"#;
        let query: AuditLogQuery = serde_json::from_str(raw).expect("deserialize");
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(5));
        assert_eq!(query.action.as_deref(), Some("ai_query"));
    }
}
