// Integration tests for Privacy Guard HTTP API
// These tests verify the complete API behavior end-to-end

#[cfg(test)]
mod integration {
    use serde_json::json;

    // Helper to create test client (when guard server is running)
    // For local development: cargo test --test integration_tests -- --ignored
    
    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_status_endpoint_integration() {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:8089/status")
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], "healthy");
        assert!(body["config_loaded"].as_bool().unwrap());
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_scan_detects_email() {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8089/guard/scan")
            .json(&json!({
                "text": "Contact john@example.com for details",
                "tenant_id": "test-org"
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        let detections = body["detections"].as_array().expect("detections should be array");
        
        assert!(detections.len() > 0, "Should detect at least one entity");
        
        // Check for EMAIL detection
        let has_email = detections.iter().any(|d| {
            d["entity_type"].as_str() == Some("EMAIL")
        });
        assert!(has_email, "Should detect EMAIL entity");
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_mask_produces_pseudonyms() {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8089/guard/mask")
            .json(&json!({
                "text": "Contact alice@example.com",
                "tenant_id": "test-org"
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        
        // Check masked_text doesn't contain original
        let masked_text = body["masked_text"].as_str().expect("masked_text should be string");
        assert!(!masked_text.contains("alice@example.com"), "Original email should be masked");
        
        // Check redactions count
        let redactions = body["redactions"].as_object().expect("redactions should be object");
        assert!(redactions.contains_key("EMAIL"), "Should have EMAIL redaction");
        
        // Check session_id present
        assert!(body["session_id"].is_string(), "Should return session_id");
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_mask_determinism() {
        let client = reqwest::Client::new();
        
        let request_body = json!({
            "text": "Email: test@example.com",
            "tenant_id": "test-org"
        });

        // First request
        let response1 = client
            .post("http://localhost:8089/guard/mask")
            .json(&request_body)
            .send()
            .await
            .expect("Failed to send request 1");
        
        let body1: serde_json::Value = response1.json().await.expect("Failed to parse JSON 1");

        // Second request (same input, same tenant)
        let response2 = client
            .post("http://localhost:8089/guard/mask")
            .json(&request_body)
            .send()
            .await
            .expect("Failed to send request 2");
        
        let body2: serde_json::Value = response2.json().await.expect("Failed to parse JSON 2");

        // Masked text should be identical (determinism)
        assert_eq!(
            body1["masked_text"], 
            body2["masked_text"],
            "Same input should produce same pseudonyms"
        );
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_mask_phone_fpe_preserves_format() {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8089/guard/mask")
            .json(&json!({
                "text": "Call me at 555-123-4567",
                "tenant_id": "test-org"
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        let masked_text = body["masked_text"].as_str().expect("masked_text should be string");
        
        // Should preserve phone format (XXX-XXX-XXXX)
        assert!(
            masked_text.contains("-"),
            "Phone format should be preserved with hyphens"
        );
        
        // Original number should not be present
        assert!(!masked_text.contains("555-123-4567"), "Original phone should be masked");
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_reidentify_requires_auth() {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8089/guard/reidentify")
            .json(&json!({
                "pseudonym": "EMAIL_abc123",
                "session_id": "sess_test"
            }))
            .send()
            .await
            .expect("Failed to send request");

        // Without Authorization header, should return 401
        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_flush_session() {
        let client = reqwest::Client::new();
        
        // First, create a session by masking
        let mask_response = client
            .post("http://localhost:8089/guard/mask")
            .json(&json!({
                "text": "Email: test@example.com",
                "tenant_id": "test-org",
                "session_id": "sess_test_flush"
            }))
            .send()
            .await
            .expect("Failed to mask");
        
        assert_eq!(mask_response.status(), 200);

        // Now flush the session
        let flush_response = client
            .post("http://localhost:8089/internal/flush-session")
            .json(&json!({
                "session_id": "sess_test_flush"
            }))
            .send()
            .await
            .expect("Failed to flush");

        assert_eq!(flush_response.status(), 200);
        
        let body: serde_json::Value = flush_response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], "flushed");
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_invalid_tenant_id() {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8089/guard/mask")
            .json(&json!({
                "text": "Some text",
                "tenant_id": ""
            }))
            .send()
            .await
            .expect("Failed to send request");

        // Empty tenant_id should return 400
        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_multiple_entity_types() {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8089/guard/mask")
            .json(&json!({
                "text": "Contact John Doe at 555-123-4567 or john@example.com. SSN: 123-45-6789",
                "tenant_id": "test-org"
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        let redactions = body["redactions"].as_object().expect("redactions should be object");
        
        // Should have multiple entity types
        assert!(redactions.len() >= 2, "Should detect multiple entity types");
    }

    #[tokio::test]
    #[ignore] // Requires running server  
    async fn test_no_pii_passthrough() {
        let client = reqwest::Client::new();
        let original_text = "The quick brown fox jumps over the lazy dog";
        
        let response = client
            .post("http://localhost:8089/guard/mask")
            .json(&json!({
                "text": original_text,
                "tenant_id": "test-org"
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        
        // Text with no PII should pass through unchanged
        assert_eq!(body["masked_text"].as_str(), Some(original_text));
        
        // Should have no redactions
        let redactions = body["redactions"].as_object().expect("redactions should be object");
        assert_eq!(redactions.len(), 0, "Should have no redactions for clean text");
    }
}
