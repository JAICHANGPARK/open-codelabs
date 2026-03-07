use serde::Deserialize;

/// Query parameters used to filter and page audit log results.
#[derive(Deserialize)]
pub struct AuditLogQuery {
    /// Maximum number of rows to return.
    pub limit: Option<i32>,
    /// Number of rows to skip for pagination.
    pub offset: Option<i32>,
    /// Optional codelab id filter.
    pub codelab_id: Option<String>,
    /// Optional action name filter.
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
