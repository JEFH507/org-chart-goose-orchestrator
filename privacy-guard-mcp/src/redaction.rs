// PII redaction logic - reuses Phase 2.2 patterns

use anyhow::Result;
use regex::Regex;
use tracing::{debug, info};

use crate::config::{Config, PiiCategory, PrivacyMode};

/// PII redactor
pub struct Redactor {
    config: Config,
    patterns: Vec<(PiiCategory, Regex, String)>, // (category, pattern, replacement)
}

impl Redactor {
    /// Create new redactor
    pub fn new(config: Config) -> Result<Self> {
        let patterns = Self::build_patterns(&config.categories)?;
        Ok(Self { config, patterns })
    }

    /// Build regex patterns for configured PII categories
    fn build_patterns(categories: &[PiiCategory]) -> Result<Vec<(PiiCategory, Regex, String)>> {
        let mut patterns = Vec::new();

        for category in categories {
            let (pattern_str, replacement) = match category {
                PiiCategory::Ssn => {
                    // SSN: 123-45-6789 or 123456789
                    (r"\b\d{3}-?\d{2}-?\d{4}\b", "[SSN]")
                }
                PiiCategory::Email => {
                    // Email: user@example.com
                    (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b", "[EMAIL]")
                }
                PiiCategory::Phone => {
                    // Phone: (123) 456-7890, 123-456-7890, 1234567890
                    (r"\b(\+\d{1,2}\s?)?(\(?\d{3}\)?[\s.-]?)?\d{3}[\s.-]?\d{4}\b", "[PHONE]")
                }
                PiiCategory::CreditCard => {
                    // Credit card: 4111-1111-1111-1111, 4111111111111111
                    (r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b", "[CREDIT_CARD]")
                }
                PiiCategory::EmployeeId => {
                    // Employee ID: EMP12345, E-12345
                    (r"\b[E][M]?[P]?-?\d{5,8}\b", "[EMPLOYEE_ID]")
                }
                PiiCategory::IpAddress => {
                    // IP address: 192.168.1.1
                    (r"\b(?:\d{1,3}\.){3}\d{1,3}\b", "[IP_ADDRESS]")
                }
                PiiCategory::PersonName | PiiCategory::Organization => {
                    // NER-based detection (Phase E2 - deferred to NER mode)
                    continue;
                }
            };

            let regex = Regex::new(pattern_str)?;
            patterns.push((category.clone(), regex, replacement.to_string()));
        }

        Ok(patterns)
    }

    /// Redact PII from text
    pub async fn redact(&self, text: &str) -> Result<String> {
        match self.config.mode {
            PrivacyMode::Off => {
                debug!("Privacy mode: Off (passthrough)");
                Ok(text.to_string())
            }
            PrivacyMode::Rules => self.redact_rules(text).await,
            PrivacyMode::Ner => self.redact_ner(text).await,
            PrivacyMode::Hybrid => {
                // Apply rules first, then NER
                let rules_result = self.redact_rules(text).await?;
                self.redact_ner(&rules_result).await
            }
        }
    }

    /// Apply regex-based redaction
    async fn redact_rules(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();
        let mut redaction_count = 0;

        for (category, pattern, replacement) in &self.patterns {
            let before_len = result.len();
            result = pattern.replace_all(&result, replacement.as_str()).to_string();
            let after_len = result.len();

            if before_len != after_len {
                redaction_count += 1;
                debug!("Redacted {:?} using pattern", category);
            }
        }

        info!("Rules-based redaction complete: {} patterns matched", redaction_count);
        Ok(result)
    }

    /// Apply NER-based redaction (Ollama)
    async fn redact_ner(&self, text: &str) -> Result<String> {
        use crate::ollama::OllamaClient;
        
        // Create Ollama client
        let client = OllamaClient::new(
            self.config.ollama_url.clone(),
            "llama3.2:latest".to_string(),
        );
        
        // Health check (non-blocking)
        if !client.health_check().await {
            debug!("Ollama not available, skipping NER redaction");
            return Ok(text.to_string());
        }
        
        // Extract entities
        let entities = match client.extract_entities(text).await {
            Ok(e) => e,
            Err(err) => {
                debug!("NER extraction failed: {}, skipping", err);
                return Ok(text.to_string());
            }
        };
        
        if entities.is_empty() {
            debug!("No entities detected by NER");
            return Ok(text.to_string());
        }
        
        // Apply redactions based on entity type
        let mut result = text.to_string();
        let mut redaction_count = 0;
        
        for entity in entities {
            let replacement = match entity.entity_type.as_str() {
                "PERSON" => "[PERSON]",
                "ORGANIZATION" => "[ORG]",
                "EMAIL" => "[EMAIL]",
                "PHONE" => "[PHONE]",
                "SSN" => "[SSN]",
                "CREDIT_CARD" => "[CREDIT_CARD]",
                "IP_ADDRESS" => "[IP_ADDRESS]",
                _ => continue, // Unknown entity type
            };
            
            // Only redact if text contains this entity
            if result.contains(&entity.text) {
                result = result.replace(&entity.text, replacement);
                redaction_count += 1;
                debug!("NER redacted {:?}: {}", entity.entity_type, entity.text);
            }
        }
        
        info!("NER redaction complete: {} entities redacted", redaction_count);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redact_ssn() {
        let mut config = Config::from_env().unwrap();
        config.mode = PrivacyMode::Rules;
        config.categories = vec![PiiCategory::Ssn];

        let redactor = Redactor::new(config).unwrap();
        let input = "Employee SSN is 123-45-6789 on file.";
        let output = redactor.redact(input).await.unwrap();

        assert_eq!(output, "Employee SSN is [SSN] on file.");
    }

    #[tokio::test]
    async fn test_redact_email() {
        let mut config = Config::from_env().unwrap();
        config.mode = PrivacyMode::Rules;
        config.categories = vec![PiiCategory::Email];

        let redactor = Redactor::new(config).unwrap();
        let input = "Contact john.doe@example.com for details.";
        let output = redactor.redact(input).await.unwrap();

        assert_eq!(output, "Contact [EMAIL] for details.");
    }

    #[tokio::test]
    async fn test_redact_multiple_categories() {
        let mut config = Config::from_env().unwrap();
        config.mode = PrivacyMode::Rules;
        config.categories = vec![PiiCategory::Ssn, PiiCategory::Email, PiiCategory::Phone];

        let redactor = Redactor::new(config).unwrap();
        let input = "Call 555-123-4567 or email john@example.com. SSN: 123-45-6789.";
        let output = redactor.redact(input).await.unwrap();

        assert!(output.contains("[PHONE]"));
        assert!(output.contains("[EMAIL]"));
        assert!(output.contains("[SSN]"));
    }

    #[tokio::test]
    async fn test_mode_off() {
        let mut config = Config::from_env().unwrap();
        config.mode = PrivacyMode::Off;

        let redactor = Redactor::new(config).unwrap();
        let input = "SSN: 123-45-6789";
        let output = redactor.redact(input).await.unwrap();

        assert_eq!(output, input); // Unchanged
    }
}
