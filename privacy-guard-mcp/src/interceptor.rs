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
        // TODO (E5): Implement audit log submission to Controller
        // POST /privacy/audit
        // Body: { session_id, redaction_count, categories, mode, timestamp }
        warn!("Audit log not yet implemented (E5) - session: {}, redactions: {}", session_id, token_map.len());
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
