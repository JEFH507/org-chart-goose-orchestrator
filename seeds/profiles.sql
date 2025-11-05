-- Seed Data for Phase 5 Profiles Table
-- Inserts 6 role profiles: Finance, Manager, Analyst, Marketing, Support, Legal
-- Note: Signature values will be populated by Controller on publish via POST /admin/profiles/{role}/publish

-- Finance Profile
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'finance',
  'Finance Team Agent',
  '{
    "role": "finance",
    "display_name": "Finance Team Agent",
    "description": "Budget approvals, compliance reporting, financial analysis, and regulatory oversight",
    "providers": {
      "primary": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.3},
      "planner": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.2},
      "worker": {"provider": "openrouter", "model": "openai/gpt-4o-mini", "temperature": 0.4},
      "allowed_providers": ["openrouter"],
      "forbidden_providers": []
    },
    "extensions": [
      {"name": "github", "enabled": true, "tools": ["list_issues", "create_issue", "add_comment"]},
      {"name": "agent_mesh", "enabled": true, "tools": ["send_task", "request_approval", "notify", "fetch_status"]},
      {"name": "memory", "enabled": true, "preferences": {"retention_days": 90, "auto_summarize": true, "include_pii": false}},
      {"name": "excel-mcp", "enabled": true}
    ],
    "recipes": [
      {"name": "monthly-budget-close", "path": "recipes/finance/monthly-budget-close.yaml", "schedule": "0 9 5 * *", "enabled": true},
      {"name": "weekly-spend-report", "path": "recipes/finance/weekly-spend-report.yaml", "schedule": "0 10 * * 1", "enabled": true},
      {"name": "quarterly-forecast", "path": "recipes/finance/quarterly-forecast.yaml", "schedule": "0 9 1 1,4,7,10 *", "enabled": true}
    ],
    "privacy": {
      "mode": "hybrid",
      "strictness": "strict",
      "allow_override": false,
      "pii_categories": ["SSN", "EMAIL", "PHONE", "EMPLOYEE_ID", "CREDIT_CARD", "ROUTING_NUMBER", "COMPENSATION"]
    },
    "env_vars": {
      "SESSION_RETENTION_DAYS": "90",
      "PRIVACY_GUARD_MODE": "hybrid",
      "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet"
    }
  }'::jsonb,
  NULL  -- Signature populated on publish
);

-- Manager Profile
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'manager',
  'Manager Team Agent',
  '{
    "role": "manager",
    "display_name": "Manager Team Agent",
    "description": "Team oversight, approval workflows, delegation, and cross-functional coordination",
    "providers": {
      "primary": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.4},
      "planner": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.3},
      "worker": {"provider": "openrouter", "model": "openai/gpt-4o", "temperature": 0.5},
      "allowed_providers": ["openrouter"],
      "forbidden_providers": []
    },
    "extensions": [
      {"name": "github", "enabled": true, "tools": ["list_issues", "create_issue", "add_comment", "update_issue", "assign_issue"]},
      {"name": "agent_mesh", "enabled": true, "tools": ["send_task", "request_approval", "notify", "fetch_status"]},
      {"name": "memory", "enabled": true, "preferences": {"retention_days": 90, "auto_summarize": true, "include_pii": false}}
    ],
    "recipes": [
      {"name": "daily-standup-summary", "path": "recipes/manager/daily-standup-summary.yaml", "schedule": "0 9 * * 1-5", "enabled": true},
      {"name": "weekly-team-metrics", "path": "recipes/manager/weekly-team-metrics.yaml", "schedule": "0 10 * * 1", "enabled": true},
      {"name": "monthly-1on1-prep", "path": "recipes/manager/monthly-1on1-prep.yaml", "schedule": "0 9 1 * *", "enabled": true}
    ],
    "privacy": {
      "mode": "hybrid",
      "strictness": "moderate",
      "allow_override": true,
      "pii_categories": ["SSN", "EMAIL", "PHONE", "EMPLOYEE_ID"]
    },
    "env_vars": {
      "SESSION_RETENTION_DAYS": "90",
      "PRIVACY_GUARD_MODE": "hybrid",
      "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet",
      "BUDGET_APPROVAL_LIMIT": "50000"
    }
  }'::jsonb,
  NULL
);

-- Analyst Profile
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'analyst',
  'Business Analyst',
  '{
    "role": "analyst",
    "display_name": "Business Analyst",
    "description": "Data analysis, process optimization, time studies, and business intelligence",
    "providers": {
      "primary": {"provider": "openrouter", "model": "openai/gpt-4o", "temperature": 0.3},
      "planner": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.2},
      "worker": {"provider": "openrouter", "model": "openai/gpt-4o-mini", "temperature": 0.4},
      "allowed_providers": ["openrouter"],
      "forbidden_providers": []
    },
    "extensions": [
      {"name": "developer", "enabled": true},
      {"name": "excel-mcp", "enabled": true},
      {"name": "sql-mcp", "enabled": true},
      {"name": "agent_mesh", "enabled": true, "tools": ["send_task", "request_approval", "notify", "fetch_status"]},
      {"name": "memory", "enabled": true, "preferences": {"retention_days": 60, "auto_summarize": true, "include_pii": false}}
    ],
    "recipes": [
      {"name": "daily-kpi-report", "path": "recipes/analyst/daily-kpi-report.yaml", "schedule": "0 9 * * 1-5", "enabled": true},
      {"name": "process-bottleneck-analysis", "path": "recipes/analyst/process-bottleneck-analysis.yaml", "schedule": "0 10 * * 1", "enabled": true},
      {"name": "time-study-analysis", "path": "recipes/analyst/time-study-analysis.yaml", "schedule": "0 9 1 * *", "enabled": true}
    ],
    "privacy": {
      "mode": "hybrid",
      "strictness": "moderate",
      "allow_override": true,
      "pii_categories": ["SSN", "EMAIL", "PHONE", "EMPLOYEE_ID"]
    },
    "env_vars": {
      "SESSION_RETENTION_DAYS": "60",
      "PRIVACY_GUARD_MODE": "hybrid",
      "DEFAULT_MODEL": "openrouter/openai/gpt-4o"
    }
  }'::jsonb,
  NULL
);

-- Marketing Profile
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'marketing',
  'Marketing Team Agent',
  '{
    "role": "marketing",
    "display_name": "Marketing Team Agent",
    "description": "Campaign management, content creation, competitor analysis, and marketing analytics",
    "providers": {
      "primary": {"provider": "openrouter", "model": "openai/gpt-4o", "temperature": 0.7},
      "planner": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.5},
      "worker": {"provider": "openrouter", "model": "openai/gpt-4o-mini", "temperature": 0.8},
      "allowed_providers": ["openrouter"],
      "forbidden_providers": []
    },
    "extensions": [
      {"name": "github", "enabled": true, "tools": ["list_issues", "create_issue", "add_comment"]},
      {"name": "agent_mesh", "enabled": true, "tools": ["send_task", "request_approval", "notify", "fetch_status"]},
      {"name": "memory", "enabled": true, "preferences": {"retention_days": 60, "auto_summarize": true, "include_pii": false}},
      {"name": "web-scraper", "enabled": true}
    ],
    "recipes": [
      {"name": "weekly-campaign-report", "path": "recipes/marketing/weekly-campaign-report.yaml", "schedule": "0 10 * * 1", "enabled": true},
      {"name": "monthly-content-calendar", "path": "recipes/marketing/monthly-content-calendar.yaml", "schedule": "0 9 1 * *", "enabled": true},
      {"name": "competitor-analysis", "path": "recipes/marketing/competitor-analysis.yaml", "schedule": "0 9 1 * *", "enabled": true}
    ],
    "privacy": {
      "mode": "rules",
      "strictness": "permissive",
      "allow_override": true,
      "pii_categories": ["EMAIL", "PHONE"]
    },
    "env_vars": {
      "SESSION_RETENTION_DAYS": "60",
      "PRIVACY_GUARD_MODE": "rules",
      "DEFAULT_MODEL": "openrouter/openai/gpt-4o"
    }
  }'::jsonb,
  NULL
);

-- Support Profile
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'support',
  'Support Team Agent',
  '{
    "role": "support",
    "display_name": "Support Team Agent",
    "description": "Customer support, ticket triage, knowledge base management, and satisfaction tracking",
    "providers": {
      "primary": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.5},
      "planner": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.4},
      "worker": {"provider": "openrouter", "model": "openai/gpt-4o-mini", "temperature": 0.6},
      "allowed_providers": ["openrouter"],
      "forbidden_providers": []
    },
    "extensions": [
      {"name": "github", "enabled": true, "tools": ["list_issues", "create_issue", "add_comment", "update_issue", "assign_issue"]},
      {"name": "agent_mesh", "enabled": true, "tools": ["send_task", "request_approval", "notify", "fetch_status"]},
      {"name": "memory", "enabled": true, "preferences": {"retention_days": 30, "auto_summarize": true, "include_pii": false}}
    ],
    "recipes": [
      {"name": "daily-ticket-summary", "path": "recipes/support/daily-ticket-summary.yaml", "schedule": "0 9 * * 1-5", "enabled": true},
      {"name": "weekly-kb-updates", "path": "recipes/support/weekly-kb-updates.yaml", "schedule": "0 10 * * 5", "enabled": true},
      {"name": "monthly-satisfaction-report", "path": "recipes/support/monthly-satisfaction-report.yaml", "schedule": "0 9 1 * *", "enabled": true}
    ],
    "privacy": {
      "mode": "hybrid",
      "strictness": "strict",
      "allow_override": false,
      "pii_categories": ["EMAIL", "PHONE", "CREDIT_CARD", "ADDRESS"]
    },
    "env_vars": {
      "SESSION_RETENTION_DAYS": "30",
      "PRIVACY_GUARD_MODE": "hybrid",
      "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet",
      "SLA_P0_RESPONSE_HOURS": "1",
      "SLA_P1_RESPONSE_HOURS": "4"
    }
  }'::jsonb,
  NULL
);

-- Legal Profile (LOCAL-ONLY with Ollama)
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'legal',
  'Legal Team Agent',
  '{
    "role": "legal",
    "display_name": "Legal Team Agent",
    "description": "Contract review, compliance, risk assessment, and legal research (ATTORNEY-CLIENT PRIVILEGE - LOCAL ONLY)",
    "providers": {
      "primary": {"provider": "ollama", "model": "llama3.2:latest", "temperature": 0.2},
      "planner": {"provider": "ollama", "model": "llama3.2:latest", "temperature": 0.1},
      "worker": {"provider": "ollama", "model": "llama3.2:latest", "temperature": 0.3},
      "allowed_providers": ["ollama"],
      "forbidden_providers": ["openrouter", "openai", "anthropic", "google", "azure", "bedrock"]
    },
    "extensions": [
      {"name": "agent_mesh", "enabled": true, "tools": ["send_task", "request_approval", "notify", "fetch_status"]},
      {"name": "memory", "enabled": true, "preferences": {"retention_days": 0, "auto_summarize": false, "include_pii": false}}
    ],
    "recipes": [
      {"name": "weekly-compliance-scan", "path": "recipes/legal/weekly-compliance-scan.yaml", "schedule": "0 9 * * 1", "enabled": true},
      {"name": "contract-expiry-alerts", "path": "recipes/legal/contract-expiry-alerts.yaml", "schedule": "0 9 1 * *", "enabled": true},
      {"name": "monthly-risk-assessment", "path": "recipes/legal/monthly-risk-assessment.yaml", "schedule": "0 9 1 * *", "enabled": true}
    ],
    "privacy": {
      "mode": "strict",
      "strictness": "maximum",
      "allow_override": false,
      "local_only": true,
      "pii_categories": ["SSN", "EMAIL", "PHONE", "EMPLOYEE_ID", "ADDRESS", "LEGAL_PARTY", "CASE_NUMBER", "CONTRACT_ID"]
    },
    "env_vars": {
      "SESSION_RETENTION_DAYS": "0",
      "PRIVACY_GUARD_MODE": "strict",
      "DEFAULT_MODEL": "ollama/llama3.2:latest",
      "OLLAMA_URL": "http://localhost:11434"
    }
  }'::jsonb,
  NULL
);

-- Verification Queries
-- Count profiles
SELECT COUNT(*) as total_profiles FROM profiles;

-- List all profiles
SELECT role, display_name, created_at FROM profiles ORDER BY role;

-- Check JSONB data structure
SELECT 
  role, 
  data->'providers'->'primary'->>'model' as primary_model,
  jsonb_array_length(data->'recipes') as recipe_count,
  data->'privacy'->>'mode' as privacy_mode
FROM profiles
ORDER BY role;
