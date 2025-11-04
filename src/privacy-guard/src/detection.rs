// PII detection engine
// This module implements regex-based pattern matching for 8 entity types

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    SSN,
    EMAIL,
    PHONE,
    #[serde(rename = "CREDIT_CARD")]
    CreditCard,
    PERSON,
    #[serde(rename = "IP_ADDRESS")]
    IpAddress,
    #[serde(rename = "DATE_OF_BIRTH")]
    DateOfBirth,
    #[serde(rename = "ACCOUNT_NUMBER")]
    AccountNumber,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::SSN => write!(f, "SSN"),
            EntityType::EMAIL => write!(f, "EMAIL"),
            EntityType::PHONE => write!(f, "PHONE"),
            EntityType::CreditCard => write!(f, "CREDIT_CARD"),
            EntityType::PERSON => write!(f, "PERSON"),
            EntityType::IpAddress => write!(f, "IP_ADDRESS"),
            EntityType::DateOfBirth => write!(f, "DATE_OF_BIRTH"),
            EntityType::AccountNumber => write!(f, "ACCOUNT_NUMBER"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    HIGH,
    MEDIUM,
    LOW,
}

impl PartialOrd for Confidence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Confidence {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            Confidence::HIGH => 3,
            Confidence::MEDIUM => 2,
            Confidence::LOW => 1,
        };
        let other_val = match other {
            Confidence::HIGH => 3,
            Confidence::MEDIUM => 2,
            Confidence::LOW => 1,
        };
        self_val.cmp(&other_val)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "type")]
    pub entity_type: EntityType,
    pub confidence: Confidence,
    pub matched_text: String,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub regex: Regex,
    pub confidence: Confidence,
    pub context_keywords: Option<Vec<String>>,
    pub description: String,
    pub luhn_check: bool,
}

pub struct Rules {
    patterns: HashMap<EntityType, Vec<Pattern>>,
}

impl Rules {
    /// Create rules from default hardcoded patterns
    pub fn default_rules() -> Self {
        let mut patterns: HashMap<EntityType, Vec<Pattern>> = HashMap::new();

        // SSN patterns
        patterns.insert(
            EntityType::SSN,
            vec![
                Pattern {
                    regex: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "US SSN with hyphens (xxx-xx-xxxx)".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\b\d{3}\s\d{2}\s\d{4}\b").unwrap(),
                    confidence: Confidence::MEDIUM,
                    context_keywords: None,
                    description: "US SSN with spaces".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\b\d{9}\b").unwrap(),
                    confidence: Confidence::LOW,
                    context_keywords: Some(vec!["SSN".to_string(), "social security".to_string(), "SS#".to_string()]),
                    description: "US SSN no separators (context-dependent)".to_string(),
                    luhn_check: false,
                },
            ],
        );

        // EMAIL pattern
        patterns.insert(
            EntityType::EMAIL,
            vec![Pattern {
                regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap(),
                confidence: Confidence::HIGH,
                context_keywords: None,
                description: "RFC-compliant email".to_string(),
                luhn_check: false,
            }],
        );

        // PHONE patterns
        patterns.insert(
            EntityType::PHONE,
            vec![
                Pattern {
                    regex: Regex::new(r"\b\d{3}-\d{3}-\d{4}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "US phone (xxx-xxx-xxxx)".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\(\d{3}\)\s*\d{3}-\d{4}").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "US phone with parens ((xxx) xxx-xxxx)".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\b\d{3}\.\d{3}\.\d{4}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "US phone with dots (xxx.xxx.xxxx)".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\+1\s?\d{3}\s?\d{3}\s?\d{4}").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "US phone with country code (+1 xxx xxx xxxx)".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\+\d{1,3}\s?\d{4,14}").unwrap(),
                    confidence: Confidence::MEDIUM,
                    context_keywords: None,
                    description: "International E.164 format".to_string(),
                    luhn_check: false,
                },
            ],
        );

        // CREDIT_CARD patterns
        patterns.insert(
            EntityType::CreditCard,
            vec![
                Pattern {
                    regex: Regex::new(r"\b4\d{15}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "Visa (16 digits starting with 4)".to_string(),
                    luhn_check: true,
                },
                Pattern {
                    regex: Regex::new(r"\b5[1-5]\d{14}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "Mastercard (16 digits starting with 51-55)".to_string(),
                    luhn_check: true,
                },
                Pattern {
                    regex: Regex::new(r"\b3[47]\d{13}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "Amex (15 digits starting with 34 or 37)".to_string(),
                    luhn_check: true,
                },
                Pattern {
                    regex: Regex::new(r"\b6(?:011|5\d{2})\d{12}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "Discover (16 digits starting with 6011 or 65)".to_string(),
                    luhn_check: true,
                },
                Pattern {
                    regex: Regex::new(r"\b\d{13,19}\b").unwrap(),
                    confidence: Confidence::MEDIUM,
                    context_keywords: Some(vec!["card".to_string(), "credit".to_string(), "payment".to_string()]),
                    description: "Generic 13-19 digit card number (context-dependent)".to_string(),
                    luhn_check: true,
                },
            ],
        );

        // PERSON patterns
        patterns.insert(
            EntityType::PERSON,
            vec![
                Pattern {
                    // Fixed regex: require at least first and last name after title
                    // Pattern now requires: Title + FirstName + LastName (minimum)
                    regex: Regex::new(r"(?:Mr\.|Mrs\.|Ms\.|Dr\.|Prof\.)\s+[A-Z][a-z]+\s+[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*").unwrap(),
                    confidence: Confidence::MEDIUM,
                    context_keywords: None,
                    description: "Name with title (Mr./Mrs./Ms./Dr./Prof.) + first + last".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b").unwrap(),
                    confidence: Confidence::LOW,
                    context_keywords: Some(vec![
                        "name".to_string(),
                        "person".to_string(),
                        "employee".to_string(),
                        "contact".to_string(),
                        "from".to_string(),
                        "to".to_string(),
                        "by".to_string(),
                    ]),
                    description: "Two capitalized words (prone to false positives)".to_string(),
                    luhn_check: false,
                },
            ],
        );

        // IP_ADDRESS patterns
        patterns.insert(
            EntityType::IpAddress,
            vec![
                Pattern {
                    regex: Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "IPv4 address".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "IPv6 address (full)".to_string(),
                    luhn_check: false,
                },
            ],
        );

        // DATE_OF_BIRTH patterns
        patterns.insert(
            EntityType::DateOfBirth,
            vec![
                Pattern {
                    regex: Regex::new(r"(?:DOB|Date of birth|Born|Birth date):\s*\d{1,2}/\d{1,2}/\d{2,4}").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "DOB with label (MM/DD/YYYY or variants)".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b").unwrap(),
                    confidence: Confidence::LOW,
                    context_keywords: Some(vec!["birth".to_string(), "DOB".to_string(), "born".to_string(), "age".to_string()]),
                    description: "Generic date (many false positives)".to_string(),
                    luhn_check: false,
                },
            ],
        );

        // ACCOUNT_NUMBER patterns
        patterns.insert(
            EntityType::AccountNumber,
            vec![
                Pattern {
                    regex: Regex::new(r"(?:Account|Acct|Account #|Acct #):\s*\d{8,16}").unwrap(),
                    confidence: Confidence::HIGH,
                    context_keywords: None,
                    description: "Account number with label".to_string(),
                    luhn_check: false,
                },
                Pattern {
                    regex: Regex::new(r"\b\d{8,16}\b").unwrap(),
                    confidence: Confidence::LOW,
                    context_keywords: Some(vec!["account".to_string(), "acct".to_string(), "number".to_string(), "ID".to_string()]),
                    description: "Generic 8-16 digit number".to_string(),
                    luhn_check: false,
                },
            ],
        );

        Rules { patterns }
    }

    pub fn count(&self) -> usize {
        self.patterns.values().map(|v| v.len()).sum()
    }
}

/// Luhn algorithm for credit card validation
fn is_luhn_valid(number: &str) -> bool {
    let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.is_empty() {
        return false;
    }

    let mut sum = 0;
    let parity = digits.len() % 2;

    for (i, &digit) in digits.iter().enumerate() {
        let mut d = digit;
        if i % 2 == parity {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }

    sum % 10 == 0
}

/// Detect PII entities in text using configured rules
pub fn detect(text: &str, rules: &Rules) -> Vec<Detection> {
    let mut detections = Vec::new();

    for (entity_type, patterns) in &rules.patterns {
        for pattern in patterns {
            for mat in pattern.regex.find_iter(text) {
                let matched_text = mat.as_str().to_string();

                // Skip if Luhn check required and fails
                if pattern.luhn_check && !is_luhn_valid(&matched_text) {
                    continue;
                }

                // Check context keywords if required
                if let Some(keywords) = &pattern.context_keywords {
                    // Look for keywords in surrounding text (±50 chars)
                    let start_ctx = mat.start().saturating_sub(50);
                    let end_ctx = (mat.end() + 50).min(text.len());
                    let context = &text[start_ctx..end_ctx];
                    let context_lower = context.to_lowercase();

                    let has_keyword = keywords
                        .iter()
                        .any(|kw| context_lower.contains(&kw.to_lowercase()));

                    // For LOW confidence, require context keyword
                    if pattern.confidence == Confidence::LOW && !has_keyword {
                        continue;
                    }
                    
                    // For LOW/MEDIUM confidence with keywords (generic patterns), 
                    // skip if already detected by higher confidence pattern (check overlap)
                    if pattern.confidence == Confidence::LOW || pattern.confidence == Confidence::MEDIUM {
                        let start = mat.start();
                        let end = mat.end();
                        let already_detected = detections.iter().any(|d: &Detection| {
                            d.entity_type == *entity_type && 
                            // Check for overlap or exact match
                            (d.start <= start && start < d.end || start <= d.start && d.start < end)
                        });
                        if already_detected {
                            continue;
                        }
                    }
                }

                detections.push(Detection {
                    start: mat.start(),
                    end: mat.end(),
                    entity_type: entity_type.clone(),
                    confidence: pattern.confidence.clone(),
                    matched_text,
                });
            }
        }
    }

    // Sort by start position
    detections.sort_by_key(|d| d.start);
    detections
}

/// Hybrid detection: combine regex-based and NER model results
/// This is the async version that integrates with OllamaClient
pub async fn detect_hybrid(
    text: &str,
    rules: &Rules,
    ollama: &crate::ollama_client::OllamaClient,
) -> Vec<Detection> {
    // Step 1: Regex-based detection (fast, high precision)
    let regex_detections = detect(text, rules);

    // Step 2: Model-based NER (if enabled)
    let model_entities = match ollama.extract_entities(text).await {
        Ok(entities) => entities,
        Err(e) => {
            tracing::warn!("Model extraction failed, using regex only: {}", e);
            return regex_detections; // Fallback to regex-only
        }
    };

    // If model is disabled or no entities found, return regex results
    if model_entities.is_empty() {
        return regex_detections;
    }

    // Step 3: Merge results (prioritize consensus, add model-only HIGH confidence)
    merge_detections(text, regex_detections, model_entities)
}

/// Merge regex and model detections
/// - Consensus (both methods detect) → upgrade to HIGH confidence
/// - Model-only detections → add as HIGH confidence
/// - Regex-only detections → keep original confidence
fn merge_detections(
    text: &str,
    regex_detections: Vec<Detection>,
    model_entities: Vec<crate::ollama_client::NerEntity>,
) -> Vec<Detection> {
    let mut merged = regex_detections.clone();

    // For each model entity
    for model_entity in model_entities {
        // Find entity text in the original text
        if let Some(start) = text.find(&model_entity.text) {
            let end = start + model_entity.text.len();

            // Check if already detected by regex
            let existing_detection = merged.iter_mut().find(|d| overlaps(d.start, d.end, start, end));

            if let Some(detection) = existing_detection {
                // Consensus: upgrade confidence to HIGH
                detection.confidence = Confidence::HIGH;
            } else {
                // Model-only detection: add as HIGH confidence if valid entity type
                if let Ok(entity_type) = map_ner_type(&model_entity.entity_type) {
                    merged.push(Detection {
                        start,
                        end,
                        entity_type,
                        confidence: Confidence::HIGH,
                        matched_text: model_entity.text.clone(),
                    });
                }
            }
        }
    }

    // Sort by start position
    merged.sort_by_key(|d| d.start);
    merged
}

/// Check if two ranges overlap
fn overlaps(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
    !(end1 <= start2 || end2 <= start1)
}

/// Map NER entity type string to EntityType enum
fn map_ner_type(ner_type: &str) -> Result<EntityType, String> {
    match ner_type.to_uppercase().as_str() {
        "PERSON" => Ok(EntityType::PERSON),
        "ORGANIZATION" => Ok(EntityType::PERSON), // Map to PERSON for now
        "LOCATION" => Err("LOCATION not supported".into()),
        "EMAIL" => Ok(EntityType::EMAIL),
        "PHONE" => Ok(EntityType::PHONE),
        "SSN" => Ok(EntityType::SSN),
        "CREDIT_CARD" => Ok(EntityType::CreditCard),
        "IP_ADDRESS" => Ok(EntityType::IpAddress),
        "DATE_OF_BIRTH" => Ok(EntityType::DateOfBirth),
        "ACCOUNT_NUMBER" => Ok(EntityType::AccountNumber),
        _ => Err(format!("Unknown NER type: {}", ner_type)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luhn_validation() {
        // Valid Visa
        assert!(is_luhn_valid("4532015112830366"));
        // Valid Mastercard
        assert!(is_luhn_valid("5425233430109903"));
        // Valid Amex
        assert!(is_luhn_valid("378282246310005"));
        // Invalid
        assert!(!is_luhn_valid("1234567890123456"));
        // Empty
        assert!(!is_luhn_valid(""));
    }

    #[test]
    fn test_ssn_detection() {
        let rules = Rules::default_rules();
        let text = "My SSN is 123-45-6789 and my friend's is 987 65 4321";
        let detections = detect(text, &rules);

        let ssn_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::SSN))
            .collect();

        assert_eq!(ssn_detections.len(), 2);
        assert_eq!(ssn_detections[0].matched_text, "123-45-6789");
        assert_eq!(ssn_detections[0].confidence, Confidence::HIGH);
        assert_eq!(ssn_detections[1].matched_text, "987 65 4321");
        assert_eq!(ssn_detections[1].confidence, Confidence::MEDIUM);
    }

    #[test]
    fn test_email_detection() {
        let rules = Rules::default_rules();
        let text = "Contact me at john.doe@example.com or alice@test.org";
        let detections = detect(text, &rules);

        let email_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::EMAIL))
            .collect();

        assert_eq!(email_detections.len(), 2);
        assert_eq!(email_detections[0].matched_text, "john.doe@example.com");
        assert_eq!(email_detections[1].matched_text, "alice@test.org");
    }

    #[test]
    fn test_phone_detection() {
        let rules = Rules::default_rules();
        let text = "Call 555-123-4567 or (555) 987-6543 or +1 555 234 5678";
        let detections = detect(text, &rules);

        let phone_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::PHONE))
            .collect();

        assert_eq!(phone_detections.len(), 3);
        assert_eq!(phone_detections[0].matched_text, "555-123-4567");
        assert_eq!(phone_detections[1].matched_text, "(555) 987-6543");
        assert_eq!(phone_detections[2].matched_text, "+1 555 234 5678");
    }

    #[test]
    fn test_credit_card_detection() {
        let rules = Rules::default_rules();
        // Valid Visa (passes Luhn)
        let text = "Card: 4532015112830366 invalid: 1234567890123456";
        let detections = detect(text, &rules);

        let cc_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::CreditCard))
            .collect();

        // Should detect only the valid Visa (Luhn check)
        assert_eq!(cc_detections.len(), 1);
        assert_eq!(cc_detections[0].matched_text, "4532015112830366");
    }

    #[test]
    fn test_person_detection() {
        let rules = Rules::default_rules();
        let text = "Contact Dr. John Smith or employee Alice Johnson";
        let detections = detect(text, &rules);

        let person_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::PERSON))
            .collect();

        // Dr. John Smith (MEDIUM - has title)
        // Alice Johnson (LOW - requires context keyword "employee" nearby)
        assert!(person_detections.len() >= 1);
        assert_eq!(person_detections[0].matched_text, "Dr. John Smith");
    }

    #[test]
    fn test_ip_address_detection() {
        let rules = Rules::default_rules();
        let text = "Server IP: 192.168.1.100 and external: 8.8.8.8";
        let detections = detect(text, &rules);

        let ip_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::IpAddress))
            .collect();

        assert_eq!(ip_detections.len(), 2);
        assert_eq!(ip_detections[0].matched_text, "192.168.1.100");
        assert_eq!(ip_detections[1].matched_text, "8.8.8.8");
    }

    #[test]
    fn test_date_of_birth_detection() {
        let rules = Rules::default_rules();
        let text = "DOB: 01/15/1985 and born on 12/25/2000";
        let detections = detect(text, &rules);

        let dob_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::DateOfBirth))
            .collect();

        assert_eq!(dob_detections.len(), 2);
        // First detection includes label
        assert!(dob_detections[0].matched_text.contains("01/15/1985"));
        assert_eq!(dob_detections[0].confidence, Confidence::HIGH);
        // Second detection is just the date (LOW confidence with "born" keyword)
        assert!(dob_detections[1].matched_text.contains("12/25/2000"));
    }

    #[test]
    fn test_account_number_detection() {
        let rules = Rules::default_rules();
        let text = "Account #: 1234567890123456 and ID: 98765432";
        let detections = detect(text, &rules);

        let acct_detections: Vec<_> = detections
            .iter()
            .filter(|d| matches!(d.entity_type, EntityType::AccountNumber))
            .collect();

        assert_eq!(acct_detections.len(), 2);
        // First detection includes label
        assert!(acct_detections[0].matched_text.contains("1234567890123456"));
        assert_eq!(acct_detections[0].confidence, Confidence::HIGH);
        // Second detection is just the number (LOW confidence with "ID" keyword)
        assert!(acct_detections[1].matched_text.contains("98765432"));
    }

    #[test]
    fn test_clean_text() {
        let rules = Rules::default_rules();
        let text = "This is completely clean text with no PII whatsoever. Just normal sentences.";
        let detections = detect(text, &rules);

        assert_eq!(detections.len(), 0);
    }

    #[test]
    fn test_multiple_entity_types() {
        let rules = Rules::default_rules();
        let text = "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789";
        let detections = detect(text, &rules);

        // Should detect: PERSON, PHONE, EMAIL, SSN
        assert!(detections.len() >= 4);

        let types: Vec<_> = detections.iter().map(|d| &d.entity_type).collect();
        assert!(types.contains(&&EntityType::PERSON));
        assert!(types.contains(&&EntityType::PHONE));
        assert!(types.contains(&&EntityType::EMAIL));
        assert!(types.contains(&&EntityType::SSN));
    }

    #[test]
    fn test_overlapping_patterns() {
        let rules = Rules::default_rules();
        // 555-1212 could match both phone and account number patterns
        let text = "Phone 555-123-4567 in account note";
        let detections = detect(text, &rules);

        // Detections should be sorted by position
        for i in 1..detections.len() {
            assert!(detections[i].start >= detections[i - 1].start);
        }
    }

    // Hybrid detection tests
    #[test]
    fn test_overlaps() {
        // Overlapping ranges
        assert!(overlaps(0, 10, 5, 15)); // Partial overlap
        assert!(overlaps(5, 15, 0, 10)); // Reverse overlap
        assert!(overlaps(0, 10, 0, 10)); // Exact match
        assert!(overlaps(0, 10, 2, 8));  // Contained

        // Non-overlapping ranges
        assert!(!overlaps(0, 10, 10, 20)); // Adjacent
        assert!(!overlaps(0, 10, 11, 20)); // Gap
        assert!(!overlaps(10, 20, 0, 10)); // Reverse adjacent
    }

    #[test]
    fn test_map_ner_type() {
        assert!(matches!(map_ner_type("PERSON"), Ok(EntityType::PERSON)));
        assert!(matches!(map_ner_type("EMAIL"), Ok(EntityType::EMAIL)));
        assert!(matches!(map_ner_type("PHONE"), Ok(EntityType::PHONE)));
        assert!(matches!(map_ner_type("SSN"), Ok(EntityType::SSN)));
        assert!(matches!(map_ner_type("CREDIT_CARD"), Ok(EntityType::CreditCard)));
        assert!(matches!(map_ner_type("IP_ADDRESS"), Ok(EntityType::IpAddress)));
        assert!(matches!(map_ner_type("DATE_OF_BIRTH"), Ok(EntityType::DateOfBirth)));
        assert!(matches!(map_ner_type("ACCOUNT_NUMBER"), Ok(EntityType::AccountNumber)));

        // Organization maps to PERSON
        assert!(matches!(map_ner_type("ORGANIZATION"), Ok(EntityType::PERSON)));

        // Unsupported types
        assert!(map_ner_type("LOCATION").is_err());
        assert!(map_ner_type("UNKNOWN").is_err());

        // Case insensitive
        assert!(matches!(map_ner_type("person"), Ok(EntityType::PERSON)));
        assert!(matches!(map_ner_type("Email"), Ok(EntityType::EMAIL)));
    }

    #[test]
    fn test_merge_detections_consensus() {
        use crate::ollama_client::NerEntity;

        let text = "Contact john@example.com for details";
        let regex_detections = vec![Detection {
            start: 8,
            end: 24,
            entity_type: EntityType::EMAIL,
            confidence: Confidence::HIGH,
            matched_text: "john@example.com".to_string(),
        }];

        let model_entities = vec![NerEntity {
            entity_type: "EMAIL".to_string(),
            text: "john@example.com".to_string(),
        }];

        let merged = merge_detections(text, regex_detections, model_entities);

        // Should have 1 detection with HIGH confidence (consensus)
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].confidence, Confidence::HIGH);
        assert_eq!(merged[0].matched_text, "john@example.com");
    }

    #[test]
    fn test_merge_detections_model_only() {
        use crate::ollama_client::NerEntity;

        let text = "Alice Cooper discussed the project";
        let regex_detections = vec![]; // Regex might not catch this

        let model_entities = vec![NerEntity {
            entity_type: "PERSON".to_string(),
            text: "Alice Cooper".to_string(),
        }];

        let merged = merge_detections(text, regex_detections, model_entities);

        // Should have 1 detection from model only (HIGH confidence)
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].entity_type, EntityType::PERSON);
        assert_eq!(merged[0].confidence, Confidence::HIGH);
        assert_eq!(merged[0].matched_text, "Alice Cooper");
    }

    #[test]
    fn test_merge_detections_regex_only() {
        use crate::ollama_client::NerEntity;

        let text = "SSN: 123-45-6789";
        let regex_detections = vec![Detection {
            start: 5,
            end: 16,
            entity_type: EntityType::SSN,
            confidence: Confidence::HIGH,
            matched_text: "123-45-6789".to_string(),
        }];

        let model_entities = vec![]; // Model missed this

        let merged = merge_detections(text, regex_detections.clone(), model_entities);

        // Should have 1 detection from regex only (original confidence)
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].confidence, Confidence::HIGH);
        assert_eq!(merged[0].matched_text, "123-45-6789");
    }

    #[test]
    fn test_merge_detections_mixed() {
        use crate::ollama_client::NerEntity;

        let text = "Contact Alice at alice@test.com, SSN: 123-45-6789";
        
        // Regex detects EMAIL and SSN
        let regex_detections = vec![
            Detection {
                start: 17,
                end: 31,
                entity_type: EntityType::EMAIL,
                confidence: Confidence::HIGH,
                matched_text: "alice@test.com".to_string(),
            },
            Detection {
                start: 39,
                end: 50,
                entity_type: EntityType::SSN,
                confidence: Confidence::HIGH,
                matched_text: "123-45-6789".to_string(),
            },
        ];

        // Model detects PERSON and EMAIL
        let model_entities = vec![
            NerEntity {
                entity_type: "PERSON".to_string(),
                text: "Alice".to_string(),
            },
            NerEntity {
                entity_type: "EMAIL".to_string(),
                text: "alice@test.com".to_string(),
            },
        ];

        let merged = merge_detections(text, regex_detections, model_entities);

        // Should have 3 detections:
        // 1. Alice (model-only, HIGH)
        // 2. alice@test.com (consensus, HIGH)
        // 3. 123-45-6789 (regex-only, HIGH)
        assert_eq!(merged.len(), 3);

        // Check they are sorted
        assert!(merged[0].start <= merged[1].start);
        assert!(merged[1].start <= merged[2].start);

        // All should be HIGH confidence
        assert!(merged.iter().all(|d| d.confidence == Confidence::HIGH));
    }

    #[tokio::test]
    async fn test_detect_hybrid_model_disabled() {
        let rules = Rules::default_rules();
        let text = "Contact john@example.com";

        // Model disabled
        let ollama = crate::ollama_client::OllamaClient::new(
            "http://localhost:11434".to_string(),
            "qwen3:0.6b".to_string(),
            false,
        );

        let detections = detect_hybrid(text, &rules, &ollama).await;

        // Should fall back to regex-only
        assert!(detections.len() >= 1);
        let email_det = detections
            .iter()
            .find(|d| matches!(d.entity_type, EntityType::EMAIL));
        assert!(email_det.is_some());
    }

    #[tokio::test]
    async fn test_detect_hybrid_model_unavailable() {
        let rules = Rules::default_rules();
        let text = "Contact john@example.com";

        // Model enabled but unavailable (invalid URL)
        let ollama = crate::ollama_client::OllamaClient::new(
            "http://invalid:11434".to_string(),
            "qwen3:0.6b".to_string(),
            true,
        );

        let detections = detect_hybrid(text, &rules, &ollama).await;

        // Should gracefully fall back to regex-only
        assert!(detections.len() >= 1);
        let email_det = detections
            .iter()
            .find(|d| matches!(d.entity_type, EntityType::EMAIL));
        assert!(email_det.is_some());
    }
}
