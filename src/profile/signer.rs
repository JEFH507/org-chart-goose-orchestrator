// Profile Signer - Vault-backed cryptographic signatures
//
// This module implements profile signing using HashiCorp Vault's Transit engine.
// Signatures provide tamper protection and authenticity verification for role profiles.
//
// **Production-grade Vault client** - Uses vaultrs 0.7.x instead of raw HTTP calls
// This integrates with the centralized Vault client in src/vault/

use crate::profile::schema::{Profile, Signature};
use crate::vault::{VaultClient, VaultConfig, TransitOps};
use anyhow::{Result, Context};

/// Profile signer with production Vault Transit integration
pub struct ProfileSigner {
    transit: TransitOps,
    key_name: String,
}

impl ProfileSigner {
    /// Create a new ProfileSigner from environment variables
    ///
    /// Required environment variables:
    /// - `VAULT_ADDR`: Vault server address (e.g., "http://vault:8200")
    /// - `VAULT_TOKEN`: Vault access token
    pub async fn from_env() -> Result<Self> {
        Self::from_env_with_key("profile-hmac").await
    }

    /// Create a new ProfileSigner from environment with custom key name
    pub async fn from_env_with_key(key_name: &str) -> Result<Self> {
        let config = VaultConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load Vault config: {}", e))?;
        Self::new(config, key_name.to_string()).await
    }

    /// Create a new ProfileSigner with explicit Vault configuration
    pub async fn new(config: VaultConfig, key_name: String) -> Result<Self> {
        let client = VaultClient::new(config).await?;
        let transit = TransitOps::new(client);
        
        // Ensure the HMAC key exists in Vault
        transit.ensure_key(&key_name).await
            .context("Failed to ensure Vault Transit key exists")?;

        Ok(Self {
            transit,
            key_name,
        })
    }

    /// Sign a profile using Vault Transit HMAC
    ///
    /// Generates an HMAC signature over the profile JSON (excluding the signature field).
    /// The signature can be verified later to detect tampering.
    ///
    /// # Arguments
    /// * `profile` - Profile to sign (signature field will be ignored)
    /// * `signed_by` - Email of the signer (for audit trail)
    ///
    /// # Returns
    /// Complete Signature struct with algorithm, vault_key, signed_at, signed_by, and signature
    pub async fn sign(
        &self,
        profile: &Profile,
        signed_by: &str,
    ) -> Result<Signature> {
        // Serialize profile to JSON (excluding signature field)
        let mut profile_to_sign = profile.clone();
        profile_to_sign.signature = None; // Ensure signature field is not included
        let profile_json = serde_json::to_string(&profile_to_sign)
            .context("Failed to serialize profile to JSON")?;

        // Generate HMAC using Transit engine
        let hmac_signature = self.transit.sign_hmac(
            &self.key_name,
            profile_json.as_bytes(),
            Some("sha2-256"),
        )
        .await
        .context("Failed to generate HMAC signature")?;

        // Create signature struct
        let signed_at = chrono::Utc::now().to_rfc3339();
        Ok(Signature {
            algorithm: "sha2-256".to_string(), // HMAC-SHA256
            vault_key: format!("transit/hmac/{}", self.key_name),
            signed_at,
            signed_by: signed_by.to_string(),
            signature: hmac_signature,
        })
    }

    /// Verify a profile signature
    ///
    /// Checks that the signature matches the profile content, detecting any tampering.
    ///
    /// # Arguments
    /// * `profile` - Profile to verify (must include signature field)
    ///
    /// # Returns
    /// `true` if signature is valid, `false` otherwise
    pub async fn verify(&self, profile: &Profile) -> Result<bool> {
        let signature_data = profile
            .signature
            .as_ref()
            .context("Profile has no signature")?;

        // Serialize profile to JSON (excluding signature field)
        let mut profile_to_verify = profile.clone();
        profile_to_verify.signature = None;
        let profile_json = serde_json::to_string(&profile_to_verify)
            .context("Failed to serialize profile to JSON")?;

        // Verify HMAC using Transit engine
        let is_valid = self.transit.verify_hmac(
            &self.key_name,
            profile_json.as_bytes(),
            &signature_data.signature,
            Some(&signature_data.algorithm),
        )
        .await
        .context("Failed to verify HMAC signature")?;

        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::schema::*;
    use std::collections::HashMap;

    // Note: These tests require a running Vault instance with Transit engine enabled
    // For CI/CD, mock the Vault API calls or skip these tests

    fn create_test_profile() -> Profile {
        Profile {
            role: "test".to_string(),
            display_name: "Test Profile".to_string(),
            description: "Test profile for signing".to_string(),
            providers: Providers::default(),
            extensions: vec![],
            goosehints: GooseHints::default(),
            gooseignore: GooseIgnore::default(),
            recipes: vec![],
            automated_tasks: vec![],
            policies: vec![],
            privacy: PrivacyConfig::default(),
            env_vars: HashMap::new(),
            signature: None,
        }
    }

    #[tokio::test]
    #[ignore] // Requires Vault instance at http://localhost:8200 with token=root
    async fn test_sign_and_verify() {
        // Setup: requires VAULT_ADDR and VAULT_TOKEN environment variables
        let signer = ProfileSigner::from_env().await.unwrap();
        let mut profile = create_test_profile();

        // Sign the profile
        let signature = signer
            .sign(&profile, "test@example.com")
            .await
            .unwrap();

        assert_eq!(signature.algorithm, "sha2-256");
        assert!(signature.signature.starts_with("vault:v1:"));
        assert_eq!(signature.signed_by, "test@example.com");

        // Attach signature to profile
        profile.signature = Some(signature);

        // Verify the signature
        let is_valid = signer.verify(&profile).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    #[ignore] // Requires Vault instance
    async fn test_tampered_profile_fails_verification() {
        let signer = ProfileSigner::from_env().await.unwrap();
        let mut profile = create_test_profile();

        // Sign the profile
        let signature = signer
            .sign(&profile, "test@example.com")
            .await
            .unwrap();

        profile.signature = Some(signature);

        // Tamper with the profile
        profile.description = "Tampered description".to_string();

        // Verification should fail
        let is_valid = signer.verify(&profile).await.unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_signature_serialization() {
        let signature = Signature {
            algorithm: "sha2-256".to_string(),
            vault_key: "transit/hmac/profile-hmac".to_string(),
            signed_at: "2025-11-05T14:00:00Z".to_string(),
            signed_by: "admin@example.com".to_string(),
            signature: "vault:v1:HMAC...".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&signature).unwrap();
        assert!(json.contains("sha2-256"));

        // Test deserialization
        let deserialized: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.algorithm, "sha2-256");
    }
}
