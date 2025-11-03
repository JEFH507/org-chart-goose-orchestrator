// Audit logging for redaction events (counts only, no PII)
// This module implements structured logging for audit trail

use serde::Serialize;
use std::collections::HashMap;
use tracing::info;

use crate::detection::EntityType;
use crate::policy::GuardMode;

/// Audit event for redaction operations
/// CRITICAL: This struct must NEVER contain raw PII or pseudonym mappings
/// Only counts, metadata, and performance metrics are allowed
#[derive(Debug, Serialize)]
pub struct RedactionEvent {
    /// ISO 8601 timestamp
    pub timestamp: String,
    
    /// Tenant identifier
    pub tenant_id: String,
    
    /// Session identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    
    /// Guard mode used for this operation
    pub mode: String,
    
    /// Counts of each entity type detected and masked
    /// Map: EntityType -> count
    pub entity_counts: HashMap<String, usize>,
    
    /// Total number of redactions performed
    pub total_redactions: usize,
    
    /// Performance metric: processing time in milliseconds
    pub performance_ms: u64,
    
    /// Trace ID for distributed tracing (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// Log a redaction event to the audit trail
/// 
/// This function emits a structured JSON log entry with:
/// - Timestamp
/// - Tenant and session identifiers
/// - Guard mode
/// - Entity counts (NO raw PII or pseudonyms)
/// - Performance metrics
/// - Trace ID (if available)
///
/// # Safety
/// This function MUST NOT log any raw PII or pseudonym mappings
/// Only counts and metadata are allowed
pub fn log_redaction_event(
    tenant_id: &str,
    session_id: Option<&str>,
    mode: GuardMode,
    redactions: &HashMap<String, usize>,
    duration_ms: u64,
) {
    // Calculate total redactions
    let total = redactions.values().sum();
    
    // Create audit event
    let event = RedactionEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        tenant_id: tenant_id.to_string(),
        session_id: session_id.map(|s| s.to_string()),
        mode: format!("{}", mode),
        entity_counts: redactions.clone(),
        total_redactions: total,
        performance_ms: duration_ms,
        trace_id: extract_trace_id(), // TODO: Extract from request headers
    };
    
    // Emit structured log with "audit" target
    info!(
        target: "audit",
        event = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string()),
        "Redaction event"
    );
}

/// Extract trace ID from current context
/// TODO: Implement proper trace ID extraction from request headers
/// This is a placeholder for OTLP integration in future phases
fn extract_trace_id() -> Option<String> {
    // Placeholder: In production, this would extract from:
    // - X-Trace-Id header
    // - W3C Trace Context header
    // - OpenTelemetry context
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::EntityType;
    use crate::policy::Mode;

    #[test]
    fn test_redaction_event_serialization() {
        let mut redactions = HashMap::new();
        redactions.insert(EntityType::EMAIL, 2);
        redactions.insert(EntityType::PHONE, 1);
        
        let event = RedactionEvent {
            timestamp: "2025-11-03T12:00:00Z".to_string(),
            tenant_id: "test-org".to_string(),
            session_id: Some("sess_abc123".to_string()),
            mode: "MASK".to_string(),
            entity_counts: redactions
                .iter()
                .map(|(k, v)| (format!("{:?}", k), *v))
                .collect(),
            total_redactions: 3,
            performance_ms: 45,
            trace_id: Some("trace-xyz".to_string()),
        };
        
        let json = serde_json::to_string(&event).expect("Should serialize");
        
        // Verify JSON contains expected fields
        assert!(json.contains("timestamp"));
        assert!(json.contains("test-org"));
        assert!(json.contains("sess_abc123"));
        assert!(json.contains("MASK"));
        assert!(json.contains("EMAIL"));
        assert!(json.contains("PHONE"));
        assert!(json.contains("\"total_redactions\":3"));
        assert!(json.contains("\"performance_ms\":45"));
    }
    
    #[test]
    fn test_redaction_event_no_pii() {
        let mut redactions = HashMap::new();
        redactions.insert(EntityType::EMAIL, 1);
        
        let event = RedactionEvent {
            timestamp: "2025-11-03T12:00:00Z".to_string(),
            tenant_id: "test-org".to_string(),
            session_id: None,
            mode: "MASK".to_string(),
            entity_counts: HashMap::from([("EMAIL".to_string(), 1)]),
            total_redactions: 1,
            performance_ms: 30,
            trace_id: None,
        };
        
        let json = serde_json::to_string(&event).expect("Should serialize");
        
        // Verify NO raw PII in JSON (these should NOT appear)
        assert!(!json.contains("alice@example.com"));
        assert!(!json.contains("john.doe"));
        assert!(!json.contains("555-1234"));
        assert!(!json.contains("123-45-6789"));
        
        // Verify only counts present
        assert!(json.contains("\"total_redactions\":1"));
        assert!(json.contains("EMAIL"));
    }
    
    #[test]
    fn test_log_redaction_event_no_panic() {
        // This test verifies that log_redaction_event doesn't panic
        // Actual log output would be to tracing subscriber in tests
        let mut redactions = HashMap::new();
        redactions.insert(EntityType::PERSON, 2);
        redactions.insert(EntityType::SSN, 1);
        
        log_redaction_event(
            "test-org",
            Some("sess_test"),
            &Mode::MASK,
            &redactions,
            100,
        );
        
        // If we get here without panic, the test passes
    }
    
    #[test]
    fn test_empty_redactions() {
        let redactions = HashMap::new();
        
        log_redaction_event(
            "test-org",
            None,
            &Mode::DETECT,
            &redactions,
            20,
        );
        
        // Should handle empty redactions gracefully
    }
    
    #[test]
    fn test_all_entity_types() {
        let mut redactions = HashMap::new();
        redactions.insert(EntityType::SSN, 1);
        redactions.insert(EntityType::EMAIL, 2);
        redactions.insert(EntityType::PHONE, 3);
        redactions.insert(EntityType::CREDIT_CARD, 1);
        redactions.insert(EntityType::PERSON, 4);
        redactions.insert(EntityType::IP_ADDRESS, 1);
        redactions.insert(EntityType::DATE_OF_BIRTH, 1);
        redactions.insert(EntityType::ACCOUNT_NUMBER, 1);
        
        let entity_counts: HashMap<String, usize> = redactions
            .iter()
            .map(|(k, v)| (format!("{:?}", k), *v))
            .collect();
        
        let total: usize = redactions.values().sum();
        assert_eq!(total, 14);
        assert_eq!(entity_counts.len(), 8);
    }
    
    #[test]
    fn test_event_optional_fields() {
        // Test with minimal fields (session_id and trace_id omitted)
        let event = RedactionEvent {
            timestamp: "2025-11-03T12:00:00Z".to_string(),
            tenant_id: "test-org".to_string(),
            session_id: None,
            mode: "OFF".to_string(),
            entity_counts: HashMap::new(),
            total_redactions: 0,
            performance_ms: 10,
            trace_id: None,
        };
        
        let json = serde_json::to_string(&event).expect("Should serialize");
        
        // Optional fields should not appear when None
        assert!(!json.contains("session_id"));
        assert!(!json.contains("trace_id"));
        
        // Required fields should appear
        assert!(json.contains("timestamp"));
        assert!(json.contains("tenant_id"));
    }
    
    #[test]
    fn test_performance_metrics() {
        let mut redactions = HashMap::new();
        redactions.insert(EntityType::EMAIL, 1);
        
        // Test with various performance values
        for duration_ms in &[10, 50, 100, 500, 1000, 2000] {
            log_redaction_event(
                "test-org",
                Some("sess_perf"),
                &Mode::MASK,
                &redactions,
                *duration_ms,
            );
            // Should log without panic for any duration
        }
    }
    
    #[test]
    fn test_different_modes() {
        let redactions = HashMap::new();
        
        // Test all modes
        for mode in &[Mode::OFF, Mode::DETECT, Mode::MASK, Mode::STRICT] {
            log_redaction_event(
                "test-org",
                None,
                mode,
                &redactions,
                25,
            );
        }
    }
}
