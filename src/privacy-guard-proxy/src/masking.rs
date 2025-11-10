// masking.rs - PII masking/unmasking integration with Privacy Guard service

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Masking context - stores PII mappings for a single request
#[derive(Debug, Clone)]
pub struct MaskingContext {
    /// Maps masked tokens to original PII (for unmasking)
    pub mappings: HashMap<String, String>,
}

impl MaskingContext {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, masked: String, original: String) {
        self.mappings.insert(masked, original);
    }

    pub fn get_original(&self, masked: &str) -> Option<&String> {
        self.mappings.get(masked)
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

/// Request to Privacy Guard /guard/mask endpoint
#[derive(Debug, Serialize)]
struct MaskRequest {
    tenant_id: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
}

/// Response from Privacy Guard /guard/mask endpoint
#[derive(Debug, Deserialize)]
struct MaskResponse {
    masked_text: String,
    session_id: String,
    #[serde(default)]
    redactions: HashMap<String, u32>,
}

/// Request to Privacy Guard /guard/reidentify endpoint
#[derive(Debug, Serialize)]
struct ReidentifyRequest {
    tenant_id: String,
    masked_text: String,
    session_id: String,
}

/// Response from Privacy Guard /guard/reidentify endpoint
#[derive(Debug, Deserialize)]
struct ReidentifyResponse {
    original_text: String,
}

/// Mask a message using Privacy Guard service
///
/// Returns (masked_text, session_id) where session_id is used for reidentification
pub async fn mask_message(
    privacy_guard_url: &str,
    message: &str,
    tenant_id: &str,
    client: &Client,
) -> Result<(String, String), String> {
    let request = MaskRequest {
        tenant_id: tenant_id.to_string(),
        text: message.to_string(),
        session_id: None,
    };

    let url = format!("{}/guard/mask", privacy_guard_url);
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to call Privacy Guard /guard/mask: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Privacy Guard /guard/mask failed: {} - {}", status, body));
    }

    let mask_response: MaskResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse /guard/mask response: {}", e))?;

    Ok((mask_response.masked_text, mask_response.session_id))
}

/// Unmask a response using Privacy Guard service
///
/// Returns unmasked_text
pub async fn unmask_response(
    privacy_guard_url: &str,
    masked_text: &str,
    tenant_id: &str,
    session_id: &str,
    client: &Client,
) -> Result<String, String> {
    let request = ReidentifyRequest {
        tenant_id: tenant_id.to_string(),
        masked_text: masked_text.to_string(),
        session_id: session_id.to_string(),
    };

    let url = format!("{}/guard/reidentify", privacy_guard_url);
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to call Privacy Guard /guard/reidentify: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Privacy Guard /guard/reidentify failed: {} - {}", status, body));
    }

    let reidentify_response: ReidentifyResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse /guard/reidentify response: {}", e))?;

    Ok(reidentify_response.original_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masking_context_new() {
        let ctx = MaskingContext::new();
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_masking_context_add_mapping() {
        let mut ctx = MaskingContext::new();
        ctx.add_mapping("SSN_001".to_string(), "123-45-6789".to_string());
        
        assert!(!ctx.is_empty());
        assert_eq!(ctx.get_original("SSN_001"), Some(&"123-45-6789".to_string()));
    }

    #[test]
    fn test_masking_context_get_original() {
        let mut ctx = MaskingContext::new();
        ctx.add_mapping("EMAIL_001".to_string(), "john@example.com".to_string());
        
        assert_eq!(ctx.get_original("EMAIL_001"), Some(&"john@example.com".to_string()));
        assert_eq!(ctx.get_original("EMAIL_002"), None);
    }

    #[test]
    fn test_masking_context_multiple_mappings() {
        let mut ctx = MaskingContext::new();
        ctx.add_mapping("SSN_001".to_string(), "123-45-6789".to_string());
        ctx.add_mapping("EMAIL_001".to_string(), "john@example.com".to_string());
        ctx.add_mapping("PHONE_001".to_string(), "+1-555-1234".to_string());
        
        assert!(!ctx.is_empty());
        assert_eq!(ctx.mappings.len(), 3);
        assert_eq!(ctx.get_original("SSN_001"), Some(&"123-45-6789".to_string()));
        assert_eq!(ctx.get_original("EMAIL_001"), Some(&"john@example.com".to_string()));
        assert_eq!(ctx.get_original("PHONE_001"), Some(&"+1-555-1234".to_string()));
    }

    // Note: Integration tests for mask_message() and unmask_response() 
    // require a running Privacy Guard service and should be in tests/integration/
}
