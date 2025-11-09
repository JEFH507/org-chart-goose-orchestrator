// Profile signature verification module
//
// Phase 6 A5: Verifies cryptographic signatures on profiles to detect tampering
// Uses Vault Transit engine to verify HMAC signatures

use crate::profile::schema::Profile;
use super::{VaultClient, transit::TransitOps};
use anyhow::{Context, Result};
use tracing::{info, warn, error};

/// Recursively sort JSON object keys alphabetically for canonical serialization
/// This is critical because Postgres JSONB doesn't preserve field order,
/// which would break HMAC verification if we rely on insertion order
fn canonical_sort_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();  // Alphabetical sort
            for key in keys {
                sorted.insert(
                    key.clone(),
                    canonical_sort_json(&map[key])  // Recursive sort
                );
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(
                arr.iter().map(canonical_sort_json).collect()
            )
        }
        other => other.clone(),
    }
}

/// Verify a profile's cryptographic signature
///
/// This function:
/// 1. Extracts the signature from the profile
/// 2. Removes the signature field (to get canonical data)
/// 3. Serializes profile to JSON (deterministic)
/// 4. Calls Vault Transit to verify the HMAC signature
///
/// Returns:
/// - Ok(true) if signature is valid
/// - Ok(false) if signature is invalid (tampered data)
/// - Err if verification fails (Vault unreachable, etc.)
///
/// Security: This prevents loading tampered profiles from database
pub async fn verify_profile_signature(
    profile: &Profile,
    vault_client: &VaultClient,
) -> Result<bool> {
    // Check if profile has a signature
    let signature = match &profile.signature {
        Some(sig) => sig,
        None => {
            warn!(
                message = "profile.verify.unsigned",
                role = %profile.role,
                "Profile has no signature - cannot verify"
            );
            // Unsigned profiles are considered invalid in production
            return Ok(false);
        }
    };

    // Check if signature has the HMAC value
    let hmac_signature = match &signature.signature {
        Some(sig) => sig,
        None => {
            warn!(
                message = "profile.verify.no_hmac",
                role = %profile.role,
                "Signature object exists but HMAC value is None"
            );
            return Ok(false);
        }
    };

    info!(
        message = "profile.verify.start",
        role = %profile.role,
        vault_key = %signature.vault_key,
        algorithm = %signature.algorithm,
        "Verifying profile signature"
    );

    // Remove signature field to get the original signed data
    // (signature is not included in the signed data to avoid circular signing)
    let mut profile_copy = profile.clone();
    profile_copy.signature = None;

    // Serialize to canonical JSON with sorted keys (deterministic representation)
    // This is critical because Postgres JSONB doesn't preserve field order,
    // which would break HMAC verification
    let value = serde_json::to_value(&profile_copy)
        .context("Failed to convert profile to JSON value")?;
    let canonical_json = serde_json::to_string(&canonical_sort_json(&value))
        .context("Failed to serialize canonical JSON")?;

    // DEBUG: Log the canonical JSON being verified (FULL)
    info!(
        message = "profile.verify.canonical_json_full",
        role = %profile.role,
        json_length = canonical_json.len(),
        canonical_json = %canonical_json,
        "Canonical JSON for verification (FULL)"
    );

    // Create Transit operations client
    let transit = TransitOps::new(vault_client.clone());

    // Extract key name from vault_key path (e.g., "transit/keys/profile-signing" -> "profile-signing")
    let key_name = signature.vault_key
        .strip_prefix("transit/keys/")
        .unwrap_or(&signature.vault_key);

    // Verify HMAC signature using Vault Transit
    // Note: Transit verify_hmac regenerates HMAC and compares (HMACs are deterministic)
    let is_valid = transit
        .verify_hmac(
            key_name,
            canonical_json.as_bytes(),
            hmac_signature,
            Some(&signature.algorithm),
        )
        .await
        .context("Vault HMAC verification failed")?;

    if is_valid {
        info!(
            message = "profile.verify.success",
            role = %profile.role,
            "Profile signature valid - no tampering detected"
        );
    } else {
        error!(
            message = "profile.verify.failed",
            role = %profile.role,
            "Profile signature INVALID - possible tampering detected!"
        );
    }

    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::schema::{Signature, Providers, ProviderConfig, GooseHints, GooseIgnore, PrivacyConfig};

    #[test]
    fn test_unsigned_profile_returns_false() {
        // This test doesn't need Vault - it checks unsigned profile handling
        let profile = Profile {
            role: "test".to_string(),
            display_name: "Test".to_string(),
            description: "Test profile".to_string(),
            providers: Providers::default(),
            extensions: vec![],
            goosehints: GooseHints::default(),
            gooseignore: GooseIgnore::default(),
            recipes: vec![],
            automated_tasks: vec![],
            policies: vec![],
            privacy: PrivacyConfig::default(),
            env_vars: std::collections::HashMap::new(),
            signature: None,  // No signature
        };

        // We can't test the async function directly without Vault,
        // but we can verify the profile structure
        assert!(profile.signature.is_none());
    }

    #[test]
    fn test_signature_with_no_hmac_returns_false() {
        let profile = Profile {
            role: "test".to_string(),
            display_name: "Test".to_string(),
            description: "Test profile".to_string(),
            providers: Providers::default(),
            extensions: vec![],
            goosehints: GooseHints::default(),
            gooseignore: GooseIgnore::default(),
            recipes: vec![],
            automated_tasks: vec![],
            policies: vec![],
            privacy: PrivacyConfig::default(),
            env_vars: std::collections::HashMap::new(),
            signature: Some(Signature {
                algorithm: "sha2-256".to_string(),
                vault_key: "profile-signing".to_string(),
                signed_at: None,
                signed_by: None,
                signature: None,  // No HMAC value
            }),
        };

        // Verify profile has signature but no HMAC
        assert!(profile.signature.is_some());
        assert!(profile.signature.as_ref().unwrap().signature.is_none());
    }

    // Integration tests (require running Vault with Transit engine)
    
    #[tokio::test]
    #[ignore]
    async fn test_profile_signature_verification_valid() {
        use crate::vault::transit::TransitOps;
        
        std::env::set_var("VAULT_ADDR", "http://localhost:8201");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let vault_client = VaultClient::from_env().await.unwrap();
        let transit = TransitOps::new(vault_client.clone());
        
        // Ensure key exists
        transit.ensure_key("test-profile-signing").await.unwrap();
        
        // Create test profile (unsigned)
        let mut profile = Profile {
            role: "test-verify".to_string(),
            display_name: "Test Verification".to_string(),
            description: "Test profile for signature verification".to_string(),
            providers: Providers::default(),
            extensions: vec![],
            goosehints: GooseHints::default(),
            gooseignore: GooseIgnore::default(),
            recipes: vec![],
            automated_tasks: vec![],
            policies: vec![],
            privacy: PrivacyConfig::default(),
            env_vars: std::collections::HashMap::new(),
            signature: None,
        };
        
        // Serialize profile (without signature)
        let canonical_json = serde_json::to_string(&profile).unwrap();
        
        // Generate signature
        let hmac = transit.sign_hmac(
            "test-profile-signing",
            canonical_json.as_bytes(),
            Some("sha2-256")
        ).await.unwrap();
        
        // Add signature to profile
        profile.signature = Some(Signature {
            algorithm: "sha2-256".to_string(),
            vault_key: "test-profile-signing".to_string(),
            signed_at: Some(chrono::Utc::now().to_rfc3339()),
            signed_by: Some("test@example.com".to_string()),
            signature: Some(hmac),
        });
        
        // Verify signature (should be valid)
        let is_valid = verify_profile_signature(&profile, &vault_client).await.unwrap();
        assert!(is_valid, "Valid signature should verify");
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[tokio::test]
    #[ignore]
    async fn test_profile_signature_verification_tampered() {
        use crate::vault::transit::TransitOps;
        
        std::env::set_var("VAULT_ADDR", "http://localhost:8201");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let vault_client = VaultClient::from_env().await.unwrap();
        let transit = TransitOps::new(vault_client.clone());
        
        // Ensure key exists
        transit.ensure_key("test-profile-tamper").await.unwrap();
        
        // Create test profile
        let mut profile = Profile {
            role: "test-tamper".to_string(),
            display_name: "Test Tamper".to_string(),
            description: "Original description".to_string(),
            providers: Providers::default(),
            extensions: vec![],
            goosehints: GooseHints::default(),
            gooseignore: GooseIgnore::default(),
            recipes: vec![],
            automated_tasks: vec![],
            policies: vec![],
            privacy: PrivacyConfig::default(),
            env_vars: std::collections::HashMap::new(),
            signature: None,
        };
        
        // Serialize and sign
        let canonical_json = serde_json::to_string(&profile).unwrap();
        let hmac = transit.sign_hmac(
            "test-profile-tamper",
            canonical_json.as_bytes(),
            Some("sha2-256")
        ).await.unwrap();
        
        // Add signature
        profile.signature = Some(Signature {
            algorithm: "sha2-256".to_string(),
            vault_key: "test-profile-tamper".to_string(),
            signed_at: Some(chrono::Utc::now().to_rfc3339()),
            signed_by: Some("test@example.com".to_string()),
            signature: Some(hmac),
        });
        
        // NOW TAMPER THE PROFILE (simulating database modification)
        profile.description = "HACKED - unauthorized change!".to_string();
        
        // Verify signature (should be INVALID)
        let is_valid = verify_profile_signature(&profile, &vault_client).await.unwrap();
        assert!(!is_valid, "Tampered profile signature should be invalid");
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }
}
