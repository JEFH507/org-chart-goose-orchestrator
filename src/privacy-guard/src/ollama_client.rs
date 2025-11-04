use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Ollama HTTP client for Named Entity Recognition (NER)
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
    enabled: bool,
}

impl OllamaClient {
    /// Create a new Ollama client with explicit configuration
    pub fn new(base_url: String, model: String, enabled: bool) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            base_url,
            model,
            enabled,
        }
    }

    /// Create Ollama client from environment variables
    /// - GUARD_MODEL_ENABLED: Enable/disable model-enhanced detection (default: false)
    /// - OLLAMA_URL: Ollama service URL (default: http://ollama:11434)
    /// - OLLAMA_MODEL: Model name (default: qwen3:0.6b)
    pub fn from_env() -> Self {
        let enabled = std::env::var("GUARD_MODEL_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let base_url = std::env::var("OLLAMA_URL")
            .unwrap_or_else(|_| "http://ollama:11434".to_string());

        let model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "qwen3:0.6b".to_string());

        tracing::info!(
            "Ollama NER: {} (model: {}, url: {})",
            if enabled { "ENABLED" } else { "DISABLED" },
            model,
            base_url
        );

        Self::new(base_url, model, enabled)
    }

    /// Check if model is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get model name
    pub fn model_name(&self) -> &str {
        &self.model
    }

    /// Extract named entities using Ollama chat completion
    /// Returns list of detected entities or empty list on failure
    pub async fn extract_entities(&self, text: &str) -> Result<Vec<NerEntity>, String> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let prompt = self.build_ner_prompt(text);

        let req = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);

        let res = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !res.status().is_success() {
            tracing::warn!("Ollama returned error status: {}", res.status());
            return Ok(Vec::new()); // Fail gracefully
        }

        let ollama_res: OllamaResponse = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        // Parse response text to extract entities
        Ok(parse_ner_response(&ollama_res.response))
    }

    /// Build NER prompt for PII extraction
    fn build_ner_prompt(&self, text: &str) -> String {
        format!(
            "Extract PII from the following text. Return only the entity type and text, one per line.\n\
             Entity types: PERSON, ORGANIZATION, LOCATION, EMAIL, PHONE, SSN, CREDIT_CARD, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER\n\
             Format: TYPE: text\n\n\
             Text: {}\n\n\
             Entities:",
            text
        )
    }

    /// Check if Ollama service is healthy
    pub async fn health_check(&self) -> bool {
        if !self.enabled {
            return true; // Not an error if disabled
        }

        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(res) if res.status().is_success() => true,
            Ok(_) | Err(_) => {
                tracing::warn!(
                    "Ollama health check failed, model detection will be disabled"
                );
                false
            }
        }
    }
}

/// Ollama API request structure
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama API response structure
#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Named entity extracted by model
#[derive(Debug, Clone, PartialEq)]
pub struct NerEntity {
    pub entity_type: String,
    pub text: String,
}

/// Parse NER response from Ollama
/// Expected format: "TYPE: text" per line
fn parse_ner_response(response: &str) -> Vec<NerEntity> {
    let mut entities = Vec::new();

    for line in response.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((entity_type, text)) = line.split_once(':') {
            let entity_type = entity_type.trim().to_uppercase();
            let text = text.trim();

            if !entity_type.is_empty() && !text.is_empty() {
                entities.push(NerEntity {
                    entity_type,
                    text: text.to_string(),
                });
            }
        }
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ner_response() {
        let response = "PERSON: John Doe\nEMAIL: john@example.com\nPHONE: 555-1234";
        let entities = parse_ner_response(response);

        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].entity_type, "PERSON");
        assert_eq!(entities[0].text, "John Doe");
        assert_eq!(entities[1].entity_type, "EMAIL");
        assert_eq!(entities[1].text, "john@example.com");
        assert_eq!(entities[2].entity_type, "PHONE");
        assert_eq!(entities[2].text, "555-1234");
    }

    #[test]
    fn test_parse_ner_response_with_whitespace() {
        let response = "  PERSON: Alice Smith  \n\n  SSN: 123-45-6789  ";
        let entities = parse_ner_response(response);

        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].entity_type, "PERSON");
        assert_eq!(entities[0].text, "Alice Smith");
        assert_eq!(entities[1].entity_type, "SSN");
        assert_eq!(entities[1].text, "123-45-6789");
    }

    #[test]
    fn test_parse_ner_response_empty() {
        let response = "";
        let entities = parse_ner_response(response);
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_parse_ner_response_malformed() {
        let response = "This is not a valid response\nNo colon here";
        let entities = parse_ner_response(response);
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_ollama_client_disabled() {
        let client = OllamaClient::new(
            "http://localhost:11434".to_string(),
            "qwen3:0.6b".to_string(),
            false,
        );
        assert!(!client.is_enabled());
        assert_eq!(client.model_name(), "qwen3:0.6b");
    }

    #[test]
    fn test_ollama_client_enabled() {
        let client = OllamaClient::new(
            "http://ollama:11434".to_string(),
            "qwen3:0.6b".to_string(),
            true,
        );
        assert!(client.is_enabled());
        assert_eq!(client.model_name(), "qwen3:0.6b");
    }

    #[tokio::test]
    async fn test_extract_entities_disabled() {
        let client = OllamaClient::new(
            "http://localhost:11434".to_string(),
            "qwen3:0.6b".to_string(),
            false,
        );

        let result = client.extract_entities("Some text").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_build_ner_prompt() {
        let client = OllamaClient::new(
            "http://localhost:11434".to_string(),
            "qwen3:0.6b".to_string(),
            true,
        );

        let prompt = client.build_ner_prompt("Contact John at john@example.com");
        assert!(prompt.contains("Extract PII"));
        assert!(prompt.contains("PERSON"));
        assert!(prompt.contains("EMAIL"));
        assert!(prompt.contains("Contact John at john@example.com"));
    }
}
