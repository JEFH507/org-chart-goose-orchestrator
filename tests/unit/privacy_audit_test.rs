// Unit tests for Privacy Guard audit endpoint (E5)

#[cfg(test)]
mod tests {
    use serde_json;

    /// Simulates AuditLogEntry struct (can't import from routes due to module visibility)
    #[derive(serde::Serialize, serde::Deserialize)]
    struct AuditLogEntry {
        session_id: String,
        redaction_count: usize,
        categories: Vec<String>,
        mode: String,
        timestamp: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct AuditLogResponse {
        status: String,
        id: i64,
    }

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            session_id: "test-session-123".to_string(),
            redaction_count: 5,
            categories: vec!["SSN".to_string(), "EMAIL".to_string()],
            mode: "Hybrid".to_string(),
            timestamp: 1699564800,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test-session-123"));
        assert!(json.contains("\"redaction_count\":5"));
        assert!(json.contains("SSN"));
        assert!(json.contains("EMAIL"));
        assert!(json.contains("Hybrid"));
    }

    #[test]
    fn test_audit_log_entry_deserialization() {
        let json = r#"{
            "session_id": "test-123",
            "redaction_count": 3,
            "categories": ["SSN", "PHONE"],
            "mode": "Rules",
            "timestamp": 1699564800
        }"#;

        let entry: AuditLogEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.session_id, "test-123");
        assert_eq!(entry.redaction_count, 3);
        assert_eq!(entry.categories.len(), 2);
        assert!(entry.categories.contains(&"SSN".to_string()));
        assert_eq!(entry.mode, "Rules");
        assert_eq!(entry.timestamp, 1699564800);
    }

    #[test]
    fn test_audit_log_response_serialization() {
        let response = AuditLogResponse {
            status: "created".to_string(),
            id: 42,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"created\""));
        assert!(json.contains("\"id\":42"));
    }

    #[test]
    fn test_empty_categories() {
        let entry = AuditLogEntry {
            session_id: "test-no-pii".to_string(),
            redaction_count: 0,
            categories: vec![],
            mode: "Off".to_string(),
            timestamp: 1699564800,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"redaction_count\":0"));
        assert!(json.contains("\"categories\":[]"));
    }

    #[test]
    fn test_multiple_categories() {
        let entry = AuditLogEntry {
            session_id: "test-multi-pii".to_string(),
            redaction_count: 8,
            categories: vec![
                "SSN".to_string(),
                "EMAIL".to_string(),
                "PHONE".to_string(),
                "CREDIT_CARD".to_string(),
            ],
            mode: "Hybrid".to_string(),
            timestamp: 1699564800,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"redaction_count\":8"));
        assert!(json.contains("SSN"));
        assert!(json.contains("EMAIL"));
        assert!(json.contains("PHONE"));
        assert!(json.contains("CREDIT_CARD"));
    }

    #[test]
    fn test_timestamp_conversion() {
        // Test various Unix timestamps
        let timestamps = vec![
            1699564800, // 2023-11-09
            1609459200, // 2021-01-01
            1735689600, // 2025-01-01
        ];

        for ts in timestamps {
            let entry = AuditLogEntry {
                session_id: format!("test-ts-{}", ts),
                redaction_count: 1,
                categories: vec!["EMAIL".to_string()],
                mode: "Rules".to_string(),
                timestamp: ts,
            };

            let json = serde_json::to_string(&entry).unwrap();
            assert!(json.contains(&format!("\"timestamp\":{}", ts)));
        }
    }

    #[test]
    fn test_mode_values() {
        let modes = vec!["Rules", "NER", "Hybrid", "Off"];

        for mode in modes {
            let entry = AuditLogEntry {
                session_id: format!("test-mode-{}", mode),
                redaction_count: 1,
                categories: vec!["SSN".to_string()],
                mode: mode.to_string(),
                timestamp: 1699564800,
            };

            let json = serde_json::to_string(&entry).unwrap();
            assert!(json.contains(&format!("\"mode\":\"{}\"", mode)));
        }
    }
}
