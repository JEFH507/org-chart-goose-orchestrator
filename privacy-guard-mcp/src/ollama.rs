// Ollama NER client (reused from Phase 2.2)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::warn;

/// Ollama HTTP client for Named Entity Recognition (NER)
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("Failed to build HTTP client"),
            base_url,
            model,
        }
    }

    /// Extract named entities using Ollama chat completion
    pub async fn extract_entities(&self, text: &str) -> Result<Vec<NerEntity>, String> {
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
            warn!("Ollama returned error status: {}", res.status());
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
             Entity types: PERSON, ORGANIZATION, EMAIL, PHONE, SSN, CREDIT_CARD, IP_ADDRESS\n\
             Format: TYPE: text\n\n\
             Text: {}\n\n\
             Entities:",
            text
        )
    }

    /// Check if Ollama service is healthy
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(res) if res.status().is_success() => true,
            Ok(_) | Err(_) => {
                warn!("Ollama health check failed");
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
    }

    #[test]
    fn test_parse_ner_response_empty() {
        let response = "";
        let entities = parse_ner_response(response);
        assert_eq!(entities.len(), 0);
    }
}
