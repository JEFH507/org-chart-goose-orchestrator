// HMAC-SHA256 deterministic pseudonymization
// This module implements deterministic mapping using HMAC

use crate::detection::EntityType;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

/// Generate a deterministic pseudonym using HMAC-SHA256
///
/// Format: {TYPE}_{first_16_hex_chars_of_hash}
///
/// Input: tenant_id || entity_type || original_text
/// Key: PSEUDO_SALT from environment
///
/// # Arguments
/// * `text` - The original PII text to pseudonymize
/// * `entity_type` - The type of entity (SSN, EMAIL, etc.)
/// * `tenant_id` - The tenant identifier for isolation
///
/// # Returns
/// A deterministic pseudonym string like "PERSON_a3f7b2c8e1d4f9a2"
///
/// # Panics
/// Panics if PSEUDO_SALT environment variable is not set
pub fn pseudonymize(text: &str, entity_type: &EntityType, tenant_id: &str) -> String {
    let salt = env::var("PSEUDO_SALT").expect("PSEUDO_SALT environment variable not set");
    pseudonymize_with_salt(text, entity_type, tenant_id, &salt)
}

/// Generate pseudonym with explicit salt (for testing)
pub fn pseudonymize_with_salt(
    text: &str,
    entity_type: &EntityType,
    tenant_id: &str,
    salt: &str,
) -> String {
    // Create HMAC instance with salt as key
    let mut mac = HmacSha256::new_from_slice(salt.as_bytes())
        .expect("HMAC can take key of any size");

    // Build input: tenant_id || entity_type || text
    let input = format!("{}||{}||{}", tenant_id, entity_type, text);
    mac.update(input.as_bytes());

    // Finalize and get result
    let result = mac.finalize();
    let hash_bytes = result.into_bytes();

    // Convert to hex and take first 16 characters (8 bytes)
    let hash_hex: String = hash_bytes
        .iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect();

    // Return format: TYPE_hash
    format!("{}_{}", entity_type, hash_hex)
}

/// Verify if a pseudonym is valid format
pub fn is_valid_pseudonym(pseudonym: &str) -> bool {
    // Format: {TYPE}_{16_hex_chars}
    let parts: Vec<&str> = pseudonym.split('_').collect();
    if parts.len() < 2 {
        return false;
    }

    // Check if the hash part is hex (at least 16 chars)
    let hash_part = parts[1..].join("_");
    hash_part.len() >= 16 && hash_part.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SALT: &str = "test-secret-salt-for-hmac-testing";

    #[test]
    fn test_pseudonymize_determinism() {
        // Same input should produce same output
        let text = "John Doe";
        let entity_type = EntityType::PERSON;
        let tenant_id = "org1";

        let pseudo1 = pseudonymize_with_salt(text, &entity_type, tenant_id, TEST_SALT);
        let pseudo2 = pseudonymize_with_salt(text, &entity_type, tenant_id, TEST_SALT);

        assert_eq!(pseudo1, pseudo2);
        println!("Determinism test: {} -> {}", text, pseudo1);
    }

    #[test]
    fn test_pseudonymize_uniqueness() {
        // Different inputs should produce different outputs
        let tenant_id = "org1";
        let entity_type = EntityType::PERSON;

        let pseudo1 = pseudonymize_with_salt("John Doe", &entity_type, tenant_id, TEST_SALT);
        let pseudo2 = pseudonymize_with_salt("Jane Smith", &entity_type, tenant_id, TEST_SALT);
        let pseudo3 = pseudonymize_with_salt("Bob Johnson", &entity_type, tenant_id, TEST_SALT);

        assert_ne!(pseudo1, pseudo2);
        assert_ne!(pseudo2, pseudo3);
        assert_ne!(pseudo1, pseudo3);

        println!("Uniqueness test:");
        println!("  John Doe -> {}", pseudo1);
        println!("  Jane Smith -> {}", pseudo2);
        println!("  Bob Johnson -> {}", pseudo3);
    }

    #[test]
    fn test_pseudonymize_format() {
        let text = "john.doe@example.com";
        let entity_type = EntityType::EMAIL;
        let tenant_id = "org1";

        let pseudo = pseudonymize_with_salt(text, &entity_type, tenant_id, TEST_SALT);

        // Should start with "EMAIL_"
        assert!(pseudo.starts_with("EMAIL_"));

        // Should have hex characters after underscore
        let hash_part = pseudo.strip_prefix("EMAIL_").unwrap();
        assert_eq!(hash_part.len(), 16);
        assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()));

        println!("Format test: {} -> {}", text, pseudo);
    }

    #[test]
    fn test_tenant_isolation() {
        // Same text, different tenants should produce different pseudonyms
        let text = "john.doe@example.com";
        let entity_type = EntityType::EMAIL;

        let pseudo1 = pseudonymize_with_salt(text, &entity_type, "org1", TEST_SALT);
        let pseudo2 = pseudonymize_with_salt(text, &entity_type, "org2", TEST_SALT);

        assert_ne!(pseudo1, pseudo2);

        println!("Tenant isolation test:");
        println!("  org1: {} -> {}", text, pseudo1);
        println!("  org2: {} -> {}", text, pseudo2);
    }

    #[test]
    fn test_entity_type_differentiation() {
        // Same text, different entity types should produce different pseudonyms
        let text = "123-45-6789";
        let tenant_id = "org1";

        let pseudo1 = pseudonymize_with_salt(text, &EntityType::SSN, tenant_id, TEST_SALT);
        let pseudo2 = pseudonymize_with_salt(text, &EntityType::AccountNumber, tenant_id, TEST_SALT);

        assert_ne!(pseudo1, pseudo2);
        assert!(pseudo1.starts_with("SSN_"));
        assert!(pseudo2.starts_with("ACCOUNT_NUMBER_"));

        println!("Entity type differentiation test:");
        println!("  As SSN: {}", pseudo1);
        println!("  As ACCOUNT_NUMBER: {}", pseudo2);
    }

    #[test]
    fn test_all_entity_types() {
        let tenant_id = "org1";

        let entity_types = vec![
            (EntityType::SSN, "123-45-6789"),
            (EntityType::EMAIL, "test@example.com"),
            (EntityType::PHONE, "555-123-4567"),
            (EntityType::CreditCard, "4532015112830366"),
            (EntityType::PERSON, "John Doe"),
            (EntityType::IpAddress, "192.168.1.1"),
            (EntityType::DateOfBirth, "01/15/1985"),
            (EntityType::AccountNumber, "1234567890"),
        ];

        println!("\nAll entity types pseudonymization:");
        for (entity_type, text) in entity_types {
            let pseudo = pseudonymize_with_salt(text, &entity_type, tenant_id, TEST_SALT);
            println!("  {:15} {} -> {}", format!("{}:", entity_type), text, pseudo);

            // Verify format
            assert!(pseudo.contains('_'));
            let parts: Vec<&str> = pseudo.split('_').collect();
            assert!(parts.len() >= 2);
        }
    }

    #[test]
    fn test_salt_sensitivity() {
        // Different salts should produce different pseudonyms
        let text = "sensitive-data";
        let entity_type = EntityType::PERSON;
        let tenant_id = "org1";

        let pseudo1 = pseudonymize_with_salt(text, &entity_type, tenant_id, "salt1");
        let pseudo2 = pseudonymize_with_salt(text, &entity_type, tenant_id, "salt2");

        assert_ne!(pseudo1, pseudo2);

        println!("Salt sensitivity test:");
        println!("  salt1: {}", pseudo1);
        println!("  salt2: {}", pseudo2);
    }

    #[test]
    fn test_is_valid_pseudonym() {
        // Valid pseudonyms
        assert!(is_valid_pseudonym("PERSON_a3f7b2c8e1d4f9a2"));
        assert!(is_valid_pseudonym("EMAIL_1234567890abcdef"));
        assert!(is_valid_pseudonym("SSN_abcdef1234567890"));
        assert!(is_valid_pseudonym("CREDIT_CARD_0123456789abcdef"));

        // Invalid pseudonyms
        assert!(!is_valid_pseudonym("PERSON"));
        assert!(!is_valid_pseudonym("PERSON_"));
        assert!(!is_valid_pseudonym("PERSON_short"));
        assert!(!is_valid_pseudonym("PERSON_not-hex-chars!"));
        assert!(!is_valid_pseudonym("no_underscore_here"));
        assert!(!is_valid_pseudonym(""));
    }

    #[test]
    fn test_edge_cases() {
        let tenant_id = "org1";

        // Empty string
        let pseudo_empty = pseudonymize_with_salt("", &EntityType::PERSON, tenant_id, TEST_SALT);
        assert!(pseudo_empty.starts_with("PERSON_"));

        // Very long string
        let long_text = "a".repeat(10000);
        let pseudo_long = pseudonymize_with_salt(&long_text, &EntityType::PERSON, tenant_id, TEST_SALT);
        assert!(pseudo_long.starts_with("PERSON_"));
        assert_eq!(pseudo_long.len(), "PERSON_".len() + 16);

        // Special characters
        let special = "!@#$%^&*()_+-=[]{}|;:',.<>?/~`";
        let pseudo_special = pseudonymize_with_salt(special, &EntityType::PERSON, tenant_id, TEST_SALT);
        assert!(pseudo_special.starts_with("PERSON_"));

        // Unicode
        let unicode = "こんにちは 世界 🌍";
        let pseudo_unicode = pseudonymize_with_salt(unicode, &EntityType::PERSON, tenant_id, TEST_SALT);
        assert!(pseudo_unicode.starts_with("PERSON_"));

        println!("\nEdge cases:");
        println!("  Empty: {}", pseudo_empty);
        println!("  Special chars: {}", pseudo_special);
        println!("  Unicode: {}", pseudo_unicode);
    }
}

