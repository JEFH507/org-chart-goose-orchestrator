// PII tokenization and storage

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::config::Config;

/// PII tokenizer - replaces PII with tokens and stores mapping
pub struct Tokenizer {
    config: Config,
}

impl Tokenizer {
    /// Create new tokenizer
    pub fn new(config: Config) -> Result<Self> {
        // Ensure token storage directory exists
        fs::create_dir_all(&config.token_storage_dir)
            .context("Failed to create token storage directory")?;

        Ok(Self { config })
    }

    /// Tokenize PII entities in text
    /// Returns (tokenized_text, token_map)
    pub fn tokenize(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        use regex::Regex;
        use std::collections::HashSet;

        let mut result = text.to_string();
        let mut token_map = HashMap::new();
        let mut seen_tokens = HashSet::new();

        // Patterns for redacted PII markers (literal bracket matching)
        let patterns = vec![
            (r"\[SSN\]", "SSN"),
            (r"\[EMAIL\]", "EMAIL"),
            (r"\[PHONE\]", "PHONE"),
            (r"\[CREDIT_CARD\]", "CREDIT_CARD"),
            (r"\[EMPLOYEE_ID\]", "EMPLOYEE_ID"),
            (r"\[IP_ADDRESS\]", "IP_ADDRESS"),
            (r"\[PERSON\]", "PERSON"),
            (r"\[ORG\]", "ORG"),
        ];

        for (pattern_str, category) in patterns {
            let pattern = Regex::new(pattern_str)?;
            let mut idx = 0;
            
            // Replace all occurrences iteratively
            loop {
                // Find first occurrence in current result
                match pattern.find(&result) {
                    Some(m) => {
                        // Generate deterministic token ID (category + counter)
                        let token_id = format!("{}_{}_{}", category, idx, self.generate_token_suffix());
                        let token = format!("[{}]", token_id);
                        
                        // Ensure uniqueness
                        if seen_tokens.contains(&token) {
                            break; // Shouldn't happen, but safety check
                        }
                        seen_tokens.insert(token.clone());
                        
                        // Store original marker
                        token_map.insert(token.clone(), m.as_str().to_string());
                        
                        // Replace in result
                        result.replace_range(m.start()..m.end(), &token);
                        
                        idx += 1;
                    }
                    None => break, // No more occurrences
                }
            }
        }

        debug!("Tokenized {} PII entities", token_map.len());
        Ok((result, token_map))
    }

    /// Generate random token suffix for uniqueness
    fn generate_token_suffix(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const LEN: usize = 6;
        
        let mut rng = rand::thread_rng();
        (0..LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Detokenize text using token map
    pub fn detokenize(&self, text: &str, token_map: &HashMap<String, String>) -> Result<String> {
        let mut result = text.to_string();

        for (token, original) in token_map {
            result = result.replace(token, original);
        }

        debug!("Detokenized {} tokens", token_map.len());
        Ok(result)
    }

    /// Store tokens for session
    pub async fn store_tokens(&self, session_id: &str, token_map: &HashMap<String, String>) -> Result<()> {
        if token_map.is_empty() {
            debug!("No tokens to store for session {}", session_id);
            return Ok(());
        }

        let path = self.get_token_path(session_id);

        // Encrypt token map (Phase E4 - for now store as plain JSON)
        let json = serde_json::to_string_pretty(token_map)
            .context("Failed to serialize token map")?;

        // TODO (E4): Encrypt using AES-GCM with config.encryption_key
        // For now, write plain JSON with warning
        fs::write(&path, json)
            .with_context(|| format!("Failed to write tokens to {:?}", path))?;

        info!("Stored {} tokens for session {} (UNENCRYPTED - E4 TODO)", token_map.len(), session_id);

        Ok(())
    }

    /// Load tokens for session
    pub async fn load_tokens(&self, session_id: &str) -> Result<HashMap<String, String>> {
        let path = self.get_token_path(session_id);

        if !path.exists() {
            debug!("No tokens found for session {}", session_id);
            return Ok(HashMap::new());
        }

        let json = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read tokens from {:?}", path))?;

        // TODO (E4): Decrypt using AES-GCM with config.encryption_key
        let token_map: HashMap<String, String> = serde_json::from_str(&json)
            .context("Failed to deserialize token map")?;

        info!("Loaded {} tokens for session {}", token_map.len(), session_id);

        Ok(token_map)
    }

    /// Delete tokens for session
    pub async fn delete_tokens(&self, session_id: &str) -> Result<()> {
        let path = self.get_token_path(session_id);

        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete tokens at {:?}", path))?;
            info!("Deleted tokens for session {}", session_id);
        }

        Ok(())
    }

    /// Get file path for session tokens
    fn get_token_path(&self, session_id: &str) -> PathBuf {
        let filename = format!("session_{}.json", session_id);
        PathBuf::from(&self.config.token_storage_dir).join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Config {
        let mut config = Config::from_env().unwrap();
        config.token_storage_dir = temp_dir.path().to_str().unwrap().to_string();
        config
    }

    #[tokio::test]
    async fn test_store_and_load_tokens() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let tokenizer = Tokenizer::new(config).unwrap();

        let mut token_map = HashMap::new();
        token_map.insert("[SSN_ABC]".to_string(), "123-45-6789".to_string());
        token_map.insert("[EMAIL_XYZ]".to_string(), "john@example.com".to_string());

        // Store
        tokenizer.store_tokens("test-session", &token_map).await.unwrap();

        // Load
        let loaded = tokenizer.load_tokens("test-session").await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("[SSN_ABC]"), Some(&"123-45-6789".to_string()));
        assert_eq!(loaded.get("[EMAIL_XYZ]"), Some(&"john@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_delete_tokens() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let tokenizer = Tokenizer::new(config).unwrap();

        let mut token_map = HashMap::new();
        token_map.insert("[SSN_ABC]".to_string(), "123-45-6789".to_string());

        // Store
        tokenizer.store_tokens("test-session", &token_map).await.unwrap();

        // Delete
        tokenizer.delete_tokens("test-session").await.unwrap();

        // Load should return empty
        let loaded = tokenizer.load_tokens("test-session").await.unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_tokenize() {
        let config = Config::from_env().unwrap();
        let tokenizer = Tokenizer::new(config).unwrap();

        let redacted = "Employee [SSN] works at [EMAIL]. Call [PHONE].";
        let (tokenized, token_map) = tokenizer.tokenize(redacted).unwrap();

        // Should have 3 tokens
        assert_eq!(token_map.len(), 3);
        
        // Should contain unique tokens
        assert!(tokenized.contains("[SSN_0_"));
        assert!(tokenized.contains("[EMAIL_0_"));
        assert!(tokenized.contains("[PHONE_0_"));
        
        // Original markers should be replaced
        assert!(!tokenized.contains("[SSN]"));
        assert!(!tokenized.contains("[EMAIL]"));
        assert!(!tokenized.contains("[PHONE]"));
    }

    #[test]
    fn test_detokenize() {
        let config = Config::from_env().unwrap();
        let tokenizer = Tokenizer::new(config).unwrap();

        let mut token_map = HashMap::new();
        token_map.insert("[SSN_ABC]".to_string(), "123-45-6789".to_string());
        token_map.insert("[EMAIL_XYZ]".to_string(), "john@example.com".to_string());

        let tokenized = "Employee [SSN_ABC] email is [EMAIL_XYZ]";
        let detokenized = tokenizer.detokenize(tokenized, &token_map).unwrap();

        assert_eq!(detokenized, "Employee 123-45-6789 email is john@example.com");
    }

    #[test]
    fn test_tokenize_multiple_same_category() {
        let config = Config::from_env().unwrap();
        let tokenizer = Tokenizer::new(config).unwrap();

        let redacted = "[EMAIL] and [EMAIL] are different.";
        let (tokenized, token_map) = tokenizer.tokenize(redacted).unwrap();

        // Should have 2 different tokens
        assert_eq!(token_map.len(), 2);
        assert!(tokenized.contains("[EMAIL_0_"));
        assert!(tokenized.contains("[EMAIL_1_"));
    }
}
