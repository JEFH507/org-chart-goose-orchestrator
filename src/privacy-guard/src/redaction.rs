//! Format-preserving encryption and text masking
//! This module implements FPE for phone/SSN and general masking logic

use crate::detection::EntityType;
use fpe::ff1::{BinaryNumeralString, FF1};
use std::error::Error;
use std::fmt;

/// Configuration for format preservation
#[derive(Debug, Clone)]
pub struct PreserveConfig {
    /// For PHONE: preserve area code (first 3 digits)
    pub preserve_area_code: bool,
    /// For SSN: preserve last 4 digits
    pub preserve_last_four: bool,
}

impl Default for PreserveConfig {
    fn default() -> Self {
        Self {
            preserve_area_code: true,
            preserve_last_four: true,
        }
    }
}

/// FPE-specific errors
#[derive(Debug)]
pub enum FpeError {
    InvalidFormat(String),
    EncryptionFailed(String),
    UnsupportedType(String),
}

impl fmt::Display for FpeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FpeError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            FpeError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            FpeError::UnsupportedType(msg) => write!(f, "Unsupported type: {}", msg),
        }
    }
}

impl Error for FpeError {}

/// Apply format-preserving encryption to text based on entity type
///
/// # Arguments
/// * `text` - The original text to encrypt
/// * `entity_type` - The type of entity (PHONE, SSN, etc.)
/// * `key` - Encryption key (32 bytes for AES-256)
/// * `config` - Configuration for format preservation
///
/// # Returns
/// Encrypted text preserving the original format, or error if encryption fails
pub fn fpe_encrypt(
    text: &str,
    entity_type: EntityType,
    key: &[u8; 32],
    config: &PreserveConfig,
) -> Result<String, FpeError> {
    match entity_type {
        EntityType::Phone => encrypt_phone(text, key, config),
        EntityType::Ssn => encrypt_ssn(text, key, config),
        _ => Err(FpeError::UnsupportedType(format!(
            "FPE not supported for {:?}",
            entity_type
        ))),
    }
}

/// Encrypt phone number with optional area code preservation
///
/// Supports formats:
/// - xxx-xxx-xxxx
/// - (xxx) xxx-xxxx
/// - xxx.xxx.xxxx
/// - xxxxxxxxxx (plain)
fn encrypt_phone(
    text: &str,
    key: &[u8; 32],
    config: &PreserveConfig,
) -> Result<String, FpeError> {
    // Extract digits only
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 10 {
        return Err(FpeError::InvalidFormat(format!(
            "Phone number must have 10 digits, got {}",
            digits.len()
        )));
    }

    // Determine format pattern
    let format = detect_phone_format(text);

    // Split into area code and rest
    let (area_code, rest) = if config.preserve_area_code {
        let area = &digits[0..3];
        let remaining = &digits[3..10];
        (area.to_string(), remaining)
    } else {
        ("".to_string(), digits.as_str())
    };

    // Encrypt the non-preserved portion
    let encrypted_rest = encrypt_digits(rest, key)?;

    // Reconstruct with format
    let full_encrypted = if config.preserve_area_code {
        format!("{}{}", area_code, encrypted_rest)
    } else {
        encrypted_rest
    };

    // Apply original format
    Ok(apply_phone_format(&full_encrypted, &format))
}

/// Detect phone number format pattern
fn detect_phone_format(text: &str) -> PhoneFormat {
    if text.contains('(') && text.contains(')') {
        PhoneFormat::Parentheses // (xxx) xxx-xxxx
    } else if text.contains('-') {
        PhoneFormat::Dashes // xxx-xxx-xxxx
    } else if text.contains('.') {
        PhoneFormat::Dots // xxx.xxx.xxxx
    } else {
        PhoneFormat::Plain // xxxxxxxxxx
    }
}

#[derive(Debug, Clone)]
enum PhoneFormat {
    Dashes,
    Parentheses,
    Dots,
    Plain,
}

/// Apply format pattern to phone number digits
fn apply_phone_format(digits: &str, format: &PhoneFormat) -> String {
    if digits.len() != 10 {
        return digits.to_string(); // Fallback for invalid length
    }

    match format {
        PhoneFormat::Dashes => format!("{}-{}-{}", &digits[0..3], &digits[3..6], &digits[6..10]),
        PhoneFormat::Parentheses => {
            format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10])
        }
        PhoneFormat::Dots => format!("{}.{}.{}", &digits[0..3], &digits[3..6], &digits[6..10]),
        PhoneFormat::Plain => digits.to_string(),
    }
}

/// Encrypt SSN with optional last-4 preservation
///
/// Supports formats:
/// - xxx-xx-xxxx
/// - xxxxxxxxx (plain)
fn encrypt_ssn(text: &str, key: &[u8; 32], config: &PreserveConfig) -> Result<String, FpeError> {
    // Extract digits only
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 9 {
        return Err(FpeError::InvalidFormat(format!(
            "SSN must have 9 digits, got {}",
            digits.len()
        )));
    }

    // Detect format
    let has_dashes = text.contains('-');

    // Split into prefix and last 4
    let (prefix, last_four) = if config.preserve_last_four {
        let first_five = &digits[0..5];
        let last = &digits[5..9];
        (first_five, last.to_string())
    } else {
        (digits.as_str(), "".to_string())
    };

    // Encrypt the non-preserved portion
    let encrypted_prefix = encrypt_digits(prefix, key)?;

    // Reconstruct
    let full_encrypted = if config.preserve_last_four {
        format!("{}{}", encrypted_prefix, last_four)
    } else {
        encrypted_prefix
    };

    // Apply format
    if has_dashes {
        Ok(format!(
            "{}-{}-{}",
            &full_encrypted[0..3],
            &full_encrypted[3..5],
            &full_encrypted[5..9]
        ))
    } else {
        Ok(full_encrypted)
    }
}

/// Encrypt a string of digits using FF1 (AES-FFX)
///
/// FF1 is part of NIST SP 800-38G for format-preserving encryption
fn encrypt_digits(digits: &str, key: &[u8; 32]) -> Result<String, FpeError> {
    // Convert digits to radix-10 representation
    let plaintext = BinaryNumeralString::from_bytes_le(digits.as_bytes());

    // Create FF1 cipher with radix 10 (for digits)
    let ff = FF1::<10>::new(key, 2).map_err(|e| {
        FpeError::EncryptionFailed(format!("Failed to create FF1 cipher: {}", e))
    })?;

    // Encrypt with empty tweak (can be customized per use case)
    let tweak = [];
    let ciphertext = ff.encrypt(&tweak, &plaintext).map_err(|e| {
        FpeError::EncryptionFailed(format!("FF1 encryption failed: {}", e))
    })?;

    // Convert back to digit string
    let encrypted_bytes = ciphertext.to_bytes_le();
    String::from_utf8(encrypted_bytes)
        .map_err(|e| FpeError::EncryptionFailed(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        // Test key (in production, use secure random key)
        *b"0123456789abcdef0123456789abcdef"
    }

    #[test]
    fn test_phone_format_detection() {
        assert!(matches!(
            detect_phone_format("555-123-4567"),
            PhoneFormat::Dashes
        ));
        assert!(matches!(
            detect_phone_format("(555) 123-4567"),
            PhoneFormat::Parentheses
        ));
        assert!(matches!(
            detect_phone_format("555.123.4567"),
            PhoneFormat::Dots
        ));
        assert!(matches!(
            detect_phone_format("5551234567"),
            PhoneFormat::Plain
        ));
    }

    #[test]
    fn test_phone_fpe_preserves_format_dashes() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("555-123-4567", EntityType::Phone, &key, &config).unwrap();

        // Should preserve dashes and area code
        assert!(result.contains('-'));
        assert!(result.starts_with("555-"));
        assert_eq!(result.len(), "555-123-4567".len());
    }

    #[test]
    fn test_phone_fpe_preserves_format_parentheses() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("(555) 123-4567", EntityType::Phone, &key, &config).unwrap();

        // Should preserve parentheses and area code
        assert!(result.starts_with("(555)"));
        assert!(result.contains('-'));
        assert_eq!(result.len(), "(555) 123-4567".len());
    }

    #[test]
    fn test_phone_fpe_preserves_format_dots() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("555.123.4567", EntityType::Phone, &key, &config).unwrap();

        // Should preserve dots and area code
        assert!(result.contains('.'));
        assert!(result.starts_with("555."));
        assert_eq!(result.len(), "555.123.4567".len());
    }

    #[test]
    fn test_phone_fpe_plain_format() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("5551234567", EntityType::Phone, &key, &config).unwrap();

        // Should preserve plain format and area code
        assert!(!result.contains('-'));
        assert!(result.starts_with("555"));
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_phone_fpe_without_area_code_preservation() {
        let key = test_key();
        let config = PreserveConfig {
            preserve_area_code: false,
            preserve_last_four: false,
        };
        let result = fpe_encrypt("555-123-4567", EntityType::Phone, &key, &config).unwrap();

        // Should not preserve area code but should preserve format
        assert!(result.contains('-'));
        // Area code should be different
        assert_ne!(&result[0..3], "555");
    }

    #[test]
    fn test_phone_fpe_determinism() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("555-123-4567", EntityType::Phone, &key, &config).unwrap();
        let result2 = fpe_encrypt("555-123-4567", EntityType::Phone, &key, &config).unwrap();

        // Same input should produce same output
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_phone_fpe_uniqueness() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("555-123-4567", EntityType::Phone, &key, &config).unwrap();
        let result2 = fpe_encrypt("555-987-6543", EntityType::Phone, &key, &config).unwrap();

        // Different inputs should produce different outputs (with high probability)
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_phone_fpe_invalid_length() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123-4567", EntityType::Phone, &key, &config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FpeError::InvalidFormat(_)));
    }

    #[test]
    fn test_ssn_fpe_preserves_format_dashes() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123-45-6789", EntityType::Ssn, &key, &config).unwrap();

        // Should preserve dashes and last 4
        assert!(result.contains('-'));
        assert!(result.ends_with("6789"));
        assert_eq!(result.len(), "123-45-6789".len());
    }

    #[test]
    fn test_ssn_fpe_plain_format() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123456789", EntityType::Ssn, &key, &config).unwrap();

        // Should preserve plain format and last 4
        assert!(!result.contains('-'));
        assert!(result.ends_with("6789"));
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn test_ssn_fpe_without_last_four_preservation() {
        let key = test_key();
        let config = PreserveConfig {
            preserve_area_code: false,
            preserve_last_four: false,
        };
        let result = fpe_encrypt("123-45-6789", EntityType::Ssn, &key, &config).unwrap();

        // Should not preserve last 4 but should preserve format
        assert!(result.contains('-'));
        // Last 4 should be different
        assert_ne!(&result[7..11], "6789");
    }

    #[test]
    fn test_ssn_fpe_determinism() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("123-45-6789", EntityType::Ssn, &key, &config).unwrap();
        let result2 = fpe_encrypt("123-45-6789", EntityType::Ssn, &key, &config).unwrap();

        // Same input should produce same output
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_ssn_fpe_uniqueness() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("123-45-6789", EntityType::Ssn, &key, &config).unwrap();
        let result2 = fpe_encrypt("987-65-4321", EntityType::Ssn, &key, &config).unwrap();

        // Different inputs should produce different outputs
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_ssn_fpe_invalid_length() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123-45-67", EntityType::Ssn, &key, &config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FpeError::InvalidFormat(_)));
    }

    #[test]
    fn test_fpe_unsupported_entity_type() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("test@example.com", EntityType::Email, &key, &config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FpeError::UnsupportedType(_)));
    }

    #[test]
    fn test_encrypt_digits() {
        let key = test_key();
        let result1 = encrypt_digits("1234567", &key).unwrap();
        let result2 = encrypt_digits("1234567", &key).unwrap();

        // Determinism
        assert_eq!(result1, result2);

        // Length preservation
        assert_eq!(result1.len(), 7);

        // Only digits
        assert!(result1.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_apply_phone_format() {
        let digits = "1234567890";

        assert_eq!(
            apply_phone_format(digits, &PhoneFormat::Dashes),
            "123-456-7890"
        );
        assert_eq!(
            apply_phone_format(digits, &PhoneFormat::Parentheses),
            "(123) 456-7890"
        );
        assert_eq!(
            apply_phone_format(digits, &PhoneFormat::Dots),
            "123.456.7890"
        );
        assert_eq!(apply_phone_format(digits, &PhoneFormat::Plain), "1234567890");
    }

    #[test]
    fn test_edge_case_phone_with_spaces() {
        let key = test_key();
        let config = PreserveConfig::default();
        // Input with extra spaces (digits extracted)
        let result = fpe_encrypt("555 123 4567", EntityType::Phone, &key, &config).unwrap();

        // Should work (spaces ignored during digit extraction)
        // Format detection falls back to plain
        assert_eq!(result.len(), 10);
        assert!(result.starts_with("555"));
    }

    #[test]
    fn test_edge_case_ssn_with_spaces() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123 45 6789", EntityType::Ssn, &key, &config).unwrap();

        // Should work (spaces ignored)
        assert_eq!(result.len(), 9);
        assert!(result.ends_with("6789"));
    }
}
