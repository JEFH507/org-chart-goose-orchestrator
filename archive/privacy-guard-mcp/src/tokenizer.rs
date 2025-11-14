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

    /// Store tokens for session (encrypted with AES-256-GCM)
    pub async fn store_tokens(&self, session_id: &str, token_map: &HashMap<String, String>) -> Result<()> {
        if token_map.is_empty() {
            debug!("No tokens to store for session {}", session_id);
            return Ok(());
        }

        let path = self.get_token_path(session_id);

        // Serialize token map to JSON
        let json = serde_json::to_string(token_map)
            .context("Failed to serialize token map")?;

        // Encrypt using AES-256-GCM
        let encrypted = self.encrypt_data(json.as_bytes())
            .context("Failed to encrypt token map")?;

        // Write encrypted data (nonce + ciphertext)
        fs::write(&path, encrypted)
            .with_context(|| format!("Failed to write encrypted tokens to {:?}", path))?;

        info!("Stored {} tokens for session {} (AES-256-GCM encrypted)", token_map.len(), session_id);

        Ok(())
    }

    /// Load tokens for session (decrypt with AES-256-GCM)
    pub async fn load_tokens(&self, session_id: &str) -> Result<HashMap<String, String>> {
        let path = self.get_token_path(session_id);

        if !path.exists() {
            debug!("No tokens found for session {}", session_id);
            return Ok(HashMap::new());
        }

        // Read encrypted data (nonce + ciphertext)
        let encrypted = fs::read(&path)
            .with_context(|| format!("Failed to read encrypted tokens from {:?}", path))?;

        // Decrypt using AES-256-GCM
        let decrypted = self.decrypt_data(&encrypted)
            .context("Failed to decrypt token map")?;

        // Deserialize JSON
        let json = String::from_utf8(decrypted)
            .context("Invalid UTF-8 in decrypted token map")?;
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

    /// Encrypt data using AES-256-GCM
    /// Returns: nonce (12 bytes) + ciphertext
    fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use rand::RngCore;

        // Create cipher with 256-bit key
        let cipher = Aes256Gcm::new_from_slice(&self.config.encryption_key)
            .map_err(|_| anyhow::anyhow!("Failed to create AES-256-GCM cipher: invalid key length"))?;

        // Generate random 12-byte nonce (96 bits, recommended for GCM)
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt plaintext
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext (needed for decryption)
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    /// Input: nonce (12 bytes) + ciphertext
    fn decrypt_data(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        if encrypted.len() < 12 {
            anyhow::bail!("Invalid encrypted data: too short (need at least 12-byte nonce)");
        }

        // Extract nonce (first 12 bytes) and ciphertext (rest)
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Create cipher with 256-bit key
        let cipher = Aes256Gcm::new_from_slice(&self.config.encryption_key)
            .map_err(|_| anyhow::anyhow!("Failed to create AES-256-GCM cipher: invalid key length"))?;

        // Decrypt ciphertext
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
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

    #[test]
    fn test_encryption_decryption() {
        let config = Config::from_env().unwrap();
        let tokenizer = Tokenizer::new(config).unwrap();

        let plaintext = b"sensitive PII data: SSN 123-45-6789";

        // Encrypt
        let encrypted = tokenizer.encrypt_data(plaintext).unwrap();

        // Verify structure (12-byte nonce + ciphertext)
        assert!(encrypted.len() > 12, "Encrypted data should contain nonce + ciphertext");

        // Verify encrypted data is different from plaintext
        assert_ne!(&encrypted[12..], plaintext);

        // Decrypt
        let decrypted = tokenizer.decrypt_data(&encrypted).unwrap();

        // Verify decrypted matches original
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encryption_unique_nonce() {
        let config = Config::from_env().unwrap();
        let tokenizer = Tokenizer::new(config).unwrap();

        let plaintext = b"same plaintext";

        // Encrypt twice
        let encrypted1 = tokenizer.encrypt_data(plaintext).unwrap();
        let encrypted2 = tokenizer.encrypt_data(plaintext).unwrap();

        // Nonces should be different (first 12 bytes)
        assert_ne!(&encrypted1[..12], &encrypted2[..12], "Nonces should be unique");

        // Ciphertexts should be different (due to different nonces)
        assert_ne!(&encrypted1[12..], &encrypted2[12..], "Ciphertexts should differ with different nonces");

        // Both should decrypt to same plaintext
        let decrypted1 = tokenizer.decrypt_data(&encrypted1).unwrap();
        let decrypted2 = tokenizer.decrypt_data(&encrypted2).unwrap();
        assert_eq!(decrypted1, plaintext);
        assert_eq!(decrypted2, plaintext);
    }

    #[test]
    fn test_decryption_invalid_data() {
        use rand::RngCore;

        let config = Config::from_env().unwrap();
        let tokenizer = Tokenizer::new(config).unwrap();

        // Too short (less than 12 bytes)
        let short_data = vec![1, 2, 3, 4, 5];
        let result = tokenizer.decrypt_data(&short_data);
        assert!(result.is_err(), "Should fail on data shorter than nonce size");

        // Invalid ciphertext (random data)
        let mut invalid_data = vec![0u8; 50];
        rand::thread_rng().fill_bytes(&mut invalid_data);
        let result = tokenizer.decrypt_data(&invalid_data);
        assert!(result.is_err(), "Should fail on invalid ciphertext");
    }

    #[tokio::test]
    async fn test_encrypted_storage_persistence() {
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let tokenizer = Tokenizer::new(config).unwrap();

        let mut token_map = HashMap::new();
        token_map.insert("[SSN_0_TEST]".to_string(), "123-45-6789".to_string());

        // Store tokens (encrypted)
        tokenizer.store_tokens("test-encrypted", &token_map).await.unwrap();

        // Read raw file content
        let path = temp_dir.path().join("session_test-encrypted.json");
        let raw_content = fs::read(&path).unwrap();

        // Verify file is NOT plain JSON (should be encrypted binary)
        let json_check = serde_json::from_slice::<HashMap<String, String>>(&raw_content);
        assert!(json_check.is_err(), "File should be encrypted, not plain JSON");

        // Verify we can still load through tokenizer
        let loaded = tokenizer.load_tokens("test-encrypted").await.unwrap();
        assert_eq!(loaded, token_map);
    }
}
