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
///
/// Supports two deserial formats:
/// 1. YAML/user-friendly format (rule type as key):
///    ```yaml
///    - allow_tool: "github__*"
///      reason: "Allowed for this role"
///    ```
/// 2. JSON/struct format (explicit rule_type field):
///    ```json
///    {"rule_type": "allow_tool", "pattern": "github__*", "reason": "Allowed"}
///    ```
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
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

// Custom deserializer implementation to support both YAML and JSON formats
impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct PolicyVisitor;

        impl<'de> Visitor<'de> for PolicyVisitor {
            type Value = Policy;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a policy rule in either YAML format (rule_type: pattern) or JSON format (with explicit rule_type field)")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Policy, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut rule_type: Option<String> = None;
                let mut pattern: Option<String> = None;
                let mut conditions: Option<HashMap<String, String>> = None;
                let mut reason: Option<String> = None;

                // Collect all key-value pairs
                let mut unknown_keys: Vec<(String, serde_json::Value)> = Vec::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "rule_type" => {
                            if rule_type.is_some() {
                                return Err(de::Error::duplicate_field("rule_type"));
                            }
                            rule_type = Some(map.next_value()?);
                        }
                        "pattern" => {
                            if pattern.is_some() {
                                return Err(de::Error::duplicate_field("pattern"));
                            }
                            pattern = Some(map.next_value()?);
                        }
                        "conditions" => {
                            if conditions.is_some() {
                                return Err(de::Error::duplicate_field("conditions"));
                            }
                            // Conditions can be either:
                            // 1. Object: {"repo": "finance/*", "database": "analytics_*"}
                            // 2. Array: [{"repo": "finance/*"}, {"database": "analytics_*"}]
                            let value: serde_json::Value = map.next_value()?;
                            
                            let parsed_conditions = match value {
                                serde_json::Value::Object(obj) => {
                                    // Direct object format
                                    obj.into_iter()
                                        .map(|(k, v)| {
                                            let v_str = v.as_str().ok_or_else(|| {
                                                de::Error::custom(format!("Condition value must be string: {}", v))
                                            })?;
                                            Ok((k, v_str.to_string()))
                                        })
                                        .collect::<Result<HashMap<String, String>, _>>()?
                                }
                                serde_json::Value::Array(arr) => {
                                    // Array of single-key objects format
                                    let mut map = HashMap::new();
                                    for item in arr {
                                        if let serde_json::Value::Object(obj) = item {
                                            if obj.len() != 1 {
                                                return Err(de::Error::custom(format!(
                                                    "Each condition array item must have exactly one key, got: {:?}",
                                                    obj
                                                )));
                                            }
                                            for (k, v) in obj {
                                                let v_str = v.as_str().ok_or_else(|| {
                                                    de::Error::custom(format!("Condition value must be string: {}", v))
                                                })?;
                                                map.insert(k, v_str.to_string());
                                            }
                                        } else {
                                            return Err(de::Error::custom(format!(
                                                "Condition array items must be objects, got: {:?}",
                                                item
                                            )));
                                        }
                                    }
                                    map
                                }
                                _ => {
                                    return Err(de::Error::custom(format!(
                                        "Conditions must be object or array, got: {:?}",
                                        value
                                    )));
                                }
                            };
                            
                            conditions = Some(parsed_conditions);
                        }
                        "reason" => {
                            if reason.is_some() {
                                return Err(de::Error::duplicate_field("reason"));
                            }
                            reason = Some(map.next_value()?);
                        }
                        // Unknown key might be a rule type in YAML format
                        _ => {
                            let value: serde_json::Value = map.next_value()?;
                            unknown_keys.push((key, value));
                        }
                    }
                }

                // Determine format:
                // 1. If rule_type and pattern are present → JSON/struct format
                // 2. If unknown_keys present → YAML format (rule type as key)
                if rule_type.is_some() && pattern.is_some() {
                    // JSON/struct format: {"rule_type": "allow_tool", "pattern": "..."}
                    Ok(Policy {
                        rule_type: rule_type.unwrap(),
                        pattern: pattern.unwrap(),
                        conditions,
                        reason,
                    })
                } else if !unknown_keys.is_empty() {
                    // YAML format: {allow_tool: "github__*", reason: "..."}
                    // The first unknown key is the rule type, its value is the pattern
                    let (rule_type_key, pattern_value) = unknown_keys.remove(0);

                    // Extract pattern as string
                    let pattern_str = match pattern_value {
                        serde_json::Value::String(s) => s,
                        _ => return Err(de::Error::custom(format!(
                            "Pattern value for '{}' must be a string, got: {:?}",
                            rule_type_key, pattern_value
                        ))),
                    };

                    Ok(Policy {
                        rule_type: rule_type_key,
                        pattern: pattern_str,
                        conditions,
                        reason,
                    })
                } else {
                    // Neither format detected - missing required fields
                    Err(de::Error::missing_field("rule_type or policy rule (e.g., allow_tool)"))
                }
            }
        }

        deserializer.deserialize_map(PolicyVisitor)
    }
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
    /// Signature algorithm (e.g., "HS256", "sha2-256")
    pub algorithm: String,
    
    /// Vault transit key path
    pub vault_key: String,
    
    /// Timestamp when signed (ISO 8601) - None if not yet signed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_at: Option<String>,
    
    /// Email of signer - None if not yet signed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_by: Option<String>,
    
    /// HMAC signature (base64-encoded) - None if not yet signed
    /// Note: YAML uses 'value' field, JSON uses 'signature'
    #[serde(skip_serializing_if = "Option::is_none", alias = "value")]
    pub signature: Option<String>,
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

    #[test]
    fn test_policy_yaml_format_deserialization() {
        // YAML format: rule type as key
        let yaml = r#"
- allow_tool: "github__*"
  reason: "Allowed for this role"
- deny_tool: "developer__shell"
  reason: "No arbitrary code execution"
  conditions:
    database: "analytics_*"
"#;

        let policies: Vec<Policy> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(policies.len(), 2);

        // First policy
        assert_eq!(policies[0].rule_type, "allow_tool");
        assert_eq!(policies[0].pattern, "github__*");
        assert_eq!(policies[0].reason, Some("Allowed for this role".to_string()));
        assert!(policies[0].conditions.is_none());

        // Second policy
        assert_eq!(policies[1].rule_type, "deny_tool");
        assert_eq!(policies[1].pattern, "developer__shell");
        assert_eq!(policies[1].reason, Some("No arbitrary code execution".to_string()));
        assert!(policies[1].conditions.is_some());
        let conditions = policies[1].conditions.as_ref().unwrap();
        assert_eq!(conditions.get("database"), Some(&"analytics_*".to_string()));
    }

    #[test]
    fn test_policy_yaml_array_conditions() {
        // YAML format with array conditions (YAML list of single-key maps)
        let yaml = r#"
- allow_tool: "github__list_issues"
  reason: "Read budget tracking issues"
  conditions:
    - repo: "finance/*"
- allow_tool: "github__create_issue"
  reason: "Create budget request issues"
  conditions:
    - repo: "finance/budget-requests"
    - project: "budgeting"
"#;

        let policies: Vec<Policy> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(policies.len(), 2);

        // First policy
        assert_eq!(policies[0].rule_type, "allow_tool");
        assert_eq!(policies[0].pattern, "github__list_issues");
        let cond1 = policies[0].conditions.as_ref().unwrap();
        assert_eq!(cond1.get("repo"), Some(&"finance/*".to_string()));

        // Second policy
        assert_eq!(policies[1].rule_type, "allow_tool");
        assert_eq!(policies[1].pattern, "github__create_issue");
        let cond2 = policies[1].conditions.as_ref().unwrap();
        assert_eq!(cond2.get("repo"), Some(&"finance/budget-requests".to_string()));
        assert_eq!(cond2.get("project"), Some(&"budgeting".to_string()));
    }

    #[test]
    fn test_policy_json_format_deserialization() {
        // JSON format: explicit rule_type field
        let json = r#"[
  {"rule_type": "allow_tool", "pattern": "excel-mcp__*", "reason": "Finance needs spreadsheets"},
  {"rule_type": "deny_data", "pattern": "pii_*"}
]"#;

        let policies: Vec<Policy> = serde_json::from_str(json).unwrap();
        assert_eq!(policies.len(), 2);

        // First policy
        assert_eq!(policies[0].rule_type, "allow_tool");
        assert_eq!(policies[0].pattern, "excel-mcp__*");
        assert_eq!(policies[0].reason, Some("Finance needs spreadsheets".to_string()));

        // Second policy
        assert_eq!(policies[1].rule_type, "deny_data");
        assert_eq!(policies[1].pattern, "pii_*");
        assert!(policies[1].reason.is_none());
    }

    #[test]
    fn test_policy_roundtrip() {
        let policy = Policy {
            rule_type: "allow_tool".to_string(),
            pattern: "github__*".to_string(),
            conditions: None,
            reason: Some("Allowed".to_string()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&policy).unwrap();

        // Deserialize back
        let deserialized: Policy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, policy);
    }

    #[test]
    fn test_full_profile_with_yaml_policies() {
        let yaml = r#"
role: finance
display_name: Finance Team Agent
description: Budget approvals and reporting
providers:
  primary:
    provider: openrouter
    model: anthropic/claude-3.5-sonnet
    temperature: 0.3
  allowed_providers:
    - openrouter
  forbidden_providers: []
extensions:
  - name: excel-mcp
    enabled: true
goosehints:
  global: "You are a finance agent"
  local_templates: []
gooseignore:
  global: "**/.env"
  local_templates: []
recipes: []
automated_tasks: []
policies:
  - allow_tool: "excel-mcp__*"
    reason: "Finance needs spreadsheet operations"
  - deny_tool: "developer__shell"
    reason: "No arbitrary code execution"
privacy:
  mode: moderate
  strictness: moderate
  allow_override: true
  rules: []
  pii_categories: []
env_vars: {}
"#;

        let profile: Profile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(profile.role, "finance");
        assert_eq!(profile.policies.len(), 2);
        assert_eq!(profile.policies[0].rule_type, "allow_tool");
        assert_eq!(profile.policies[0].pattern, "excel-mcp__*");
        assert_eq!(profile.policies[1].rule_type, "deny_tool");
        assert_eq!(profile.policies[1].pattern, "developer__shell");
    }
}
