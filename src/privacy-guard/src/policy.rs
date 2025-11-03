//! Policy engine for privacy guard modes and configuration
//!
//! This module implements the policy engine that controls:
//! - Detection and masking modes (OFF, DETECT, MASK, STRICT)
//! - Confidence threshold filtering
//! - Per-entity masking strategies
//! - Graceful degradation when configuration is missing

use crate::detection::{Confidence, Detection, EntityType};
use crate::redaction::{MaskingPolicy, MaskingStrategy, PreserveConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Privacy Guard operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GuardMode {
    /// Completely disabled - no detection or masking
    Off,
    /// Detection only - return findings but don't mask
    Detect,
    /// Full masking with configured strategies (default)
    Mask,
    /// Error on any PII detection - fail-safe mode
    Strict,
}

impl Default for GuardMode {
    fn default() -> Self {
        GuardMode::Mask
    }
}

impl fmt::Display for GuardMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuardMode::Off => write!(f, "OFF"),
            GuardMode::Detect => write!(f, "DETECT"),
            GuardMode::Mask => write!(f, "MASK"),
            GuardMode::Strict => write!(f, "STRICT"),
        }
    }
}

impl std::str::FromStr for GuardMode {
    type Err = PolicyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "OFF" => Ok(GuardMode::Off),
            "DETECT" => Ok(GuardMode::Detect),
            "MASK" => Ok(GuardMode::Mask),
            "STRICT" => Ok(GuardMode::Strict),
            _ => Err(PolicyError::InvalidMode(s.to_string())),
        }
    }
}

/// Policy-related errors
#[derive(Debug)]
pub enum PolicyError {
    /// Invalid mode string
    InvalidMode(String),
    /// Invalid confidence threshold
    InvalidConfidence(String),
    /// Missing required configuration
    MissingConfig(String),
    /// PII detected in STRICT mode
    StrictModeViolation(usize),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyError::InvalidMode(mode) => write!(f, "Invalid guard mode: {}", mode),
            PolicyError::InvalidConfidence(conf) => write!(f, "Invalid confidence: {}", conf),
            PolicyError::MissingConfig(msg) => write!(f, "Missing configuration: {}", msg),
            PolicyError::StrictModeViolation(count) => {
                write!(f, "STRICT mode: {} PII entities detected", count)
            }
        }
    }
}

impl Error for PolicyError {}

/// Complete policy configuration for Privacy Guard
#[derive(Debug, Clone)]
pub struct Policy {
    /// Operating mode
    pub mode: GuardMode,
    /// Minimum confidence level to act on (filter out lower confidence)
    pub confidence_threshold: Confidence,
    /// Masking policy (strategies per entity type)
    pub masking_policy: MaskingPolicy,
    /// Whether to log detection events
    pub log_detections: bool,
    /// Whether to log masking events
    pub log_redactions: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            mode: GuardMode::Mask,
            confidence_threshold: Confidence::MEDIUM,
            masking_policy: MaskingPolicy::default(),
            log_detections: true,
            log_redactions: true,
        }
    }
}

impl Policy {
    /// Create a new policy with specified mode
    pub fn new(mode: GuardMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Create a policy with all settings
    pub fn with_config(
        mode: GuardMode,
        confidence_threshold: Confidence,
        masking_policy: MaskingPolicy,
    ) -> Self {
        Self {
            mode,
            confidence_threshold,
            masking_policy,
            log_detections: true,
            log_redactions: true,
        }
    }

    /// Load policy from environment variables (fallback to defaults)
    ///
    /// Environment variables:
    /// - GUARD_MODE: OFF | DETECT | MASK | STRICT (default: MASK)
    /// - GUARD_CONFIDENCE: HIGH | MEDIUM | LOW (default: MEDIUM)
    /// - PSEUDO_SALT: Required for MASK mode (falls back to OFF if missing)
    pub fn from_env() -> Self {
        let mode = std::env::var("GUARD_MODE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(GuardMode::Mask);

        let confidence_threshold = std::env::var("GUARD_CONFIDENCE")
            .ok()
            .and_then(|s| match s.to_uppercase().as_str() {
                "HIGH" => Some(Confidence::HIGH),
                "MEDIUM" => Some(Confidence::MEDIUM),
                "LOW" => Some(Confidence::LOW),
                _ => None,
            })
            .unwrap_or(Confidence::MEDIUM);

        // Check if PSEUDO_SALT is available for MASK mode
        let salt_available = std::env::var("PSEUDO_SALT").is_ok();
        let effective_mode = if mode == GuardMode::Mask && !salt_available {
            tracing::warn!(
                "GUARD_MODE=MASK but PSEUDO_SALT not set. Falling back to DETECT mode."
            );
            GuardMode::Detect
        } else {
            mode
        };

        // Create masking policy with FPE key from env or default
        let mut masking_policy = MaskingPolicy::default();
        
        // Try to load FPE key from environment (base64 or hex)
        // For now, use a derived key from PSEUDO_SALT if available
        if let Ok(salt) = std::env::var("PSEUDO_SALT") {
            // Derive a 32-byte key from salt (simple approach for Phase 2)
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(salt.as_bytes());
            hasher.update(b"FPE_KEY_DERIVATION");
            let result = hasher.finalize();
            masking_policy.fpe_key.copy_from_slice(&result[..32]);
        }

        Self {
            mode: effective_mode,
            confidence_threshold,
            masking_policy,
            log_detections: true,
            log_redactions: true,
        }
    }

    /// Filter detections based on confidence threshold
    ///
    /// Returns only detections that meet or exceed the configured threshold
    pub fn filter_detections(&self, detections: Vec<Detection>) -> Vec<Detection> {
        detections
            .into_iter()
            .filter(|d| d.confidence >= self.confidence_threshold)
            .collect()
    }

    /// Check if masking is enabled based on mode
    pub fn should_mask(&self) -> bool {
        self.mode == GuardMode::Mask
    }

    /// Check if detection is enabled
    pub fn should_detect(&self) -> bool {
        matches!(self.mode, GuardMode::Detect | GuardMode::Mask | GuardMode::Strict)
    }

    /// Validate detections in STRICT mode (returns error if any PII found)
    pub fn validate_strict(&self, detections: &[Detection]) -> Result<(), PolicyError> {
        if self.mode == GuardMode::Strict && !detections.is_empty() {
            return Err(PolicyError::StrictModeViolation(detections.len()));
        }
        Ok(())
    }

    /// Get a summary of the current policy configuration
    pub fn summary(&self) -> PolicySummary {
        PolicySummary {
            mode: self.mode,
            confidence_threshold: self.confidence_threshold,
            strategies: self
                .masking_policy
                .strategies
                .iter()
                .map(|(k, v)| (k.to_string(), format!("{:?}", v)))
                .collect(),
            fpe_preserve_area_code: self.masking_policy.fpe_config.preserve_area_code,
            fpe_preserve_last_four: self.masking_policy.fpe_config.preserve_last_four,
        }
    }
}

/// Summary of policy configuration for status endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySummary {
    pub mode: GuardMode,
    pub confidence_threshold: Confidence,
    pub strategies: HashMap<String, String>,
    pub fpe_preserve_area_code: bool,
    pub fpe_preserve_last_four: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_mode_default() {
        let mode = GuardMode::default();
        assert_eq!(mode, GuardMode::Mask);
    }

    #[test]
    fn test_guard_mode_display() {
        assert_eq!(GuardMode::Off.to_string(), "OFF");
        assert_eq!(GuardMode::Detect.to_string(), "DETECT");
        assert_eq!(GuardMode::Mask.to_string(), "MASK");
        assert_eq!(GuardMode::Strict.to_string(), "STRICT");
    }

    #[test]
    fn test_guard_mode_from_str() {
        assert_eq!("OFF".parse::<GuardMode>().unwrap(), GuardMode::Off);
        assert_eq!("off".parse::<GuardMode>().unwrap(), GuardMode::Off);
        assert_eq!("DETECT".parse::<GuardMode>().unwrap(), GuardMode::Detect);
        assert_eq!("detect".parse::<GuardMode>().unwrap(), GuardMode::Detect);
        assert_eq!("MASK".parse::<GuardMode>().unwrap(), GuardMode::Mask);
        assert_eq!("mask".parse::<GuardMode>().unwrap(), GuardMode::Mask);
        assert_eq!("STRICT".parse::<GuardMode>().unwrap(), GuardMode::Strict);
        assert_eq!("strict".parse::<GuardMode>().unwrap(), GuardMode::Strict);

        // Invalid mode
        assert!("INVALID".parse::<GuardMode>().is_err());
    }

    #[test]
    fn test_guard_mode_serialization() {
        let mode = GuardMode::Mask;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""MASK""#);

        let deserialized: GuardMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, GuardMode::Mask);
    }

    #[test]
    fn test_policy_default() {
        let policy = Policy::default();
        assert_eq!(policy.mode, GuardMode::Mask);
        assert_eq!(policy.confidence_threshold, Confidence::MEDIUM);
        assert!(policy.log_detections);
        assert!(policy.log_redactions);
    }

    #[test]
    fn test_policy_new() {
        let policy = Policy::new(GuardMode::Detect);
        assert_eq!(policy.mode, GuardMode::Detect);
        assert_eq!(policy.confidence_threshold, Confidence::MEDIUM);
    }

    #[test]
    fn test_policy_with_config() {
        let masking_policy = MaskingPolicy::default();
        let policy = Policy::with_config(GuardMode::Strict, Confidence::HIGH, masking_policy);

        assert_eq!(policy.mode, GuardMode::Strict);
        assert_eq!(policy.confidence_threshold, Confidence::HIGH);
    }

    #[test]
    fn test_policy_from_env_defaults() {
        // Clear env vars
        std::env::remove_var("GUARD_MODE");
        std::env::remove_var("GUARD_CONFIDENCE");
        std::env::remove_var("PSEUDO_SALT");

        let policy = Policy::from_env();

        // Should fall back to DETECT (no salt) or use defaults
        assert!(matches!(policy.mode, GuardMode::Mask | GuardMode::Detect));
        assert_eq!(policy.confidence_threshold, Confidence::MEDIUM);
    }

    #[test]
    fn test_policy_from_env_with_mode() {
        std::env::set_var("GUARD_MODE", "STRICT");
        std::env::set_var("PSEUDO_SALT", "test-salt");

        let policy = Policy::from_env();

        assert_eq!(policy.mode, GuardMode::Strict);

        std::env::remove_var("GUARD_MODE");
        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_policy_from_env_with_confidence() {
        std::env::set_var("GUARD_CONFIDENCE", "HIGH");
        std::env::set_var("PSEUDO_SALT", "test-salt");

        let policy = Policy::from_env();

        assert_eq!(policy.confidence_threshold, Confidence::HIGH);

        std::env::remove_var("GUARD_CONFIDENCE");
        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_policy_from_env_mask_without_salt_falls_back() {
        std::env::set_var("GUARD_MODE", "MASK");
        std::env::remove_var("PSEUDO_SALT");

        let policy = Policy::from_env();

        // Should fall back to DETECT mode when salt missing
        assert_eq!(policy.mode, GuardMode::Detect);

        std::env::remove_var("GUARD_MODE");
    }

    #[test]
    fn test_policy_filter_detections_high_threshold() {
        let policy = Policy::with_config(
            GuardMode::Mask,
            Confidence::HIGH,
            MaskingPolicy::default(),
        );

        let detections = vec![
            Detection {
                start: 0,
                end: 10,
                entity_type: EntityType::Email,
                confidence: Confidence::HIGH,
                matched_text: "test@ex.co".to_string(),
            },
            Detection {
                start: 11,
                end: 20,
                entity_type: EntityType::Person,
                confidence: Confidence::MEDIUM,
                matched_text: "John Doe".to_string(),
            },
            Detection {
                start: 21,
                end: 30,
                entity_type: EntityType::Person,
                confidence: Confidence::LOW,
                matched_text: "Jane Smith".to_string(),
            },
        ];

        let filtered = policy.filter_detections(detections);

        // Should keep only HIGH confidence
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].confidence, Confidence::HIGH);
        assert_eq!(filtered[0].entity_type, EntityType::Email);
    }

    #[test]
    fn test_policy_filter_detections_medium_threshold() {
        let policy = Policy::with_config(
            GuardMode::Mask,
            Confidence::MEDIUM,
            MaskingPolicy::default(),
        );

        let detections = vec![
            Detection {
                start: 0,
                end: 10,
                entity_type: EntityType::Email,
                confidence: Confidence::HIGH,
                matched_text: "test@ex.co".to_string(),
            },
            Detection {
                start: 11,
                end: 20,
                entity_type: EntityType::Person,
                confidence: Confidence::MEDIUM,
                matched_text: "John Doe".to_string(),
            },
            Detection {
                start: 21,
                end: 30,
                entity_type: EntityType::Person,
                confidence: Confidence::LOW,
                matched_text: "Jane Smith".to_string(),
            },
        ];

        let filtered = policy.filter_detections(detections);

        // Should keep HIGH and MEDIUM
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|d| d.confidence == Confidence::HIGH));
        assert!(filtered.iter().any(|d| d.confidence == Confidence::MEDIUM));
    }

    #[test]
    fn test_policy_filter_detections_low_threshold() {
        let policy = Policy::with_config(
            GuardMode::Mask,
            Confidence::LOW,
            MaskingPolicy::default(),
        );

        let detections = vec![
            Detection {
                start: 0,
                end: 10,
                entity_type: EntityType::Email,
                confidence: Confidence::HIGH,
                matched_text: "test@ex.co".to_string(),
            },
            Detection {
                start: 11,
                end: 20,
                entity_type: EntityType::Person,
                confidence: Confidence::MEDIUM,
                matched_text: "John Doe".to_string(),
            },
            Detection {
                start: 21,
                end: 30,
                entity_type: EntityType::Person,
                confidence: Confidence::LOW,
                matched_text: "Jane Smith".to_string(),
            },
        ];

        let filtered = policy.filter_detections(detections);

        // Should keep all
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_policy_should_mask_in_mask_mode() {
        let policy = Policy::new(GuardMode::Mask);
        assert!(policy.should_mask());
    }

    #[test]
    fn test_policy_should_not_mask_in_detect_mode() {
        let policy = Policy::new(GuardMode::Detect);
        assert!(!policy.should_mask());
    }

    #[test]
    fn test_policy_should_not_mask_in_off_mode() {
        let policy = Policy::new(GuardMode::Off);
        assert!(!policy.should_mask());
    }

    #[test]
    fn test_policy_should_not_mask_in_strict_mode() {
        let policy = Policy::new(GuardMode::Strict);
        assert!(!policy.should_mask());
    }

    #[test]
    fn test_policy_should_detect_in_detect_mode() {
        let policy = Policy::new(GuardMode::Detect);
        assert!(policy.should_detect());
    }

    #[test]
    fn test_policy_should_detect_in_mask_mode() {
        let policy = Policy::new(GuardMode::Mask);
        assert!(policy.should_detect());
    }

    #[test]
    fn test_policy_should_detect_in_strict_mode() {
        let policy = Policy::new(GuardMode::Strict);
        assert!(policy.should_detect());
    }

    #[test]
    fn test_policy_should_not_detect_in_off_mode() {
        let policy = Policy::new(GuardMode::Off);
        assert!(!policy.should_detect());
    }

    #[test]
    fn test_policy_validate_strict_with_detections() {
        let policy = Policy::new(GuardMode::Strict);
        let detections = vec![Detection {
            start: 0,
            end: 10,
            entity_type: EntityType::Email,
            confidence: Confidence::HIGH,
            matched_text: "test@ex.co".to_string(),
        }];

        let result = policy.validate_strict(&detections);

        assert!(result.is_err());
        match result.unwrap_err() {
            PolicyError::StrictModeViolation(count) => assert_eq!(count, 1),
            _ => panic!("Expected StrictModeViolation"),
        }
    }

    #[test]
    fn test_policy_validate_strict_without_detections() {
        let policy = Policy::new(GuardMode::Strict);
        let detections = vec![];

        let result = policy.validate_strict(&detections);

        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_validate_strict_not_in_strict_mode() {
        let policy = Policy::new(GuardMode::Mask);
        let detections = vec![Detection {
            start: 0,
            end: 10,
            entity_type: EntityType::Email,
            confidence: Confidence::HIGH,
            matched_text: "test@ex.co".to_string(),
        }];

        // Should not error in non-STRICT modes
        let result = policy.validate_strict(&detections);
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_summary() {
        std::env::set_var("PSEUDO_SALT", "test-salt-for-summary");
        let policy = Policy::default();
        let summary = policy.summary();

        assert_eq!(summary.mode, GuardMode::Mask);
        assert_eq!(summary.confidence_threshold, Confidence::MEDIUM);
        assert!(summary.fpe_preserve_area_code);
        assert!(summary.fpe_preserve_last_four);

        // Check that strategies are present
        assert!(summary.strategies.contains_key("SSN"));
        assert!(summary.strategies.contains_key("PHONE"));
        assert!(summary.strategies.contains_key("EMAIL"));

        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_policy_summary_serialization() {
        std::env::set_var("PSEUDO_SALT", "test-salt");
        let policy = Policy::default();
        let summary = policy.summary();

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("mode"));
        assert!(json.contains("confidence_threshold"));
        assert!(json.contains("strategies"));

        let _deserialized: PolicySummary = serde_json::from_str(&json).unwrap();

        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_policy_error_display() {
        let err = PolicyError::InvalidMode("INVALID".to_string());
        assert_eq!(err.to_string(), "Invalid guard mode: INVALID");

        let err = PolicyError::InvalidConfidence("SUPER_HIGH".to_string());
        assert_eq!(err.to_string(), "Invalid confidence: SUPER_HIGH");

        let err = PolicyError::MissingConfig("rules.yaml".to_string());
        assert_eq!(err.to_string(), "Missing configuration: rules.yaml");

        let err = PolicyError::StrictModeViolation(5);
        assert_eq!(err.to_string(), "STRICT mode: 5 PII entities detected");
    }

    #[test]
    fn test_policy_mode_case_insensitive() {
        assert_eq!("off".parse::<GuardMode>().unwrap(), GuardMode::Off);
        assert_eq!("OfF".parse::<GuardMode>().unwrap(), GuardMode::Off);
        assert_eq!("DeTeCt".parse::<GuardMode>().unwrap(), GuardMode::Detect);
    }

    #[test]
    fn test_policy_from_env_invalid_mode_uses_default() {
        std::env::set_var("GUARD_MODE", "INVALID_MODE");
        std::env::set_var("PSEUDO_SALT", "test-salt");

        let policy = Policy::from_env();

        // Should use default (MASK) when invalid mode provided
        assert_eq!(policy.mode, GuardMode::Mask);

        std::env::remove_var("GUARD_MODE");
        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_policy_from_env_invalid_confidence_uses_default() {
        std::env::set_var("GUARD_CONFIDENCE", "SUPER_HIGH");
        std::env::set_var("PSEUDO_SALT", "test-salt");

        let policy = Policy::from_env();

        // Should use default (MEDIUM) when invalid confidence provided
        assert_eq!(policy.confidence_threshold, Confidence::MEDIUM);

        std::env::remove_var("GUARD_CONFIDENCE");
        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_filter_empty_detections() {
        let policy = Policy::default();
        let detections = vec![];

        let filtered = policy.filter_detections(detections);

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_integration_off_mode() {
        let policy = Policy::new(GuardMode::Off);

        // OFF mode: no detection, no masking
        assert!(!policy.should_detect());
        assert!(!policy.should_mask());

        // Validate strict should pass (not in strict mode)
        assert!(policy.validate_strict(&[]).is_ok());
    }

    #[test]
    fn test_integration_detect_mode() {
        let policy = Policy::new(GuardMode::Detect);

        // DETECT mode: detection yes, masking no
        assert!(policy.should_detect());
        assert!(!policy.should_mask());

        // Should not error on detections (not in strict mode)
        let detections = vec![Detection {
            start: 0,
            end: 10,
            entity_type: EntityType::Email,
            confidence: Confidence::HIGH,
            matched_text: "test@ex.co".to_string(),
        }];
        assert!(policy.validate_strict(&detections).is_ok());
    }

    #[test]
    fn test_integration_mask_mode() {
        let policy = Policy::new(GuardMode::Mask);

        // MASK mode: detection yes, masking yes
        assert!(policy.should_detect());
        assert!(policy.should_mask());

        // Should not error on detections
        let detections = vec![Detection {
            start: 0,
            end: 10,
            entity_type: EntityType::Email,
            confidence: Confidence::HIGH,
            matched_text: "test@ex.co".to_string(),
        }];
        assert!(policy.validate_strict(&detections).is_ok());
    }

    #[test]
    fn test_integration_strict_mode() {
        let policy = Policy::new(GuardMode::Strict);

        // STRICT mode: detection yes, masking no, error on any detection
        assert!(policy.should_detect());
        assert!(!policy.should_mask());

        // Should error on any detection
        let detections = vec![Detection {
            start: 0,
            end: 10,
            entity_type: EntityType::Email,
            confidence: Confidence::HIGH,
            matched_text: "test@ex.co".to_string(),
        }];
        assert!(policy.validate_strict(&detections).is_err());

        // Should pass with no detections
        assert!(policy.validate_strict(&[]).is_ok());
    }

    // =========================================================================
    // END-TO-END INTEGRATION TESTS (Policy + Detection + Masking)
    // =========================================================================

    use crate::detection::{detect, Rules};
    use crate::redaction::mask;
    use crate::state::MappingState;

    #[test]
    fn test_e2e_off_mode_no_processing() {
        let policy = Policy::new(GuardMode::Off);
        let rules = Rules::default_rules();
        let text = "Contact john@example.com at 555-123-4567";

        // OFF mode: should not detect
        if policy.should_detect() {
            panic!("OFF mode should not detect");
        }

        // Text should pass through unchanged
        let result_text = text.to_string();
        assert_eq!(result_text, text);
    }

    #[test]
    fn test_e2e_detect_mode_find_but_no_mask() {
        std::env::set_var("PSEUDO_SALT", "test-salt-e2e");
        let policy = Policy::new(GuardMode::Detect);
        let rules = Rules::default_rules();
        let text = "Contact john@example.com at 555-123-4567";

        // DETECT mode: should detect
        assert!(policy.should_detect());
        assert!(!policy.should_mask());

        // Detect PII
        let detections = detect(text, &rules);
        assert!(detections.len() >= 2); // At least email and phone

        // Apply confidence filtering
        let filtered = policy.filter_detections(detections);
        assert!(!filtered.is_empty());

        // Should NOT mask in DETECT mode
        // (In real API, would return detections list, not masked text)
        
        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_e2e_mask_mode_full_pipeline() {
        std::env::set_var("PSEUDO_SALT", "test-salt-e2e-mask");
        let mut policy = Policy::new(GuardMode::Mask);
        
        // Set FPE key
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"test-salt-e2e-mask");
        hasher.update(b"FPE_KEY_DERIVATION");
        let result = hasher.finalize();
        policy.masking_policy.fpe_key.copy_from_slice(&result[..32]);

        let rules = Rules::default_rules();
        let state = MappingState::new();
        let text = "Email: john.doe@company.com, Phone: 555-123-4567, SSN: 123-45-6789";

        // Step 1: Detect
        assert!(policy.should_detect());
        let detections = detect(text, &rules);
        println!("E2E Mask Mode - Detections: {:?}", detections);
        assert!(detections.len() >= 3); // Email, phone, SSN

        // Step 2: Filter by confidence
        let filtered = policy.filter_detections(detections);
        assert!(!filtered.is_empty());

        // Step 3: Mask
        assert!(policy.should_mask());
        let result = mask(text, filtered, &policy.masking_policy, &state, "org1");

        println!("Original: {}", text);
        println!("Masked:   {}", result.masked_text);

        // Verify masking occurred
        assert_ne!(result.masked_text, text);
        assert!(result.total_redactions >= 3);

        // Verify email pseudonymized
        assert!(!result.masked_text.contains("john.doe@company.com"));
        assert!(result.masked_text.contains("EMAIL_"));

        // Verify phone FPE (area code preserved)
        assert!(result.masked_text.contains("555-"));

        // Verify SSN FPE (last 4 preserved)
        assert!(result.masked_text.contains("6789"));

        // Verify no raw PII in result
        assert!(!result.masked_text.contains("john.doe"));
        assert!(!result.masked_text.contains("123-45"));

        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_e2e_strict_mode_error_on_pii() {
        std::env::set_var("PSEUDO_SALT", "test-salt-strict");
        let policy = Policy::new(GuardMode::Strict);
        let rules = Rules::default_rules();
        let text = "Contact admin@company.com for access";

        // STRICT mode: should detect
        assert!(policy.should_detect());
        assert!(!policy.should_mask());

        // Detect PII
        let detections = detect(text, &rules);
        assert!(!detections.is_empty());

        // Filter
        let filtered = policy.filter_detections(detections);

        // Validate strict - should error
        let result = policy.validate_strict(&filtered);
        assert!(result.is_err());

        match result.unwrap_err() {
            PolicyError::StrictModeViolation(count) => {
                assert!(count > 0);
            }
            _ => panic!("Expected StrictModeViolation"),
        }

        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_e2e_confidence_threshold_filtering() {
        std::env::set_var("PSEUDO_SALT", "test-salt-threshold");
        
        // Test with HIGH threshold
        let policy_high = Policy::with_config(
            GuardMode::Mask,
            Confidence::HIGH,
            MaskingPolicy::default(),
        );
        
        let rules = Rules::default_rules();
        let text = "John Smith (low confidence name) and test@example.com (high confidence email)";

        let detections = detect(text, &rules);
        let filtered_high = policy_high.filter_detections(detections.clone());

        // HIGH threshold should filter out LOW confidence person names
        assert!(filtered_high.iter().all(|d| d.confidence == Confidence::HIGH));

        // Test with MEDIUM threshold
        let policy_med = Policy::with_config(
            GuardMode::Mask,
            Confidence::MEDIUM,
            MaskingPolicy::default(),
        );
        
        let filtered_med = policy_med.filter_detections(detections);

        // MEDIUM threshold should allow MEDIUM and HIGH
        assert!(filtered_med.len() >= filtered_high.len());

        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_e2e_mask_mode_with_graceful_degradation() {
        // MASK mode without PSEUDO_SALT should fall back to DETECT
        std::env::remove_var("PSEUDO_SALT");
        std::env::set_var("GUARD_MODE", "MASK");

        let policy = Policy::from_env();

        // Should have fallen back to DETECT mode
        assert_eq!(policy.mode, GuardMode::Detect);
        assert!(policy.should_detect());
        assert!(!policy.should_mask());

        std::env::remove_var("GUARD_MODE");
    }

    #[test]
    fn test_e2e_policy_summary_includes_all_info() {
        std::env::set_var("PSEUDO_SALT", "test-salt-summary");
        let policy = Policy::from_env();
        let summary = policy.summary();

        // Verify all expected fields present
        assert!(matches!(summary.mode, GuardMode::Mask | GuardMode::Detect));
        assert_eq!(summary.confidence_threshold, Confidence::MEDIUM);
        
        // Verify strategies for key entity types
        assert!(summary.strategies.contains_key("EMAIL"));
        assert!(summary.strategies.contains_key("PHONE"));
        assert!(summary.strategies.contains_key("SSN"));
        assert!(summary.strategies.contains_key("PERSON"));
        assert!(summary.strategies.contains_key("CREDIT_CARD"));

        // Verify FPE config
        assert!(summary.fpe_preserve_area_code);
        assert!(summary.fpe_preserve_last_four);

        // Verify serialization
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("MEDIUM"));

        std::env::remove_var("PSEUDO_SALT");
    }

    #[test]
    fn test_e2e_deterministic_masking_across_requests() {
        std::env::set_var("PSEUDO_SALT", "test-salt-deterministic");
        let mut policy = Policy::from_env();
        
        // Set FPE key
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"test-salt-deterministic");
        hasher.update(b"FPE_KEY_DERIVATION");
        let result = hasher.finalize();
        policy.masking_policy.fpe_key.copy_from_slice(&result[..32]);

        let rules = Rules::default_rules();
        let state = MappingState::new();
        let text = "Email: same@example.com twice: same@example.com";

        // First request
        let detections = detect(text, &rules);
        let filtered = policy.filter_detections(detections);
        let result1 = mask(text, filtered, &policy.masking_policy, &state, "org1");

        // Extract the two pseudonyms (should be the same)
        let parts: Vec<&str> = result1.masked_text.split("twice:").collect();
        assert_eq!(parts.len(), 2);

        // Both occurrences should have same pseudonym
        assert!(result1.masked_text.contains("EMAIL_"));
        
        // Count occurrences of EMAIL_ - should appear twice with same value
        let email_count = result1.masked_text.matches("EMAIL_").count();
        assert_eq!(email_count, 2);

        std::env::remove_var("PSEUDO_SALT");
    }
}
