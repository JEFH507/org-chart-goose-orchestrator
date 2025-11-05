// Profile Validation Tests
//
// Comprehensive test suite for profile schema and validation logic.
// Tests cover valid profiles, invalid configurations, cross-field validations,
// serialization/deserialization, and edge cases.

#[cfg(test)]
mod profile_tests {
    use goose_controller::profile::schema::*;
    use goose_controller::profile::ProfileValidator;
    use std::collections::HashMap;

    // Helper: Create a valid finance profile for testing
    fn create_valid_finance_profile() -> Profile {
        Profile {
            role: "finance".to_string(),
            display_name: "Finance Team Agent".to_string(),
            description: "Budget approvals, compliance, and financial reporting".to_string(),
            providers: Providers {
                primary: ProviderConfig {
                    provider: "openrouter".to_string(),
                    model: "anthropic/claude-3.5-sonnet".to_string(),
                    temperature: Some(0.3),
                },
                planner: None,
                worker: Some(ProviderConfig {
                    provider: "openrouter".to_string(),
                    model: "openai/gpt-4o-mini".to_string(),
                    temperature: Some(0.5),
                }),
                allowed_providers: vec!["openrouter".to_string(), "ollama".to_string()],
                forbidden_providers: vec!["openai".to_string()], // Direct OpenAI forbidden (use OpenRouter)
            },
            extensions: vec![
                Extension {
                    name: "github".to_string(),
                    enabled: true,
                    tools: Some(vec!["list_issues".to_string(), "create_issue".to_string()]),
                    preferences: None,
                },
                Extension {
                    name: "agent_mesh".to_string(),
                    enabled: true,
                    tools: Some(vec![
                        "send_task".to_string(),
                        "request_approval".to_string(),
                        "notify".to_string(),
                        "fetch_status".to_string(),
                    ]),
                    preferences: None,
                },
                Extension {
                    name: "memory".to_string(),
                    enabled: true,
                    tools: None,
                    preferences: Some({
                        let mut prefs = HashMap::new();
                        prefs.insert("retention_days".to_string(), serde_json::json!(90));
                        prefs.insert("auto_summarize".to_string(), serde_json::json!(true));
                        prefs.insert("include_pii".to_string(), serde_json::json!(false));
                        prefs
                    }),
                },
            ],
            goosehints: GooseHints {
                global: r#"You are the Finance team agent.
Focus on budget compliance, cost tracking, and regulatory reporting.
Always verify budget availability before approving spend requests.

@finance/policies/approval-matrix.md
"#.to_string(),
                local_templates: vec![LocalTemplate {
                    path: "finance/budgets".to_string(),
                    content: "# Budget-specific context\n- Current fiscal year: FY2026\n".to_string(),
                }],
            },
            gooseignore: GooseIgnore {
                global: "**/salary_data.*\n**/bonus_plans.*\n**/tax_records.*\n**/.env\n**/secrets.*".to_string(),
                local_templates: vec![LocalTemplate {
                    path: "finance/sensitive".to_string(),
                    content: "**/employee_salaries.*\n**/executive_comp.*\n".to_string(),
                }],
            },
            recipes: vec![], // Empty for MVP unit tests
            automated_tasks: vec![],
            policies: vec![
                Policy {
                    rule_type: "deny_tool".to_string(),
                    pattern: "developer__shell".to_string(),
                    conditions: None,
                    reason: Some("No arbitrary code execution for Finance role".to_string()),
                },
                Policy {
                    rule_type: "allow_tool".to_string(),
                    pattern: "github__*".to_string(),
                    conditions: None,
                    reason: Some("Finance can use all GitHub tools".to_string()),
                },
            ],
            privacy: PrivacyConfig {
                mode: "strict".to_string(),
                strictness: "strict".to_string(),
                allow_override: false,
                local_only: None,
                rules: vec![
                    RedactionRule {
                        pattern: r"\b\d{3}-\d{2}-\d{4}\b".to_string(),
                        replacement: "[SSN]".to_string(),
                    },
                    RedactionRule {
                        pattern: r"\b[A-Z]{2}\d{6,8}\b".to_string(),
                        replacement: "[EMP_ID]".to_string(),
                    },
                ],
                pii_categories: vec!["SSN".to_string(), "EMAIL".to_string(), "EMPLOYEE_ID".to_string()],
            },
            env_vars: {
                let mut vars = HashMap::new();
                vars.insert("SESSION_RETENTION_DAYS".to_string(), "90".to_string());
                vars.insert("PRIVACY_GUARD_MODE".to_string(), "strict".to_string());
                vars
            },
            signature: None,
        }
    }

    // Test 1: Valid profile passes validation
    #[test]
    fn test_valid_profile() {
        let profile = create_valid_finance_profile();
        assert!(ProfileValidator::validate(&profile).is_ok());
    }

    // Test 2: JSON serialization/deserialization
    #[test]
    fn test_profile_json_serialization() {
        let profile = create_valid_finance_profile();

        // Serialize to JSON
        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("finance"));
        assert!(json.contains("Finance Team Agent"));

        // Deserialize back
        let deserialized: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, "finance");
        assert_eq!(deserialized.display_name, "Finance Team Agent");
        assert_eq!(deserialized.providers.primary.provider, "openrouter");
    }

    // Test 3: YAML serialization/deserialization
    #[test]
    fn test_profile_yaml_serialization() {
        let profile = create_valid_finance_profile();

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&profile).unwrap();
        assert!(yaml.contains("finance"));
        assert!(yaml.contains("Finance Team Agent"));

        // Deserialize back
        let deserialized: Profile = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.role, "finance");
        assert_eq!(deserialized.providers.primary.provider, "openrouter");
    }

    // Test 4: Invalid provider not in allowed list
    #[test]
    fn test_invalid_provider_not_in_allowed_list() {
        let mut profile = create_valid_finance_profile();
        profile.providers.allowed_providers = vec!["ollama".to_string()];
        // primary.provider is "openrouter" (not in allowed list)

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not in allowed_providers"));
    }

    // Test 5: Forbidden provider
    #[test]
    fn test_forbidden_provider() {
        let mut profile = create_valid_finance_profile();
        profile.providers.primary.provider = "openai".to_string(); // Forbidden provider

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("forbidden_providers"));
    }

    // Test 6: Missing required field (role)
    #[test]
    fn test_missing_role() {
        let mut profile = create_valid_finance_profile();
        profile.role = String::new();

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("role cannot be empty"));
    }

    // Test 7: Missing required field (display_name)
    #[test]
    fn test_missing_display_name() {
        let mut profile = create_valid_finance_profile();
        profile.display_name = String::new();

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("display_name cannot be empty"));
    }

    // Test 8: Invalid privacy mode
    #[test]
    fn test_invalid_privacy_mode() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.mode = "invalid_mode".to_string();

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid privacy mode"));
    }

    // Test 9: Invalid privacy strictness
    #[test]
    fn test_invalid_privacy_strictness() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.strictness = "ultra_strict".to_string();

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid privacy strictness"));
    }

    // Test 10: Empty extension name
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

    // Test 11: Invalid policy rule type
    #[test]
    fn test_invalid_policy_rule_type() {
        let mut profile = create_valid_finance_profile();
        profile.policies.push(Policy {
            rule_type: "maybe_allow".to_string(), // Invalid rule type
            pattern: "some_pattern".to_string(),
            conditions: None,
            reason: None,
        });

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid policy rule_type"));
    }

    // Test 12: Empty policy pattern
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

    // Test 13: Invalid temperature (out of range)
    #[test]
    fn test_invalid_temperature() {
        let mut profile = create_valid_finance_profile();
        profile.providers.primary.temperature = Some(1.5); // > 1.0

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("temperature must be between"));
    }

    // Test 14: Planner provider validation
    #[test]
    fn test_planner_provider_not_in_allowed_list() {
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

    // Test 15: Worker provider validation
    #[test]
    fn test_worker_provider_forbidden() {
        let mut profile = create_valid_finance_profile();
        profile.providers.worker = Some(ProviderConfig {
            provider: "openai".to_string(), // Forbidden provider
            model: "gpt-4".to_string(),
            temperature: Some(0.3),
        });

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Worker provider"));
    }

    // Test 16: Empty redaction pattern
    #[test]
    fn test_empty_redaction_pattern() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.rules.push(RedactionRule {
            pattern: String::new(),
            replacement: "[REDACTED]".to_string(),
        });

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pattern cannot be empty"));
    }

    // Test 17: Empty redaction replacement
    #[test]
    fn test_empty_redaction_replacement() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.rules.push(RedactionRule {
            pattern: r"\b\d{16}\b".to_string(),
            replacement: String::new(),
        });

        let result = ProfileValidator::validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("replacement cannot be empty"));
    }

    // Test 18: Default profile values
    #[test]
    fn test_default_profile() {
        let profile = Profile::default();
        assert_eq!(profile.role, "");
        assert_eq!(profile.providers.primary.provider, "openrouter");
        assert!(profile.providers.allowed_providers.contains(&"openrouter".to_string()));
        assert_eq!(profile.privacy.mode, "moderate");
        assert_eq!(profile.privacy.strictness, "moderate");
        assert!(profile.privacy.allow_override);
    }

    // Test 19: Signature serialization
    #[test]
    fn test_signature_serialization() {
        let signature = Signature {
            algorithm: "HS256".to_string(),
            vault_key: "transit/hmac/profile-signing".to_string(),
            signed_at: "2025-11-05T14:00:00Z".to_string(),
            signed_by: "admin@example.com".to_string(),
            signature: "vault:v1:HMAC...".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&signature).unwrap();
        assert!(json.contains("HS256"));
        assert!(json.contains("transit/hmac/profile-signing"));

        // Test deserialization
        let deserialized: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.algorithm, "HS256");
        assert_eq!(deserialized.signed_by, "admin@example.com");
    }

    // Test 20: Profile with signature
    #[test]
    fn test_profile_with_signature() {
        let mut profile = create_valid_finance_profile();
        profile.signature = Some(Signature {
            algorithm: "HS256".to_string(),
            vault_key: "transit/hmac/profile-signing".to_string(),
            signed_at: "2025-11-05T14:00:00Z".to_string(),
            signed_by: "admin@example.com".to_string(),
            signature: "vault:v1:HMAC...".to_string(),
        });

        // Validation should still pass
        assert!(ProfileValidator::validate(&profile).is_ok());

        // Serialization should include signature
        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("HS256"));
        assert!(json.contains("admin@example.com"));
    }
}
