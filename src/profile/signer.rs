// Profile Signer - Vault-backed cryptographic signatures
//
// This module implements profile signing using HashiCorp Vault's Transit engine.
// Signatures provide tamper protection and authenticity verification for role profiles.

use crate::profile::schema::{Profile, Signature};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::env;

/// Vault Transit API response for signing
#[derive(Debug, Deserialize)]
struct TransitSignResponse {
    data: TransitSignData,
}

#[derive(Debug, Deserialize)]
struct TransitSignData {
    signature: String,
}

/// Vault Transit API response for verification
#[derive(Debug, Deserialize)]
struct TransitVerifyResponse {
    data: TransitVerifyData,
}

#[derive(Debug, Deserialize)]
struct TransitVerifyData {
    valid: bool,
}

/// Profile signer with Vault Transit integration
pub struct ProfileSigner {
    vault_addr: String,
    vault_token: String,
    http_client: reqwest::Client,
}

impl ProfileSigner {
    /// Create a new ProfileSigner from environment variables
    ///
    /// Required environment variables:
    /// - `VAULT_ADDR`: Vault server address (e.g., "http://localhost:8200")
    /// - `VAULT_TOKEN`: Vault access token
    pub fn from_env() -> Result<Self> {
        let vault_addr = env::var("VAULT_ADDR")
            .context("VAULT_ADDR environment variable not set")?;
        let vault_token = env::var("VAULT_TOKEN")
            .context("VAULT_TOKEN environment variable not set")?;

        Ok(Self {
            vault_addr,
            vault_token,
            http_client: reqwest::Client::new(),
        })
    }

    /// Create a new ProfileSigner with explicit configuration
    pub fn new(vault_addr: String, vault_token: String) -> Self {
        Self {
            vault_addr,
            vault_token,
            http_client: reqwest::Client::new(),
        }
    }

    /// Sign a profile using Vault Transit HMAC
    ///
    /// Generates an HMAC signature over the profile JSON (excluding the signature field).
    /// The signature can be verified later to detect tampering.
    ///
    /// # Arguments
    /// * `profile` - Profile to sign (signature field will be ignored)
    /// * `signed_by` - Email of the signer (for audit trail)
    /// * `key_name` - Vault transit key name (defaults to "profile-signing")
    ///
    /// # Returns
    /// Complete Signature struct with algorithm, vault_key, signed_at, signed_by, and signature
    pub async fn sign(
        &self,
        profile: &Profile,
        signed_by: &str,
        key_name: Option<&str>,
    ) -> Result<Signature> {
        let key_name = key_name.unwrap_or("profile-signing");

        // Serialize profile to JSON (excluding signature field)
        let mut profile_to_sign = profile.clone();
        profile_to_sign.signature = None; // Ensure signature field is not included
        let profile_json = serde_json::to_string(&profile_to_sign)
            .context("Failed to serialize profile to JSON")?;

        // Convert to base64 for Vault (Transit API expects base64-encoded input)
        let input_base64 = base64::encode(&profile_json);

        // Prepare Vault Transit sign request
        #[derive(Serialize)]
        struct SignRequest {
            input: String,
        }

        let sign_url = format!(
            "{}/v1/transit/hmac/{}",
            self.vault_addr, key_name
        );

        // Call Vault Transit API
        let response = self
            .http_client
            .post(&sign_url)
            .header("X-Vault-Token", &self.vault_token)
            .json(&SignRequest {
                input: input_base64,
            })
            .send()
            .await
            .context("Failed to send sign request to Vault")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Vault Transit sign failed (status {}): {}",
                status,
                body
            );
        }

        let sign_response: TransitSignResponse = response
            .json()
            .await
            .context("Failed to parse Vault sign response")?;

        // Create signature struct
        let signed_at = chrono::Utc::now().to_rfc3339();
        Ok(Signature {
            algorithm: "HS256".to_string(), // HMAC-SHA256
            vault_key: format!("transit/hmac/{}", key_name),
            signed_at,
            signed_by: signed_by.to_string(),
            signature: sign_response.data.signature,
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

        // Extract key name from vault_key path (e.g., "transit/hmac/profile-signing" → "profile-signing")
        let key_name = signature_data
            .vault_key
            .strip_prefix("transit/hmac/")
            .context("Invalid vault_key format")?;

        // Serialize profile to JSON (excluding signature field)
        let mut profile_to_verify = profile.clone();
        profile_to_verify.signature = None;
        let profile_json = serde_json::to_string(&profile_to_verify)
            .context("Failed to serialize profile to JSON")?;

        // Convert to base64 for Vault
        let input_base64 = base64::encode(&profile_json);

        // Prepare Vault Transit verify request
        #[derive(Serialize)]
        struct VerifyRequest {
            input: String,
            hmac: String,
        }

        let verify_url = format!(
            "{}/v1/transit/verify/{}",
            self.vault_addr, key_name
        );

        // Call Vault Transit API
        let response = self
            .http_client
            .post(&verify_url)
            .header("X-Vault-Token", &self.vault_token)
            .json(&VerifyRequest {
                input: input_base64,
                hmac: signature_data.signature.clone(),
            })
            .send()
            .await
            .context("Failed to send verify request to Vault")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Vault Transit verify failed (status {}): {}",
                status,
                body
            );
        }

        let verify_response: TransitVerifyResponse = response
            .json()
            .await
            .context("Failed to parse Vault verify response")?;

        Ok(verify_response.data.valid)
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
    #[ignore] // Requires Vault instance
    async fn test_sign_and_verify() {
        // Setup: requires VAULT_ADDR and VAULT_TOKEN environment variables
        let signer = ProfileSigner::from_env().unwrap();
        let mut profile = create_test_profile();

        // Sign the profile
        let signature = signer
            .sign(&profile, "test@example.com", None)
            .await
            .unwrap();

        assert_eq!(signature.algorithm, "HS256");
        assert!(signature.signature.starts_with("vault:"));
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
        let signer = ProfileSigner::from_env().unwrap();
        let mut profile = create_test_profile();

        // Sign the profile
        let signature = signer
            .sign(&profile, "test@example.com", None)
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
            algorithm: "HS256".to_string(),
            vault_key: "transit/hmac/profile-signing".to_string(),
            signed_at: "2025-11-05T14:00:00Z".to_string(),
            signed_by: "admin@example.com".to_string(),
            signature: "vault:v1:HMAC...".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&signature).unwrap();
        assert!(json.contains("HS256"));

        // Test deserialization
        let deserialized: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.algorithm, "HS256");
    }
}
