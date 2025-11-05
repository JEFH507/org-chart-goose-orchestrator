// Profile Validator - Cross-field validation logic
//
// This module implements validation rules that span multiple fields in a profile,
// ensuring consistency and correctness across the profile structure.

use crate::profile::schema::Profile;
use anyhow::{Result, bail};
use std::path::Path;

/// Profile validator with cross-field validation rules
pub struct ProfileValidator;

impl ProfileValidator {
    /// Validate a complete profile
    ///
    /// Performs the following validations:
    /// 1. allowed_providers must include primary.provider
    /// 2. Recipe paths must exist in recipes/ directory
    /// 3. Extension names must be non-empty (Block registry validation deferred)
    /// 4. Privacy mode must be valid ("rules", "ner", "hybrid")
    /// 5. Strictness must be valid ("strict", "moderate", "permissive")
    /// 6. Policy rule types must be valid
    /// 7. Required fields must be non-empty
    pub fn validate(profile: &Profile) -> Result<()> {
        // 1. Validate required fields
        Self::validate_required_fields(profile)?;
        
        // 2. Validate provider configuration
        Self::validate_providers(profile)?;
        
        // 3. Validate recipe paths
        Self::validate_recipes(profile)?;
        
        // 4. Validate extensions
        Self::validate_extensions(profile)?;
        
        // 5. Validate privacy configuration
        Self::validate_privacy(profile)?;
        
        // 6. Validate policies
        Self::validate_policies(profile)?;
        
        Ok(())
    }
    
    /// Validate required fields
    fn validate_required_fields(profile: &Profile) -> Result<()> {
        if profile.role.is_empty() {
            bail!("Profile role cannot be empty");
        }
        
        if profile.display_name.is_empty() {
            bail!("Profile display_name cannot be empty");
        }
        
        Ok(())
    }
    
    /// Validate provider configuration
    fn validate_providers(profile: &Profile) -> Result<()> {
        let primary_provider = &profile.providers.primary.provider;
        
        // Check if primary provider is in allowed list (if list is not empty)
        if !profile.providers.allowed_providers.is_empty()
            && !profile.providers.allowed_providers.contains(primary_provider)
        {
            bail!(
                "Primary provider '{}' not in allowed_providers list: {:?}",
                primary_provider,
                profile.providers.allowed_providers
            );
        }
        
        // Check if primary provider is forbidden
        if profile.providers.forbidden_providers.contains(primary_provider) {
            bail!(
                "Primary provider '{}' is in forbidden_providers list",
                primary_provider
            );
        }
        
        // Validate planner provider (if specified)
        if let Some(planner) = &profile.providers.planner {
            if !profile.providers.allowed_providers.is_empty()
                && !profile.providers.allowed_providers.contains(&planner.provider)
            {
                bail!(
                    "Planner provider '{}' not in allowed_providers list",
                    planner.provider
                );
            }
            
            if profile.providers.forbidden_providers.contains(&planner.provider) {
                bail!(
                    "Planner provider '{}' is in forbidden_providers list",
                    planner.provider
                );
            }
        }
        
        // Validate worker provider (if specified)
        if let Some(worker) = &profile.providers.worker {
            if !profile.providers.allowed_providers.is_empty()
                && !profile.providers.allowed_providers.contains(&worker.provider)
            {
                bail!(
                    "Worker provider '{}' not in allowed_providers list",
                    worker.provider
                );
            }
            
            if profile.providers.forbidden_providers.contains(&worker.provider) {
                bail!(
                    "Worker provider '{}' is in forbidden_providers list",
                    worker.provider
                );
            }
        }
        
        // Validate temperature ranges
        if let Some(temp) = profile.providers.primary.temperature {
            if !(0.0..=1.0).contains(&temp) {
                bail!("Primary provider temperature must be between 0.0 and 1.0, got {}", temp);
            }
        }
        
        Ok(())
    }
    
    /// Validate recipe paths
    fn validate_recipes(profile: &Profile) -> Result<()> {
        for recipe in &profile.recipes {
            let recipe_path = format!("recipes/{}/{}", profile.role, recipe.path);
            
            // Skip validation for non-existent recipes directory (dev mode)
            // In production, this check should be enforced
            if Path::new("recipes").exists() && !Path::new(&recipe_path).exists() {
                bail!("Recipe file not found: {}", recipe_path);
            }
            
            // Validate cron schedule format (basic validation)
            if recipe.schedule.is_empty() {
                bail!("Recipe '{}' has empty schedule", recipe.name);
            }
        }
        
        Ok(())
    }
    
    /// Validate extensions
    fn validate_extensions(profile: &Profile) -> Result<()> {
        for ext in &profile.extensions {
            if ext.name.is_empty() {
                bail!("Extension name cannot be empty");
            }
            
            // Future: Validate against Block registry catalog
            // For MVP, we just validate non-empty names
        }
        
        Ok(())
    }
    
    /// Validate privacy configuration
    fn validate_privacy(profile: &Profile) -> Result<()> {
        // Validate privacy mode
        let valid_modes = vec!["rules", "ner", "hybrid"];
        if !valid_modes.contains(&profile.privacy.mode.as_str()) {
            bail!(
                "Invalid privacy mode: '{}'. Must be one of: {:?}",
                profile.privacy.mode,
                valid_modes
            );
        }
        
        // Validate strictness
        let valid_strictness = vec!["strict", "moderate", "permissive"];
        if !valid_strictness.contains(&profile.privacy.strictness.as_str()) {
            bail!(
                "Invalid privacy strictness: '{}'. Must be one of: {:?}",
                profile.privacy.strictness,
                valid_strictness
            );
        }
        
        // Validate redaction rules
        for rule in &profile.privacy.rules {
            if rule.pattern.is_empty() {
                bail!("Redaction rule pattern cannot be empty");
            }
            
            if rule.replacement.is_empty() {
                bail!("Redaction rule replacement cannot be empty");
            }
        }
        
        Ok(())
    }
    
    /// Validate policy rules
    fn validate_policies(profile: &Profile) -> Result<()> {
        let valid_rule_types = vec!["allow_tool", "deny_tool", "allow_data", "deny_data"];
        
        for policy in &profile.policies {
            if !valid_rule_types.contains(&policy.rule_type.as_str()) {
                bail!(
                    "Invalid policy rule_type: '{}'. Must be one of: {:?}",
                    policy.rule_type,
                    valid_rule_types
                );
            }
            
            if policy.pattern.is_empty() {
                bail!("Policy pattern cannot be empty");
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::schema::*;
    use std::collections::HashMap;

    fn create_valid_finance_profile() -> Profile {
        Profile {
            role: "finance".to_string(),
            display_name: "Finance Team Agent".to_string(),
            description: "Budget approvals and reporting".to_string(),
            providers: Providers {
                primary: ProviderConfig {
                    provider: "openrouter".to_string(),
                    model: "anthropic/claude-3.5-sonnet".to_string(),
                    temperature: Some(0.3),
                },
                planner: None,
                worker: None,
                allowed_providers: vec!["openrouter".to_string(), "ollama".to_string()],
                forbidden_providers: Vec::new(),
            },
            extensions: vec![Extension {
                name: "github".to_string(),
                enabled: true,
                tools: Some(vec!["list_issues".to_string()]),
                preferences: None,
            }],
            goosehints: GooseHints {
                global: "You are a finance agent.".to_string(),
                local_templates: Vec::new(),
            },
            gooseignore: GooseIgnore {
                global: "**/.env\n**/secrets.*".to_string(),
                local_templates: Vec::new(),
            },
            recipes: Vec::new(), // Empty for MVP tests
            automated_tasks: Vec::new(),
            policies: vec![Policy {
                rule_type: "deny_tool".to_string(),
                pattern: "developer__shell".to_string(),
                conditions: None,
                reason: Some("No code execution for Finance role".to_string()),
            }],
            privacy: PrivacyConfig {
                mode: "strict".to_string(),
                strictness: "strict".to_string(),
                allow_override: false,
                local_only: None,
                rules: Vec::new(),
                pii_categories: vec!["SSN".to_string(), "EMAIL".to_string()],
            },
            env_vars: HashMap::new(),
            signature: None,
        }
    }

    #[test]
    fn test_valid_profile() {
        let profile = create_valid_finance_profile();
        assert!(ProfileValidator::validate(&profile).is_ok());
    }

    #[test]
    fn test_invalid_provider_not_in_allowed_list() {
        let mut profile = create_valid_finance_profile();
        profile.providers.allowed_providers = vec!["ollama".to_string()];
        // primary.provider is "openrouter" (not in allowed list)
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not in allowed_providers"));
    }

    #[test]
    fn test_forbidden_provider() {
        let mut profile = create_valid_finance_profile();
        profile.providers.forbidden_providers = vec!["openrouter".to_string()];
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("forbidden_providers"));
    }

    #[test]
    fn test_missing_required_fields() {
        let mut profile = create_valid_finance_profile();
        profile.role = String::new(); // Empty role
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("role cannot be empty"));
    }

    #[test]
    fn test_missing_display_name() {
        let mut profile = create_valid_finance_profile();
        profile.display_name = String::new();
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("display_name cannot be empty"));
    }

    #[test]
    fn test_invalid_privacy_mode() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.mode = "invalid_mode".to_string();
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid privacy mode"));
    }

    #[test]
    fn test_invalid_privacy_strictness() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.strictness = "invalid_strictness".to_string();
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid privacy strictness"));
    }

    #[test]
    fn test_empty_extension_name() {
        let mut profile = create_valid_finance_profile();
        profile.extensions.push(Extension {
            name: String::new(),
            enabled: true,
            tools: None,
            preferences: None,
        });
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Extension name cannot be empty"));
    }

    #[test]
    fn test_invalid_policy_rule_type() {
        let mut profile = create_valid_finance_profile();
        profile.policies.push(Policy {
            rule_type: "invalid_type".to_string(),
            pattern: "some_pattern".to_string(),
            conditions: None,
            reason: None,
        });
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid policy rule_type"));
    }

    #[test]
    fn test_empty_policy_pattern() {
        let mut profile = create_valid_finance_profile();
        profile.policies.push(Policy {
            rule_type: "allow_tool".to_string(),
            pattern: String::new(),
            conditions: None,
            reason: None,
        });
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Policy pattern cannot be empty"));
    }

    #[test]
    fn test_invalid_temperature() {
        let mut profile = create_valid_finance_profile();
        profile.providers.primary.temperature = Some(1.5); // Out of range
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("temperature must be between"));
    }

    #[test]
    fn test_planner_provider_validation() {
        let mut profile = create_valid_finance_profile();
        profile.providers.planner = Some(ProviderConfig {
            provider: "anthropic".to_string(), // Not in allowed list
            model: "claude-3.5-sonnet".to_string(),
            temperature: Some(0.3),
        });
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Planner provider"));
    }

    #[test]
    fn test_worker_provider_validation() {
        let mut profile = create_valid_finance_profile();
        profile.providers.worker = Some(ProviderConfig {
            provider: "openai".to_string(), // Not in allowed list
            model: "gpt-4".to_string(),
            temperature: Some(0.3),
        });
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Worker provider"));
    }

    #[test]
    fn test_empty_redaction_pattern() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.rules.push(RedactionRule {
            pattern: String::new(),
            replacement: "[SSN]".to_string(),
        });
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pattern cannot be empty"));
    }

    #[test]
    fn test_empty_redaction_replacement() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.rules.push(RedactionRule {
            pattern: r"\b\d{3}-\d{2}-\d{4}\b".to_string(),
            replacement: String::new(),
        });
        
        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("replacement cannot be empty"));
    }
}
