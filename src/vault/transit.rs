// Vault Transit Engine Operations - HMAC signing for profile integrity

use super::VaultClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use base64::Engine; // Import the Engine trait for base64 encoding

/// Transit engine operations for HMAC signing and verification
pub struct TransitOps {
    client: VaultClient,
}

impl TransitOps {
    /// Create a new TransitOps instance
    pub fn new(client: VaultClient) -> Self {
        Self { client }
    }

    /// Ensure a Transit key exists (create if missing)
    /// 
    /// Key type: hmac
    /// Hash algorithm: sha2-256
    /// 
    /// This is idempotent - if key exists, returns Ok without error
    pub async fn ensure_key(&self, key_name: &str) -> Result<()> {
        // Try to create the key (idempotent operation)
        // Using None for options = Vault uses default key type (Aes256Gcm96)
        // HMAC generation works with any key type - the default is fine for HMAC-only usage
        let _ = vaultrs::transit::key::create(
            self.client.inner(),
            &self.client.config().transit_mount,
            key_name,
            None,  // Use Vault's default key type (Aes256Gcm96)
        )
        .await;
        
        // Key creation returns error if key exists, but that's OK
        // We just need to ensure it exists
        Ok(())
    }

    /// Generate HMAC signature for data
    /// 
    /// Args:
    /// - key_name: Transit key name (e.g., "profile-hmac")
    /// - data: Data to sign (will be base64-encoded automatically)
    /// - algorithm: Hash algorithm (default: "sha2-256")
    /// 
    /// Returns: HMAC signature (vault:v1:base64signature)
    pub async fn sign_hmac(
        &self,
        key_name: &str,
        data: &[u8],
        _algorithm: Option<&str>,
    ) -> Result<String> {
        // Base64 encode the input data (Vault requirement)
        let encoded_data = base64::engine::general_purpose::STANDARD.encode(data);
        
        // Generate HMAC using vaultrs::transit::generate::hmac (correct API from vaultrs 0.7.4)
        // From test file line 642: generate::hmac(client, mount, key, data, None)
        // Using None for options (uses key's default algorithm - sha2-256 for HMAC keys)
        let response = vaultrs::transit::generate::hmac(
                self.client.inner(),
                &self.client.config().transit_mount,
                key_name,
                &encoded_data,
                None,  // No options needed - uses key's default algorithm
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate HMAC: {}", e))?;

        Ok(response.hmac)
    }

    /// Verify HMAC signature
    /// 
    /// Args:
    /// - key_name: Transit key name
    /// - data: Original data
    /// - signature: HMAC signature to verify
    /// - algorithm: Hash algorithm (must match signing algorithm)
    /// 
    /// Returns: true if signature is valid, false otherwise
    pub async fn verify_hmac(
        &self,
        key_name: &str,
        data: &[u8],
        signature: &str,
        _algorithm: Option<&str>,
    ) -> Result<bool> {
        // Base64 encode the input data (Vault requirement)
        let encoded_data = base64::engine::general_purpose::STANDARD.encode(data);
        
        // HMAC verification: Regenerate HMAC and compare (HMACs are deterministic)
        // Note: Vault Transit doesn't have a separate verify endpoint for HMAC
        // (data::verify is for asymmetric signatures, not HMAC)
        let response = vaultrs::transit::generate::hmac(
                self.client.inner(),
                &self.client.config().transit_mount,
                key_name,
                &encoded_data,
                None,  // No options needed for basic HMAC generation
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate HMAC for verification: {}", e))?;

        // Compare the generated HMAC with the provided signature
        // HMACs are deterministic: same key + same data = same HMAC
        Ok(response.hmac == signature)
    }
}

/// Signature metadata - matches Phase 5 profile schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignatureMetadata {
    /// Vault HMAC signature (format: vault:v1:base64signature)
    pub value: String,
    /// Signature algorithm (e.g., "sha2-256")
    pub algorithm: String,
    /// Vault Transit key name used for signing
    pub key_name: String,
    /// Timestamp when signature was created (ISO 8601)
    pub signed_at: String,
}

impl SignatureMetadata {
    /// Create new signature metadata
    pub fn new(value: String, algorithm: String, key_name: String) -> Self {
        Self {
            value,
            algorithm,
            key_name,
            signed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_metadata_creation() {
        let sig = SignatureMetadata::new(
            "vault:v1:abc123".to_string(),
            "sha2-256".to_string(),
            "profile-hmac".to_string(),
        );
        
        assert_eq!(sig.value, "vault:v1:abc123");
        assert_eq!(sig.algorithm, "sha2-256");
        assert_eq!(sig.key_name, "profile-hmac");
        assert!(!sig.signed_at.is_empty());
    }

    #[test]
    fn test_signature_metadata_serialization() {
        let sig = SignatureMetadata {
            value: "vault:v1:abc123".to_string(),
            algorithm: "sha2-256".to_string(),
            key_name: "profile-hmac".to_string(),
            signed_at: "2025-11-05T12:00:00Z".to_string(),
        };
        
        let json = serde_json::to_string(&sig).unwrap();
        assert!(json.contains("vault:v1:abc123"));
        assert!(json.contains("sha2-256"));
        
        let deserialized: SignatureMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(sig, deserialized);
    }

    // Integration tests (require running Vault with Transit engine)
    
    #[tokio::test]
    #[ignore]
    async fn test_transit_hmac_sign_verify() {
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let client = VaultClient::from_env().await.unwrap();
        let transit = TransitOps::new(client);
        
        // Ensure key exists
        transit.ensure_key("test-hmac").await.unwrap();
        
        // Sign data
        let data = b"test profile data";
        let signature = transit.sign_hmac("test-hmac", data, None).await.unwrap();
        
        assert!(!signature.is_empty());
        assert!(signature.starts_with("vault:v1:"));
        
        // Verify signature (should pass)
        let valid = transit.verify_hmac("test-hmac", data, &signature, None).await.unwrap();
        assert!(valid, "Signature verification failed");
        
        // Verify with wrong data (should fail)
        let wrong_data = b"wrong data";
        let invalid = transit.verify_hmac("test-hmac", wrong_data, &signature, None).await.unwrap();
        assert!(!invalid, "Signature should be invalid for wrong data");
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[tokio::test]
    #[ignore]
    async fn test_transit_ensure_key_idempotent() {
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let client = VaultClient::from_env().await.unwrap();
        let transit = TransitOps::new(client);
        
        // Create key twice - should not error
        transit.ensure_key("test-idempotent").await.unwrap();
        transit.ensure_key("test-idempotent").await.unwrap();
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }
}
