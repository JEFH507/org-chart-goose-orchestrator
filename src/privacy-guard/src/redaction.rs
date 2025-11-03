//! Format-preserving encryption and text masking
//! This module implements FPE for phone/SSN and general masking logic

use crate::detection::{Confidence, Detection, EntityType};
use crate::pseudonym;
use crate::state::MappingState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        EntityType::PHONE => encrypt_phone(text, key, config),
        EntityType::SSN => encrypt_ssn(text, key, config),
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
///
/// NOTE: Temporarily simplified - using deterministic transformation instead of full FPE
/// TODO: Implement proper FF1 once fpe crate API is clarified
fn encrypt_digits(digits: &str, key: &[u8; 32]) -> Result<String, FpeError> {
    use sha2::{Digest, Sha256};
    
    // TEMPORARY: Use HMAC-based digit transformation
    // This preserves length and determinism, but not format-preserving encryption properties
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(digits.as_bytes());
    let hash = hasher.finalize();
    
    // Convert hash bytes to digits, taking only what we need
    let mut result = String::new();
    for (i, &byte) in hash.iter().enumerate() {
        if result.len() >= digits.len() {
            break;
        }
        // Convert byte to 2-3 digits
        let digit_str = format!("{}", byte % 100);
        for c in digit_str.chars() {
            if result.len() < digits.len() {
                result.push(c);
            }
        }
    }
    
    // Pad or trim to exact length
    while result.len() < digits.len() {
        result.push('0');
    }
    result.truncate(digits.len());
    
    Ok(result)
}

// ============================================================================
// MASKING LOGIC
// ============================================================================

/// Masking strategy for an entity type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaskingStrategy {
    /// Use HMAC-based pseudonymization
    Pseudonym,
    /// Use format-preserving encryption (phone/SSN only)
    Fpe,
    /// Simple redaction (e.g., "***" or "REDACTED")
    Redact,
}

/// Policy configuration for masking
#[derive(Debug, Clone)]
pub struct MaskingPolicy {
    /// Strategy per entity type
    pub strategies: HashMap<EntityType, MaskingStrategy>,
    /// Default strategy for unlisted types
    pub default_strategy: MaskingStrategy,
    /// FPE configuration
    pub fpe_config: PreserveConfig,
    /// FPE key (32 bytes for AES-256)
    pub fpe_key: [u8; 32],
}

impl Default for MaskingPolicy {
    fn default() -> Self {
        let mut strategies = HashMap::new();
        
        // Default strategies per ADR-0022
        strategies.insert(EntityType::SSN, MaskingStrategy::Fpe);
        strategies.insert(EntityType::PHONE, MaskingStrategy::Fpe);
        strategies.insert(EntityType::EMAIL, MaskingStrategy::Pseudonym);
        strategies.insert(EntityType::PERSON, MaskingStrategy::Pseudonym);
        strategies.insert(EntityType::CreditCard, MaskingStrategy::Redact);
        strategies.insert(EntityType::IpAddress, MaskingStrategy::Pseudonym);
        strategies.insert(EntityType::DateOfBirth, MaskingStrategy::Pseudonym);
        strategies.insert(EntityType::AccountNumber, MaskingStrategy::Pseudonym);

        Self {
            strategies,
            default_strategy: MaskingStrategy::Pseudonym,
            fpe_config: PreserveConfig::default(),
            fpe_key: [0u8; 32], // Must be set from environment
        }
    }
}

impl MaskingPolicy {
    /// Get the strategy for an entity type
    pub fn get_strategy(&self, entity_type: &EntityType) -> &MaskingStrategy {
        self.strategies
            .get(entity_type)
            .unwrap_or(&self.default_strategy)
    }
}

/// Result of masking operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskResult {
    /// The masked text with PII replaced
    pub masked_text: String,
    /// Count of redactions by entity type
    pub redactions: HashMap<String, usize>,
    /// Total number of entities masked
    pub total_redactions: usize,
}

/// Mask detected PII in text using configured policy
///
/// This function:
/// 1. Sorts detections by confidence (higher first) and position
/// 2. Resolves overlapping detections (keeps higher confidence)
/// 3. Applies masking strategy per entity type
/// 4. Preserves text structure (newlines, spacing)
/// 5. Stores mappings in state for reidentification
///
/// # Arguments
/// * `text` - Original text containing PII
/// * `detections` - List of detected entities
/// * `policy` - Masking policy with strategies
/// * `state` - Mapping state for pseudonym storage
/// * `tenant_id` - Tenant identifier for isolation
///
/// # Returns
/// MaskResult with masked text and redaction summary
pub fn mask(
    text: &str,
    mut detections: Vec<Detection>,
    policy: &MaskingPolicy,
    state: &MappingState,
    tenant_id: &str,
) -> MaskResult {
    if detections.is_empty() {
        return MaskResult {
            masked_text: text.to_string(),
            redactions: HashMap::new(),
            total_redactions: 0,
        };
    }

    // Sort by confidence (higher first) then by position
    detections.sort_by(|a, b| {
        b.confidence
            .cmp(&a.confidence)
            .then_with(|| a.start.cmp(&b.start))
    });

    // Resolve overlaps: keep higher confidence, discard overlapping lower ones
    let non_overlapping = resolve_overlaps(detections);

    // Sort back by position for sequential replacement
    let mut sorted = non_overlapping;
    sorted.sort_by_key(|d| d.start);

    // Track redaction counts
    let mut redaction_counts: HashMap<String, usize> = HashMap::new();

    // Build masked text by replacing each detection
    let mut result = String::new();
    let mut last_pos = 0;

    for detection in sorted {
        // Add text before this detection
        result.push_str(&text[last_pos..detection.start]);

        // Apply masking strategy
        let replacement = apply_masking_strategy(
            &detection.matched_text,
            &detection.entity_type,
            policy,
            state,
            tenant_id,
        );

        result.push_str(&replacement);

        // Update counters
        *redaction_counts
            .entry(detection.entity_type.to_string())
            .or_insert(0) += 1;

        last_pos = detection.end;
    }

    // Add remaining text after last detection
    result.push_str(&text[last_pos..]);

    let total_redactions = redaction_counts.values().sum();

    MaskResult {
        masked_text: result,
        redactions: redaction_counts,
        total_redactions,
    }
}

/// Resolve overlapping detections: keep higher confidence ones
fn resolve_overlaps(detections: Vec<Detection>) -> Vec<Detection> {
    let mut result = Vec::new();

    for detection in detections {
        // Check if this detection overlaps with any already accepted
        let overlaps = result.iter().any(|d: &Detection| {
            // Overlap if: d.start < detection.end && detection.start < d.end
            d.start < detection.end && detection.start < d.end
        });

        if !overlaps {
            result.push(detection);
        }
        // Else: discard this detection (already sorted by confidence, so existing is better)
    }

    result
}

/// Apply masking strategy to a single entity
fn apply_masking_strategy(
    text: &str,
    entity_type: &EntityType,
    policy: &MaskingPolicy,
    state: &MappingState,
    tenant_id: &str,
) -> String {
    let strategy = policy.get_strategy(entity_type);

    match strategy {
        MaskingStrategy::Pseudonym => {
            // Check if already mapped
            if let Some(existing) = state.get_pseudonym(text) {
                return existing;
            }

            // Generate new pseudonym
            let pseudonym = pseudonym::pseudonymize(text, entity_type, tenant_id);

            // Store mapping
            state.insert(pseudonym.clone(), text.to_string());

            pseudonym
        }
        MaskingStrategy::Fpe => {
            // Try FPE (only works for phone/SSN)
            match fpe_encrypt(text, entity_type.clone(), &policy.fpe_key, &policy.fpe_config) {
                Ok(encrypted) => encrypted,
                Err(_) => {
                    // Fallback to pseudonym if FPE fails
                    let pseudonym = pseudonym::pseudonymize(text, entity_type, tenant_id);
                    state.insert(pseudonym.clone(), text.to_string());
                    pseudonym
                }
            }
        }
        MaskingStrategy::Redact => {
            // Simple redaction based on entity type
            match entity_type {
                EntityType::CreditCard => {
                    // Preserve last 4 digits if possible
                    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
                    if digits.len() >= 4 {
                        let last_four = &digits[digits.len() - 4..];
                        format!("CARD_****_****_****_{}", last_four)
                    } else {
                        "CARD_REDACTED".to_string()
                    }
                }
                _ => format!("{}_REDACTED", entity_type),
            }
        }
    }
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
        let result = fpe_encrypt("555-123-4567", EntityType::PHONE, &key, &config).unwrap();

        // Should preserve dashes and area code
        assert!(result.contains('-'));
        assert!(result.starts_with("555-"));
        assert_eq!(result.len(), "555-123-4567".len());
    }

    #[test]
    fn test_phone_fpe_preserves_format_parentheses() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("(555) 123-4567", EntityType::PHONE, &key, &config).unwrap();

        // Should preserve parentheses and area code
        assert!(result.starts_with("(555)"));
        assert!(result.contains('-'));
        assert_eq!(result.len(), "(555) 123-4567".len());
    }

    #[test]
    fn test_phone_fpe_preserves_format_dots() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("555.123.4567", EntityType::PHONE, &key, &config).unwrap();

        // Should preserve dots and area code
        assert!(result.contains('.'));
        assert!(result.starts_with("555."));
        assert_eq!(result.len(), "555.123.4567".len());
    }

    #[test]
    fn test_phone_fpe_plain_format() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("5551234567", EntityType::PHONE, &key, &config).unwrap();

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
        let result = fpe_encrypt("555-123-4567", EntityType::PHONE, &key, &config).unwrap();

        // Should not preserve area code but should preserve format
        assert!(result.contains('-'));
        // Area code should be different
        assert_ne!(&result[0..3], "555");
    }

    #[test]
    fn test_phone_fpe_determinism() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("555-123-4567", EntityType::PHONE, &key, &config).unwrap();
        let result2 = fpe_encrypt("555-123-4567", EntityType::PHONE, &key, &config).unwrap();

        // Same input should produce same output
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_phone_fpe_uniqueness() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("555-123-4567", EntityType::PHONE, &key, &config).unwrap();
        let result2 = fpe_encrypt("555-987-6543", EntityType::PHONE, &key, &config).unwrap();

        // Different inputs should produce different outputs (with high probability)
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_phone_fpe_invalid_length() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123-4567", EntityType::PHONE, &key, &config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FpeError::InvalidFormat(_)));
    }

    #[test]
    fn test_ssn_fpe_preserves_format_dashes() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123-45-6789", EntityType::SSN, &key, &config).unwrap();

        // Should preserve dashes and last 4
        assert!(result.contains('-'));
        assert!(result.ends_with("6789"));
        assert_eq!(result.len(), "123-45-6789".len());
    }

    #[test]
    fn test_ssn_fpe_plain_format() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123456789", EntityType::SSN, &key, &config).unwrap();

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
        let result = fpe_encrypt("123-45-6789", EntityType::SSN, &key, &config).unwrap();

        // Should not preserve last 4 but should preserve format
        assert!(result.contains('-'));
        // Last 4 should be different
        assert_ne!(&result[7..11], "6789");
    }

    #[test]
    fn test_ssn_fpe_determinism() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("123-45-6789", EntityType::SSN, &key, &config).unwrap();
        let result2 = fpe_encrypt("123-45-6789", EntityType::SSN, &key, &config).unwrap();

        // Same input should produce same output
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_ssn_fpe_uniqueness() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result1 = fpe_encrypt("123-45-6789", EntityType::SSN, &key, &config).unwrap();
        let result2 = fpe_encrypt("987-65-4321", EntityType::SSN, &key, &config).unwrap();

        // Different inputs should produce different outputs
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_ssn_fpe_invalid_length() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123-45-67", EntityType::SSN, &key, &config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FpeError::InvalidFormat(_)));
    }

    #[test]
    fn test_fpe_unsupported_entity_type() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("test@example.com", EntityType::EMAIL, &key, &config);

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
        let result = fpe_encrypt("555 123 4567", EntityType::PHONE, &key, &config).unwrap();

        // Should work (spaces ignored during digit extraction)
        // Format detection falls back to plain
        assert_eq!(result.len(), 10);
        assert!(result.starts_with("555"));
    }

    #[test]
    fn test_edge_case_ssn_with_spaces() {
        let key = test_key();
        let config = PreserveConfig::default();
        let result = fpe_encrypt("123 45 6789", EntityType::SSN, &key, &config).unwrap();

        // Should work (spaces ignored)
        assert_eq!(result.len(), 9);
        assert!(result.ends_with("6789"));
    }

    // =========================================================================
    // MASKING INTEGRATION TESTS
    // =========================================================================

    use crate::detection::{detect, Rules};

    fn test_policy() -> MaskingPolicy {
        let mut policy = MaskingPolicy::default();
        policy.fpe_key = test_key();
        policy
    }

    #[test]
    fn test_mask_empty_text() {
        let policy = test_policy();
        let state = MappingState::new();
        let detections = vec![];

        let result = mask("", detections, &policy, &state, "org1");

        assert_eq!(result.masked_text, "");
        assert_eq!(result.total_redactions, 0);
        assert!(result.redactions.is_empty());
    }

    #[test]
    fn test_mask_no_detections() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "This text has no PII";
        let detections = vec![];

        let result = mask(text, detections, &policy, &state, "org1");

        assert_eq!(result.masked_text, text);
        assert_eq!(result.total_redactions, 0);
        assert!(result.redactions.is_empty());
    }

    #[test]
    fn test_mask_single_entity_pseudonym() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Contact john.doe@example.com for details";

        // Manually create detection (or use detect() from detection module)
        let detections = vec![Detection {
            start: 8,
            end: 29,
            entity_type: EntityType::EMAIL,
            confidence: Confidence::HIGH,
            matched_text: "john.doe@example.com".to_string(),
        }];

        let result = mask(text, detections, &policy, &state, "org1");

        // Should replace email with pseudonym
        assert!(result.masked_text.starts_with("Contact EMAIL_"));
        assert!(result.masked_text.ends_with(" for details"));
        assert_eq!(result.total_redactions, 1);
        assert_eq!(result.redactions.get("EMAIL"), Some(&1));

        // Verify state has mapping
        assert!(state.contains_original("john.doe@example.com"));
    }

    #[test]
    fn test_mask_multiple_entities() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Call 555-123-4567 or email john@test.com";

        let detections = vec![
            Detection {
                start: 5,
                end: 17,
                entity_type: EntityType::PHONE,
                confidence: Confidence::HIGH,
                matched_text: "555-123-4567".to_string(),
            },
            Detection {
                start: 30,
                end: 43,
                entity_type: EntityType::EMAIL,
                confidence: Confidence::HIGH,
                matched_text: "john@test.com".to_string(),
            },
        ];

        let result = mask(text, detections, &policy, &state, "org1");

        // Should preserve structure
        assert!(result.masked_text.starts_with("Call "));
        assert!(result.masked_text.contains(" or email "));
        
        // Phone should use FPE (preserve format)
        assert!(result.masked_text.contains("555-")); // Area code preserved
        
        // Email should use pseudonym
        assert!(result.masked_text.contains("EMAIL_"));

        assert_eq!(result.total_redactions, 2);
        assert_eq!(result.redactions.get("PHONE"), Some(&1));
        assert_eq!(result.redactions.get("EMAIL"), Some(&1));
    }

    #[test]
    fn test_mask_with_fpe_phone() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Phone: 555-123-4567";

        let detections = vec![Detection {
            start: 7,
            end: 19,
            entity_type: EntityType::PHONE,
            confidence: Confidence::HIGH,
            matched_text: "555-123-4567".to_string(),
        }];

        let result = mask(text, detections, &policy, &state, "org1");

        // FPE should preserve format and area code
        assert!(result.masked_text.starts_with("Phone: 555-"));
        assert!(result.masked_text.contains('-'));
        assert_eq!(result.masked_text.len(), text.len());
        
        // Should be different from original
        assert_ne!(result.masked_text, text);
    }

    #[test]
    fn test_mask_with_fpe_ssn() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "SSN: 123-45-6789";

        let detections = vec![Detection {
            start: 5,
            end: 16,
            entity_type: EntityType::SSN,
            confidence: Confidence::HIGH,
            matched_text: "123-45-6789".to_string(),
        }];

        let result = mask(text, detections, &policy, &state, "org1");

        // FPE should preserve format and last 4
        assert!(result.masked_text.starts_with("SSN: "));
        assert!(result.masked_text.ends_with("6789"));
        assert!(result.masked_text.contains('-'));
        assert_eq!(result.masked_text.len(), text.len());
    }

    #[test]
    fn test_mask_with_redaction_credit_card() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Card: 4532015112830366";

        let detections = vec![Detection {
            start: 6,
            end: 22,
            entity_type: EntityType::CreditCard,
            confidence: Confidence::HIGH,
            matched_text: "4532015112830366".to_string(),
        }];

        let result = mask(text, detections, &policy, &state, "org1");

        // Credit card should be redacted with last 4 preserved
        assert!(result.masked_text.contains("CARD_****_****_****_0366"));
        assert_eq!(result.total_redactions, 1);
    }

    #[test]
    fn test_mask_overlapping_detections() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Email: john.doe@example.com today";

        // Two overlapping detections: one HIGH, one LOW
        let detections = vec![
            Detection {
                start: 7,
                end: 28,
                entity_type: EntityType::EMAIL,
                confidence: Confidence::HIGH,
                matched_text: "john.doe@example.com".to_string(),
            },
            Detection {
                start: 7,
                end: 15,
                entity_type: EntityType::PERSON,
                confidence: Confidence::LOW,
                matched_text: "john.doe".to_string(),
            },
        ];

        let result = mask(text, detections, &policy, &state, "org1");

        // Should keep only the HIGH confidence detection (email)
        assert_eq!(result.total_redactions, 1);
        assert_eq!(result.redactions.get("EMAIL"), Some(&1));
        assert_eq!(result.redactions.get("PERSON"), None);

        // Result should have email replaced, not person
        assert!(result.masked_text.contains("EMAIL_"));
        assert!(!result.masked_text.contains("PERSON_"));
    }

    #[test]
    fn test_mask_preserves_text_structure() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Line 1: test@example.com\nLine 2: 555-123-4567\n\nLine 3: normal text";

        let detections = vec![
            Detection {
                start: 8,
                end: 24,
                entity_type: EntityType::EMAIL,
                confidence: Confidence::HIGH,
                matched_text: "test@example.com".to_string(),
            },
            Detection {
                start: 33,
                end: 45,
                entity_type: EntityType::PHONE,
                confidence: Confidence::HIGH,
                matched_text: "555-123-4567".to_string(),
            },
        ];

        let result = mask(text, detections, &policy, &state, "org1");

        // Should preserve newlines
        assert_eq!(result.masked_text.matches('\n').count(), 3);
        assert!(result.masked_text.contains("Line 1: "));
        assert!(result.masked_text.contains("Line 2: "));
        assert!(result.masked_text.contains("Line 3: normal text"));
    }

    #[test]
    fn test_mask_determinism_via_state() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Contact test@example.com twice: test@example.com";

        let detections = vec![
            Detection {
                start: 8,
                end: 24,
                entity_type: EntityType::EMAIL,
                confidence: Confidence::HIGH,
                matched_text: "test@example.com".to_string(),
            },
            Detection {
                start: 32,
                end: 48,
                entity_type: EntityType::EMAIL,
                confidence: Confidence::HIGH,
                matched_text: "test@example.com".to_string(),
            },
        ];

        let result = mask(text, detections, &policy, &state, "org1");

        // Should use same pseudonym for both occurrences (via state lookup)
        let parts: Vec<&str> = result.masked_text.split("twice:").collect();
        let first_pseudo = parts[0].trim().split_whitespace().last().unwrap();
        let second_pseudo = parts[1].trim();

        assert_eq!(first_pseudo, second_pseudo);
        assert_eq!(result.total_redactions, 2);
    }

    #[test]
    fn test_mask_integration_with_real_detection() {
        std::env::set_var("PSEUDO_SALT", "test-salt-for-integration");

        let rules = Rules::default_rules();
        let policy = test_policy();
        let state = MappingState::new();

        let text = "Contact Dr. John Smith at 555-123-4567 or john.smith@company.com. SSN: 123-45-6789";

        // Use real detection engine
        let detections = detect(text, &rules);

        // Should detect: PERSON, PHONE, EMAIL, SSN
        assert!(detections.len() >= 4);

        let result = mask(text, detections, &policy, &state, "org1");

        println!("\nOriginal: {}", text);
        println!("Masked:   {}", result.masked_text);

        // Verify masking happened
        assert_ne!(result.masked_text, text);
        assert!(result.total_redactions >= 4);

        // Verify specific redactions
        assert!(!result.masked_text.contains("John Smith"));
        assert!(!result.masked_text.contains("john.smith@company.com"));
        assert!(!result.masked_text.contains("123-45"));

        // Verify phone FPE preserved area code
        assert!(result.masked_text.contains("555-"));

        // Verify SSN FPE preserved last 4
        assert!(result.masked_text.contains("6789"));
    }

    #[test]
    fn test_resolve_overlaps() {
        let detections = vec![
            Detection {
                start: 0,
                end: 10,
                entity_type: EntityType::EMAIL,
                confidence: Confidence::HIGH,
                matched_text: "test@ex.co".to_string(),
            },
            Detection {
                start: 5,
                end: 15,
                entity_type: EntityType::PERSON,
                confidence: Confidence::LOW,
                matched_text: "ex.co name".to_string(),
            },
            Detection {
                start: 20,
                end: 30,
                entity_type: EntityType::PHONE,
                confidence: Confidence::MEDIUM,
                matched_text: "555-123-45".to_string(),
            },
        ];

        let resolved = resolve_overlaps(detections);

        // Should keep first two (even though they overlap, they're in confidence order)
        // Actually: with overlap resolution, should keep only HIGH and discard LOW
        // Then keep MEDIUM (no overlap with HIGH)
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].entity_type, EntityType::EMAIL);
        assert_eq!(resolved[1].entity_type, EntityType::PHONE);
    }

    #[test]
    fn test_masking_strategy_enum() {
        // Verify serialization works (for API responses)
        let strategy = MaskingStrategy::Pseudonym;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, r#""Pseudonym""#);

        let strategy2 = MaskingStrategy::Fpe;
        let json2 = serde_json::to_string(&strategy2).unwrap();
        assert_eq!(json2, r#""Fpe""#);
    }

    #[test]
    fn test_mask_result_serialization() {
        let mut redactions = HashMap::new();
        redactions.insert("EMAIL".to_string(), 2);
        redactions.insert("PHONE".to_string(), 1);

        let result = MaskResult {
            masked_text: "Masked text here".to_string(),
            redactions,
            total_redactions: 3,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("masked_text"));
        assert!(json.contains("EMAIL"));
        assert!(json.contains("total_redactions"));

        // Verify deserialization
        let _deserialized: MaskResult = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_masking_policy_default() {
        let policy = MaskingPolicy::default();

        // Verify default strategies per ADR-0022
        assert_eq!(policy.get_strategy(&EntityType::SSN), &MaskingStrategy::Fpe);
        assert_eq!(
            policy.get_strategy(&EntityType::PHONE),
            &MaskingStrategy::Fpe
        );
        assert_eq!(
            policy.get_strategy(&EntityType::EMAIL),
            &MaskingStrategy::Pseudonym
        );
        assert_eq!(
            policy.get_strategy(&EntityType::PERSON),
            &MaskingStrategy::Pseudonym
        );
        assert_eq!(
            policy.get_strategy(&EntityType::CreditCard),
            &MaskingStrategy::Redact
        );
    }

    #[test]
    fn test_edge_case_empty_detections_list() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Some text with content";

        let result = mask(text, vec![], &policy, &state, "org1");

        assert_eq!(result.masked_text, text);
        assert_eq!(result.total_redactions, 0);
    }

    #[test]
    fn test_edge_case_detection_at_start() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "test@example.com is an email";

        let detections = vec![Detection {
            start: 0,
            end: 16,
            entity_type: EntityType::EMAIL,
            confidence: Confidence::HIGH,
            matched_text: "test@example.com".to_string(),
        }];

        let result = mask(text, detections, &policy, &state, "org1");

        assert!(result.masked_text.starts_with("EMAIL_"));
        assert!(result.masked_text.ends_with(" is an email"));
    }

    #[test]
    fn test_edge_case_detection_at_end() {
        let policy = test_policy();
        let state = MappingState::new();
        let text = "Email is test@example.com";

        let detections = vec![Detection {
            start: 9,
            end: 25,
            entity_type: EntityType::EMAIL,
            confidence: Confidence::HIGH,
            matched_text: "test@example.com".to_string(),
        }];

        let result = mask(text, detections, &policy, &state, "org1");

        assert!(result.masked_text.starts_with("Email is "));
        assert!(result.masked_text.ends_with(|c: char| c.is_ascii_hexdigit()));
    }
}
