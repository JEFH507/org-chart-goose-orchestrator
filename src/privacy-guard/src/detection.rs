// PII detection engine
// This module implements regex-based pattern matching for 8 entity types

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    HIGH,
    MEDIUM,
    LOW,
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

// Module implementation will be added in Task A2
