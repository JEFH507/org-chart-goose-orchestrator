// Integration tests for Privacy Guard MCP

use privacy_guard_mcp::{Config, PiiCategory, PrivacyMode, Redactor, Tokenizer};
use std::collections::HashMap;
use tempfile::TempDir;

#[tokio::test]
async fn test_full_redaction_and_tokenization_flow() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::from_env().unwrap();
    config.mode = PrivacyMode::Rules;
    config.categories = vec![PiiCategory::Ssn, PiiCategory::Email, PiiCategory::Phone];
    config.token_storage_dir = temp_dir.path().to_str().unwrap().to_string();

    let redactor = Redactor::new(config.clone()).unwrap();
    let tokenizer = Tokenizer::new(config).unwrap();

    // Original prompt with PII
    let original = "Call John at 555-123-4567 or email john@example.com. SSN: 123-45-6789.";

    // Step 1: Redact PII
    let redacted = redactor.redact(original).await.unwrap();
    assert!(redacted.contains("[PHONE]"));
    assert!(redacted.contains("[EMAIL]"));
    assert!(redacted.contains("[SSN]"));
    assert!(!redacted.contains("555-123-4567"));
    assert!(!redacted.contains("john@example.com"));
    assert!(!redacted.contains("123-45-6789"));

    // Step 2: Tokenize redacted markers
    let (tokenized, token_map) = tokenizer.tokenize(&redacted).unwrap();
    assert_eq!(token_map.len(), 3); // 3 PII entities
    assert!(!tokenized.contains("[PHONE]")); // Replaced with token
    assert!(!tokenized.contains("[EMAIL]"));
    assert!(!tokenized.contains("[SSN]"));

    // Step 3: Store tokens
    let session_id = "test-session-123";
    tokenizer.store_tokens(session_id, &token_map).await.unwrap();

    // Step 4: Simulate LLM response (with tokens)
    // Find the token IDs
    let phone_token = token_map.iter().find(|(_, v)| v.contains("PHONE")).map(|(k, _)| k.clone()).unwrap();
    let email_token = token_map.iter().find(|(_, v)| v.contains("EMAIL")).map(|(k, _)| k.clone()).unwrap();
    
    let llm_response = format!("Contacted {} at {} successfully.", phone_token, email_token);

    // Step 5: Load tokens
    let loaded_tokens = tokenizer.load_tokens(session_id).await.unwrap();
    assert_eq!(loaded_tokens.len(), 3);

    // Step 6: Detokenize response
    let restored = tokenizer.detokenize(&llm_response, &loaded_tokens).unwrap();
    assert!(restored.contains("[PHONE]") || restored.contains("[EMAIL]"));

    // Step 7: Cleanup tokens
    tokenizer.delete_tokens(session_id).await.unwrap();
    let after_delete = tokenizer.load_tokens(session_id).await.unwrap();
    assert!(after_delete.is_empty());
}

#[tokio::test]
async fn test_hybrid_mode_graceful_degradation() {
    // Test that hybrid mode falls back to rules-only if Ollama unavailable
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::from_env().unwrap();
    config.mode = PrivacyMode::Hybrid;
    config.categories = vec![PiiCategory::Ssn, PiiCategory::Email];
    config.ollama_url = "http://localhost:99999".to_string(); // Invalid URL
    config.token_storage_dir = temp_dir.path().to_str().unwrap().to_string();

    let redactor = Redactor::new(config).unwrap();

    let input = "SSN: 123-45-6789, Email: test@example.com";
    let redacted = redactor.redact(input).await.unwrap();

    // Should still redact using rules (NER fails gracefully)
    assert!(redacted.contains("[SSN]"));
    assert!(redacted.contains("[EMAIL]"));
}

#[tokio::test]
async fn test_mode_off_passthrough() {
    let mut config = Config::from_env().unwrap();
    config.mode = PrivacyMode::Off;

    let redactor = Redactor::new(config).unwrap();

    let input = "SSN: 123-45-6789, Email: test@example.com";
    let output = redactor.redact(input).await.unwrap();

    // Should be unchanged
    assert_eq!(output, input);
}

#[test]
fn test_multiple_ssn_tokenization() {
    let config = Config::from_env().unwrap();
    let tokenizer = Tokenizer::new(config).unwrap();

    let redacted = "First [SSN], second [SSN], third [SSN].";
    let (tokenized, token_map) = tokenizer.tokenize(redacted).unwrap();

    // Should have 3 different tokens
    assert_eq!(token_map.len(), 3);
    
    // Each should be unique
    assert!(tokenized.contains("[SSN_0_"));
    assert!(tokenized.contains("[SSN_1_"));
    assert!(tokenized.contains("[SSN_2_"));
    
    // No generic [SSN] should remain
    assert!(!tokenized.contains("[SSN]"));
}

#[test]
fn test_tokenize_preserves_context() {
    let config = Config::from_env().unwrap();
    let tokenizer = Tokenizer::new(config).unwrap();

    let redacted = "Contact [EMAIL] about [PHONE] before [SSN] expires.";
    let (tokenized, token_map) = tokenizer.tokenize(redacted).unwrap();

    assert_eq!(token_map.len(), 3);
    
    // Context words should be preserved
    assert!(tokenized.contains("Contact"));
    assert!(tokenized.contains("about"));
    assert!(tokenized.contains("before"));
    assert!(tokenized.contains("expires"));
}

#[tokio::test]
async fn test_response_interceptor_with_audit() {
    use mockito::Server;
    use privacy_guard_mcp::{ResponseInterceptor};
    
    // Create mock server for Controller
    let mut server = Server::new_async().await;
    let mock = server.mock("POST", "/privacy/audit")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status":"ok"}"#)
        .create_async()
        .await;

    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::from_env().unwrap();
    config.controller_url = server.url();
    config.enable_audit_logs = true;
    config.token_storage_dir = temp_dir.path().to_str().unwrap().to_string();

    let tokenizer = Tokenizer::new(config.clone()).unwrap();
    let interceptor = ResponseInterceptor::new(config).unwrap();

    // Setup: Store tokens
    let session_id = "test-audit-session";
    let mut token_map = HashMap::new();
    token_map.insert("[SSN_0_ABC123]".to_string(), "[SSN]".to_string());
    token_map.insert("[EMAIL_0_XYZ456]".to_string(), "[EMAIL]".to_string());
    tokenizer.store_tokens(session_id, &token_map).await.unwrap();

    // Mock LLM response with tokens
    let llm_response = "Found [SSN_0_ABC123] and [EMAIL_0_XYZ456] in records.";

    // Intercept response (should detokenize and send audit log)
    let result = interceptor.intercept(session_id, llm_response).await.unwrap();

    // Verify detokenization
    assert!(result.contains("[SSN]"));
    assert!(result.contains("[EMAIL]"));
    assert!(!result.contains("ABC123"));
    assert!(!result.contains("XYZ456"));

    // Verify audit log was sent
    mock.assert_async().await;
}

#[tokio::test]
async fn test_audit_log_disabled() {
    use privacy_guard_mcp::{ResponseInterceptor};
    
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::from_env().unwrap();
    config.enable_audit_logs = false; // Disabled
    config.token_storage_dir = temp_dir.path().to_str().unwrap().to_string();

    let tokenizer = Tokenizer::new(config.clone()).unwrap();
    let interceptor = ResponseInterceptor::new(config).unwrap();

    // Setup tokens
    let session_id = "test-no-audit";
    let mut token_map = HashMap::new();
    token_map.insert("[SSN_0_TEST]".to_string(), "[SSN]".to_string());
    tokenizer.store_tokens(session_id, &token_map).await.unwrap();

    // Should not send audit log (no error)
    let result = interceptor.intercept(session_id, "Test [SSN_0_TEST]").await;
    assert!(result.is_ok());
}
