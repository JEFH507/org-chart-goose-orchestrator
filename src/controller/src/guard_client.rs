// Privacy Guard HTTP Client
//
// Calls privacy-guard service to mask PII in audit event content.
// Used when GUARD_ENABLED=true in controller configuration.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, warn, debug};

#[derive(Clone)]
pub struct GuardClient {
    base_url: String,
    client: reqwest::Client,
    enabled: bool,
}

#[derive(Serialize)]
struct MaskRequest<'a> {
    text: &'a str,
    tenant_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
pub struct MaskResponse {
    pub masked_text: String,
    pub redactions: std::collections::HashMap<String, usize>,
    #[serde(default)]
    pub session_id: Option<String>,
}

impl GuardClient {
    /// Create a new guard client from environment variables
    pub fn from_env() -> Self {
        let enabled = std::env::var("GUARD_ENABLED")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);

        let base_url = std::env::var("GUARD_URL")
            .unwrap_or_else(|_| "http://privacy-guard:8089".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        debug!(
            message = "guard_client initialized",
            enabled = enabled,
            base_url = %base_url
        );

        Self {
            base_url,
            client,
            enabled,
        }
    }

    /// Check if guard is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Mask PII in text content
    ///
    /// Returns Ok(Some(response)) if successful
    /// Returns Ok(None) if guard is disabled or fails (fail-open mode)
    /// Returns Err only if fail-closed mode (not implemented yet)
    pub async fn mask_text(
        &self,
        text: &str,
        tenant_id: &str,
        session_id: Option<&str>,
    ) -> Result<Option<MaskResponse>, GuardError> {
        if !self.enabled {
            debug!("guard disabled, skipping mask operation");
            return Ok(None);
        }

        let url = format!("{}/guard/mask", self.base_url);
        let request = MaskRequest {
            text,
            tenant_id,
            session_id,
        };

        match self.client.post(&url).json(&request).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<MaskResponse>().await {
                    Ok(mask_response) => {
                        debug!(
                            message = "guard mask successful",
                            redactions = ?mask_response.redactions,
                            session_id = ?mask_response.session_id
                        );
                        Ok(Some(mask_response))
                    }
                    Err(e) => {
                        warn!(
                            message = "guard response parse error (fail-open)",
                            error = %e
                        );
                        Ok(None) // Fail-open: return original text
                    }
                }
            }
            Ok(response) => {
                warn!(
                    message = "guard returned error status (fail-open)",
                    status = response.status().as_u16()
                );
                Ok(None) // Fail-open
            }
            Err(e) => {
                warn!(
                    message = "guard request failed (fail-open)",
                    error = %e
                );
                Ok(None) // Fail-open: guard unavailable
            }
        }
    }

    /// Mask PII in JSON values
    ///
    /// Serializes JSON to string, masks it, and parses back.
    /// Simple approach for Phase 3 - avoids async recursion complexity.
    /// Returns Ok(Some(masked_json)) if successful
    /// Returns Ok(None) if guard is disabled or fails (fail-open mode)
    pub async fn mask_json(
        &self,
        value: &serde_json::Value,
        tenant_id: &str,
        session_id: Option<&str>,
    ) -> Result<Option<serde_json::Value>, GuardError> {
        if !self.enabled {
            debug!("guard disabled, skipping mask operation");
            return Ok(None);
        }

        // Convert JSON to string
        let json_str = serde_json::to_string(value)
            .map_err(|e| GuardError::ParseError(format!("Failed to serialize JSON: {}", e)))?;

        // Mask the string representation
        match self.mask_text(&json_str, tenant_id, session_id).await? {
            Some(response) => {
                // Parse masked string back to JSON
                match serde_json::from_str(&response.masked_text) {
                    Ok(masked_value) => Ok(Some(masked_value)),
                    Err(_) => {
                        // If parsing fails, guard may have broken JSON structure
                        // Return original value (fail-open)
                        warn!(
                            message = "guard broke JSON structure (fail-open)",
                            tenant_id = tenant_id
                        );
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Health check - verify guard is reachable
    pub async fn health_check(&self) -> bool {
        if !self.enabled {
            return true; // Not enabled = considered healthy
        }

        let url = format!("{}/status", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(e) => {
                error!(message = "guard health check failed", error = %e);
                false
            }
        }
    }
}

#[derive(Debug)]
pub enum GuardError {
    RequestFailed(String),
    ParseError(String),
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardError::RequestFailed(msg) => write!(f, "Guard request failed: {}", msg),
            GuardError::ParseError(msg) => write!(f, "Guard response parse error: {}", msg),
        }
    }
}

impl std::error::Error for GuardError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_client_from_env_disabled() {
        std::env::remove_var("GUARD_ENABLED");
        let client = GuardClient::from_env();
        assert!(!client.is_enabled());
    }

    #[test]
    fn test_guard_client_from_env_enabled() {
        std::env::set_var("GUARD_ENABLED", "true");
        std::env::set_var("GUARD_URL", "http://test:9999");
        let client = GuardClient::from_env();
        assert!(client.is_enabled());
        assert_eq!(client.base_url, "http://test:9999");
        std::env::remove_var("GUARD_ENABLED");
        std::env::remove_var("GUARD_URL");
    }

    #[test]
    fn test_mask_text_when_disabled() {
        std::env::remove_var("GUARD_ENABLED");
        let client = GuardClient::from_env();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(client.mask_text("test", "org1", None));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should return None when disabled
    }
}
