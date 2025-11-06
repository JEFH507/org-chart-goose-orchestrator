// Request/Response interceptors for Privacy Guard MCP

use anyhow::Result;
use serde_json::Value;
use tracing::{info, warn};

use crate::config::Config;
use crate::redaction::Redactor;
use crate::tokenizer::Tokenizer;

/// Request interceptor - applies privacy protection before sending to LLM
pub struct RequestInterceptor {
    config: Config,
    redactor: Redactor,
    tokenizer: Tokenizer,
}

impl RequestInterceptor {
    /// Create new request interceptor
    pub fn new(config: Config) -> Result<Self> {
        let redactor = Redactor::new(config.clone())?;
        let tokenizer = Tokenizer::new(config.clone())?;

        Ok(Self {
            config,
            redactor,
            tokenizer,
        })
    }

    /// Intercept and protect request
    pub async fn intercept(&self, params: Value) -> Result<Value> {
        // Extract prompt and session_id from params
        let prompt = params
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'prompt' in params"))?;

        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'session_id' in params"))?;

        info!("Intercepting request for session {}", session_id);

        // Apply redaction (Phase E2 - to be implemented)
        let redacted = self.redactor.redact(prompt).await?;

        // Tokenize PII (Phase E2 - to be implemented)
        let (tokenized, token_map) = self.tokenizer.tokenize(&redacted)?;

        // Store tokens for later detokenization (Phase E4 - to be implemented)
        self.tokenizer.store_tokens(session_id, &token_map).await?;

        // Return protected params
        let mut protected_params = params.clone();
        protected_params["prompt"] = Value::String(tokenized);

        info!("Privacy protection applied: {} PII entities redacted", token_map.len());

        Ok(protected_params)
    }
}

/// Response interceptor - detokenizes response and sends audit log
pub struct ResponseInterceptor {
    config: Config,
    tokenizer: Tokenizer,
}

impl ResponseInterceptor {
    /// Create new response interceptor
    pub fn new(config: Config) -> Result<Self> {
        let tokenizer = Tokenizer::new(config.clone())?;

        Ok(Self {
            config,
            tokenizer,
        })
    }

    /// Intercept and restore response
    pub async fn intercept(&self, session_id: &str, response: &str) -> Result<String> {
        info!("Intercepting response for session {}", session_id);

        // Load tokens (Phase E4 - to be implemented)
        let token_map = self.tokenizer.load_tokens(session_id).await?;

        // Detokenize response (Phase E3 - to be implemented)
        let restored = self.tokenizer.detokenize(response, &token_map)?;

        // Send audit log to Controller (Phase E5 - to be implemented)
        self.send_audit_log(session_id, &token_map).await?;

        // Clean up tokens (Phase E4 - to be implemented)
        self.tokenizer.delete_tokens(session_id).await?;

        info!("Response detokenized: {} PII entities restored", token_map.len());

        Ok(restored)
    }

    /// Send audit log to Controller
    async fn send_audit_log(&self, session_id: &str, token_map: &std::collections::HashMap<String, String>) -> Result<()> {
        use std::collections::HashSet;
        
        // Skip if audit logging disabled
        if !self.config.enable_audit_logs {
            info!("Audit logging disabled, skipping");
            return Ok(());
        }

        // Extract unique PII categories from token keys
        // Token format: [CATEGORY_INDEX_SUFFIX] -> extract CATEGORY
        let categories: HashSet<String> = token_map
            .keys()
            .filter_map(|token| {
                // Remove [ and ]
                token.strip_prefix('[')
                    .and_then(|s| s.strip_suffix(']'))
                    // Split on _ and take first part (category)
                    .and_then(|s| s.split('_').next())
                    .map(|c| c.to_string())
            })
            .collect();

        // Build audit log payload
        let payload = serde_json::json!({
            "session_id": session_id,
            "redaction_count": token_map.len(),
            "categories": categories.into_iter().collect::<Vec<String>>(),
            "mode": format!("{:?}", self.config.mode),
            "timestamp": chrono::Utc::now().timestamp()
        });

        // Send to Controller
        let url = format!("{}/privacy/audit", self.config.controller_url);
        
        match reqwest::Client::new()
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Audit log sent successfully: {} redactions", token_map.len());
                } else {
                    warn!("Audit log rejected by Controller: {}", response.status());
                }
            }
            Err(e) => {
                // Don't fail the whole operation if audit log fails
                warn!("Failed to send audit log (continuing anyway): {}", e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_interceptor_creation() {
        let config = Config::from_env().unwrap();
        let interceptor = RequestInterceptor::new(config);
        assert!(interceptor.is_ok());
    }

    #[tokio::test]
    async fn test_response_interceptor_creation() {
        let config = Config::from_env().unwrap();
        let interceptor = ResponseInterceptor::new(config);
        assert!(interceptor.is_ok());
    }
}
