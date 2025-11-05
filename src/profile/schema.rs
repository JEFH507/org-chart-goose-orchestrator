// Profile Schema - Rust types for role-based configuration
//
// This schema defines the complete structure of a role profile, including:
// - LLM provider configuration (primary, planner, worker models)
// - MCP extension allowlists and tool permissions
// - Global goosehints and gooseignore patterns
// - Recipe automation schedules
// - Privacy Guard configuration
// - RBAC/ABAC policy rules
// - Vault-backed cryptographic signatures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete role profile definition
///
/// A profile encapsulates all configuration needed to run a role-based agent,
/// including provider settings, extensions, hints, recipes, policies, and privacy controls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Role identifier (e.g., "finance", "manager", "analyst")
    pub role: String,
    
    /// Human-readable role name (e.g., "Finance Team Agent")
    pub display_name: String,
    
    /// Role description
    pub description: String,
    
    /// LLM provider configuration (primary, planner, worker models)
    pub providers: Providers,
    
    /// MCP extensions configuration
    pub extensions: Vec<Extension>,
    
    /// Global goosehints (org-wide context)
    pub goosehints: GooseHints,
    
    /// Global gooseignore (privacy protection)
    pub gooseignore: GooseIgnore,
    
    /// Recipe automation (scheduled workflows)
    pub recipes: Vec<Recipe>,
    
    /// Automated tasks (cron-based execution)
    #[serde(default)]
    pub automated_tasks: Vec<AutomatedTask>,
    
    /// RBAC/ABAC policy rules
    pub policies: Vec<Policy>,
    
    /// Privacy Guard configuration
    pub privacy: PrivacyConfig,
    
    /// Environment variables (role-specific defaults)
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    
    /// Cryptographic signature (Vault-backed HMAC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
}

/// LLM provider configuration
///
/// Defines which LLM providers and models to use for different purposes:
/// - Primary: Main model for interactive tasks
/// - Planner: Model for creating execution plans
/// - Worker: Model for executing tasks (often cheaper/faster)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Providers {
    /// Primary model configuration (required)
    pub primary: ProviderConfig,
    
    /// Planner model configuration (optional, defaults to primary)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planner: Option<ProviderConfig>,
    
    /// Worker model configuration (optional, defaults to primary)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<ProviderConfig>,
    
    /// Allowed providers (governance constraint)
    /// Empty list means all providers allowed
    #[serde(default)]
    pub allowed_providers: Vec<String>,
    
    /// Forbidden providers (governance constraint)
    /// Takes precedence over allowed_providers
    #[serde(default)]
    pub forbidden_providers: Vec<String>,
}

/// Individual LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    /// Provider name (e.g., "openrouter", "anthropic", "openai", "ollama")
    pub provider: String,
    
    /// Model name (e.g., "anthropic/claude-3.5-sonnet", "gpt-4")
    pub model: String,
    
    /// Temperature (0.0-1.0, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// MCP extension configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Extension {
    /// Extension name (must match Block registry catalog)
    pub name: String,
    
    /// Whether extension is enabled
    pub enabled: bool,
    
    /// Allowed tools (optional, all tools allowed if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    
    /// Extension-specific preferences (arbitrary JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<HashMap<String, serde_json::Value>>,
}

/// Global goosehints configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GooseHints {
    /// Global hints (applied to all sessions)
    pub global: String,
    
    /// Local hint templates (path-specific hints)
    #[serde(default)]
    pub local_templates: Vec<LocalTemplate>,
}

/// Global gooseignore configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GooseIgnore {
    /// Global ignore patterns (applied to all sessions)
    pub global: String,
    
    /// Local ignore templates (path-specific ignore patterns)
    #[serde(default)]
    pub local_templates: Vec<LocalTemplate>,
}

/// Local template (path-specific configuration)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalTemplate {
    /// Path where template applies (e.g., "finance/budgets")
    pub path: String,
    
    /// Template content
    pub content: String,
}

/// Recipe definition (automated workflow)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recipe {
    /// Recipe name
    pub name: String,
    
    /// Recipe description
    pub description: String,
    
    /// Path to recipe YAML file (relative to recipes/)
    pub path: String,
    
    /// Cron schedule (e.g., "0 9 * * 1-5" for Mon-Fri 9am)
    pub schedule: String,
    
    /// Whether recipe is enabled
    pub enabled: bool,
}

/// Automated task (scheduled execution)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomatedTask {
    /// Task name
    pub name: String,
    
    /// Recipe to execute
    pub recipe: String,
    
    /// Cron schedule
    pub schedule: String,
    
    /// Whether task is enabled
    pub enabled: bool,
    
    /// Whether to notify on failure
    #[serde(default)]
    pub notify_on_failure: bool,
}

/// RBAC/ABAC policy rule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    /// Rule type ("allow_tool", "deny_tool", "allow_data", "deny_data")
    pub rule_type: String,
    
    /// Pattern to match (e.g., "github__*", "sql-mcp__query")
    pub pattern: String,
    
    /// Conditions for rule evaluation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<HashMap<String, String>>,
    
    /// Human-readable reason (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Privacy Guard configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivacyConfig {
    /// Privacy mode ("rules", "ner", "hybrid")
    pub mode: String,
    
    /// Strictness level ("strict", "moderate", "permissive")
    pub strictness: String,
    
    /// Whether user can override settings
    #[serde(default)]
    pub allow_override: bool,
    
    /// Whether to force local-only processing (no cloud providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_only: Option<bool>,
    
    /// Redaction rules (regex patterns)
    #[serde(default)]
    pub rules: Vec<RedactionRule>,
    
    /// PII categories to detect (e.g., "SSN", "EMAIL", "PHONE")
    #[serde(default)]
    pub pii_categories: Vec<String>,
}

/// Redaction rule (regex-based PII masking)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedactionRule {
    /// Regex pattern to match
    pub pattern: String,
    
    /// Replacement string (e.g., "[SSN]", "[EMAIL]")
    pub replacement: String,
}

/// Cryptographic signature (Vault-backed HMAC)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signature {
    /// Signature algorithm (e.g., "HS256")
    pub algorithm: String,
    
    /// Vault transit key path
    pub vault_key: String,
    
    /// Timestamp when signed (ISO 8601)
    pub signed_at: String,
    
    /// Email of signer
    pub signed_by: String,
    
    /// HMAC signature (base64-encoded)
    pub signature: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            role: String::new(),
            display_name: String::new(),
            description: String::new(),
            providers: Providers::default(),
            extensions: Vec::new(),
            goosehints: GooseHints::default(),
            gooseignore: GooseIgnore::default(),
            recipes: Vec::new(),
            automated_tasks: Vec::new(),
            policies: Vec::new(),
            privacy: PrivacyConfig::default(),
            env_vars: HashMap::new(),
            signature: None,
        }
    }
}

impl Default for Providers {
    fn default() -> Self {
        Self {
            primary: ProviderConfig {
                provider: "openrouter".to_string(),
                model: "anthropic/claude-3.5-sonnet".to_string(),
                temperature: Some(0.3),
            },
            planner: None,
            worker: None,
            allowed_providers: vec!["openrouter".to_string()],
            forbidden_providers: Vec::new(),
        }
    }
}

impl Default for GooseHints {
    fn default() -> Self {
        Self {
            global: String::new(),
            local_templates: Vec::new(),
        }
    }
}

impl Default for GooseIgnore {
    fn default() -> Self {
        Self {
            global: "**/.env\n**/.env.*\n**/secrets.*".to_string(),
            local_templates: Vec::new(),
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            mode: "moderate".to_string(),
            strictness: "moderate".to_string(),
            allow_override: true,
            local_only: None,
            rules: Vec::new(),
            pii_categories: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_serialization() {
        let profile = Profile {
            role: "finance".to_string(),
            display_name: "Finance Team Agent".to_string(),
            description: "Budget approvals and reporting".to_string(),
            providers: Providers::default(),
            extensions: vec![Extension {
                name: "github".to_string(),
                enabled: true,
                tools: Some(vec!["list_issues".to_string()]),
                preferences: None,
            }],
            goosehints: GooseHints::default(),
            gooseignore: GooseIgnore::default(),
            recipes: vec![],
            automated_tasks: vec![],
            policies: vec![],
            privacy: PrivacyConfig::default(),
            env_vars: HashMap::new(),
            signature: None,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("finance"));

        // Test deserialization
        let deserialized: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, "finance");
    }

    #[test]
    fn test_profile_yaml_serialization() {
        let profile = Profile {
            role: "analyst".to_string(),
            display_name: "Business Analyst".to_string(),
            description: "Data analysis and insights".to_string(),
            ..Default::default()
        };

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&profile).unwrap();
        assert!(yaml.contains("analyst"));

        // Test deserialization
        let deserialized: Profile = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.role, "analyst");
    }

    #[test]
    fn test_default_profile() {
        let profile = Profile::default();
        assert_eq!(profile.role, "");
        assert_eq!(profile.providers.primary.provider, "openrouter");
        assert!(profile.providers.allowed_providers.contains(&"openrouter".to_string()));
    }
}
