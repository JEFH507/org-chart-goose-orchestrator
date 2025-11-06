// Configuration for Privacy Guard MCP Extension

use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::env;

/// Privacy protection mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyMode {
    /// Regex-only redaction (fastest, P50 < 50ms)
    Rules,
    /// NER model redaction (Ollama, accurate but slower)
    Ner,
    /// Hybrid: Rules first, then NER (balanced)
    Hybrid,
    /// No protection (passthrough)
    Off,
}

/// Privacy strictness level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyStrictness {
    /// Maximum protection (deny on uncertainty)
    Strict,
    /// Balanced (redact likely PII)
    Moderate,
    /// Minimal protection (redact only high-confidence PII)
    Permissive,
}

/// PII category for detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PiiCategory {
    Ssn,
    Email,
    Phone,
    CreditCard,
    EmployeeId,
    IpAddress,
    PersonName,
    Organization,
}

/// Privacy Guard MCP configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Privacy protection mode
    pub mode: PrivacyMode,
    
    /// Strictness level
    pub strictness: PrivacyStrictness,
    
    /// PII categories to detect
    pub categories: Vec<PiiCategory>,
    
    /// Controller API URL (for audit logs)
    pub controller_url: String,
    
    /// Ollama URL (for NER mode)
    pub ollama_url: String,
    
    /// Token storage directory
    pub token_storage_dir: String,
    
    /// Local-only mode (for Legal role - no cloud providers)
    pub local_only: bool,
    
    /// Encryption key for token storage (base64 encoded)
    pub encryption_key: Vec<u8>,
    
    /// Enable audit log submission to Controller
    pub enable_audit_logs: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env if present
        dotenvy::dotenv().ok();

        // Privacy mode
        let mode_str = env::var("PRIVACY_GUARD_MODE").unwrap_or_else(|_| "hybrid".to_string());
        let mode = match mode_str.to_lowercase().as_str() {
            "rules" => PrivacyMode::Rules,
            "ner" => PrivacyMode::Ner,
            "hybrid" => PrivacyMode::Hybrid,
            "off" => PrivacyMode::Off,
            _ => {
                tracing::warn!("Unknown privacy mode '{}', defaulting to 'hybrid'", mode_str);
                PrivacyMode::Hybrid
            }
        };

        // Strictness
        let strictness_str = env::var("PRIVACY_GUARD_STRICTNESS")
            .unwrap_or_else(|_| "moderate".to_string());
        let strictness = match strictness_str.to_lowercase().as_str() {
            "strict" => PrivacyStrictness::Strict,
            "moderate" => PrivacyStrictness::Moderate,
            "permissive" => PrivacyStrictness::Permissive,
            _ => {
                tracing::warn!("Unknown strictness '{}', defaulting to 'moderate'", strictness_str);
                PrivacyStrictness::Moderate
            }
        };

        // PII categories
        let categories_str = env::var("PRIVACY_GUARD_CATEGORIES")
            .unwrap_or_else(|_| "SSN,EMAIL,PHONE,CREDIT_CARD,EMPLOYEE_ID".to_string());
        let categories = categories_str
            .split(',')
            .filter_map(|s| match s.trim().to_uppercase().as_str() {
                "SSN" => Some(PiiCategory::Ssn),
                "EMAIL" => Some(PiiCategory::Email),
                "PHONE" => Some(PiiCategory::Phone),
                "CREDIT_CARD" => Some(PiiCategory::CreditCard),
                "EMPLOYEE_ID" => Some(PiiCategory::EmployeeId),
                "IP_ADDRESS" => Some(PiiCategory::IpAddress),
                "PERSON_NAME" => Some(PiiCategory::PersonName),
                "ORGANIZATION" => Some(PiiCategory::Organization),
                _ => None,
            })
            .collect();

        // Controller URL
        let controller_url = env::var("CONTROLLER_URL")
            .unwrap_or_else(|_| "http://localhost:8088".to_string());

        // Ollama URL
        let ollama_url = env::var("OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        // Token storage directory
        let token_storage_dir = env::var("PRIVACY_GUARD_TOKEN_DIR")
            .unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                format!("{}/.goose/pii-tokens", home)
            });

        // Local-only mode
        let local_only = env::var("PRIVACY_GUARD_LOCAL_ONLY")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        // Encryption key (generate or load)
        let encryption_key = match env::var("PRIVACY_GUARD_ENCRYPTION_KEY") {
            Ok(key_b64) => base64::engine::general_purpose::STANDARD
                .decode(key_b64)
                .context("Invalid encryption key (must be base64)")?,
            Err(_) => {
                // Generate random 32-byte key for AES-256
                use rand::RngCore;
                let mut key = vec![0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                tracing::warn!(
                    "No encryption key provided, generated ephemeral key. Set PRIVACY_GUARD_ENCRYPTION_KEY to persist tokens across restarts."
                );
                key
            }
        };

        if encryption_key.len() != 32 {
            anyhow::bail!("Encryption key must be 32 bytes (256 bits) for AES-256");
        }

        // Enable audit logs (default: true)
        let enable_audit_logs = env::var("ENABLE_AUDIT_LOGS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Ok(Self {
            mode,
            strictness,
            categories,
            controller_url,
            ollama_url,
            token_storage_dir,
            local_only,
            encryption_key,
            enable_audit_logs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_mode_deserialize() {
        let json = r#""rules""#;
        let mode: PrivacyMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, PrivacyMode::Rules);
    }

    #[test]
    fn test_config_defaults() {
        // Clear env vars
        env::remove_var("PRIVACY_GUARD_MODE");
        env::remove_var("PRIVACY_GUARD_STRICTNESS");

        let config = Config::from_env().unwrap();
        assert_eq!(config.mode, PrivacyMode::Hybrid);
        assert_eq!(config.strictness, PrivacyStrictness::Moderate);
        assert_eq!(config.encryption_key.len(), 32);
    }
}
