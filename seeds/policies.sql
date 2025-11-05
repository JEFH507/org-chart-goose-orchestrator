-- Phase 5 Workstream C: Policy Seed Data
-- Purpose: Populate policies table with role-based tool access rules
-- Dependencies: profiles table must be populated first (seeds/profiles.sql)

-- Finance Role Policies
-- Finance can use financial tools but not developer tools

-- Allow: Excel MCP (all tools)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('finance', 'excel-mcp__*', true, NULL, 'Finance team needs Excel access for budgets');

-- Allow: GitHub (read-only)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('finance', 'github__list_issues', true, NULL, 'Finance can view issues for budget tracking');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('finance', 'github__get_issue', true, NULL, 'Finance can read issue details');

-- Allow: Agent Mesh (all tools)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('finance', 'agent_mesh__*', true, NULL, 'Finance needs cross-agent communication');

-- Allow: Memory (no PII)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('finance', 'memory__*', true, NULL, 'Finance can use memory tools (PII excluded by profile config)');

-- Deny: Developer tools (no code execution)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('finance', 'developer__shell', false, NULL, 'Finance role cannot execute arbitrary code');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('finance', 'developer__*', false, NULL, 'Finance role cannot use developer tools');

---

-- Manager Role Policies
-- Manager can delegate and approve, but limited GitHub write access

-- Allow: Agent Mesh (all tools - delegation)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('manager', 'agent_mesh__*', true, NULL, 'Manager needs full delegation capabilities');

-- Allow: Memory (full context)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('manager', 'memory__*', true, NULL, 'Manager needs full context for team oversight');

-- Allow: GitHub (read + issue creation)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('manager', 'github__*', true, NULL, 'Manager can manage GitHub issues');

-- Deny: Privacy Guard disable (cannot bypass privacy)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('manager', 'privacy-guard__disable', false, NULL, 'Managers cannot bypass privacy controls');

---

-- Analyst Role Policies
-- Analyst can use data tools with database restrictions

-- Allow: Developer tools (for data analysis scripts)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('analyst', 'developer__*', true, NULL, 'Analyst needs developer tools for data analysis');

-- Allow: Excel MCP
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('analyst', 'excel-mcp__*', true, NULL, 'Analyst needs Excel for data analysis');

-- Allow: SQL MCP (analytics databases only)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('analyst', 'sql-mcp__query', true, '{"database": "analytics_*"}'::jsonb, 'Analyst can query analytics databases only');

-- Allow: Agent Mesh
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('analyst', 'agent_mesh__*', true, NULL, 'Analyst needs cross-agent communication');

-- Allow: Memory
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('analyst', 'memory__*', true, NULL, 'Analyst needs memory for context');

-- Deny: SQL MCP on production databases
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('analyst', 'sql-mcp__query', false, '{"database": "prod_*"}'::jsonb, 'Analyst cannot query production databases');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('analyst', 'sql-mcp__query', false, '{"database": "finance_*"}'::jsonb, 'Analyst cannot query finance databases');

---

-- Marketing Role Policies
-- Marketing can use web scraping and content tools

-- Allow: Web scraper
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('marketing', 'web-scraper__*', true, NULL, 'Marketing needs web scraping for competitive analysis');

-- Allow: Agent Mesh
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('marketing', 'agent_mesh__*', true, NULL, 'Marketing needs cross-agent communication');

-- Allow: Memory
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('marketing', 'memory__*', true, NULL, 'Marketing needs context for campaigns');

-- Allow: GitHub (for content repos)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('marketing', 'github__*', true, NULL, 'Marketing manages content in GitHub');

---

-- Support Role Policies
-- Support can use GitHub issues and agent mesh

-- Allow: GitHub (issue triage)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('support', 'github__*', true, NULL, 'Support triages customer issues in GitHub');

-- Allow: Agent Mesh
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('support', 'agent_mesh__*', true, NULL, 'Support needs cross-agent communication');

-- Allow: Memory (strict privacy for customer data)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('support', 'memory__*', true, NULL, 'Support needs context (PII protected by profile config)');

---

-- Legal Role Policies
-- Legal: LOCAL-ONLY (no cloud providers, attorney-client privilege)

-- Allow: Agent Mesh (local-only communication)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'agent_mesh__*', true, NULL, 'Legal needs internal communication (local-only)');

-- Allow: Memory (zero retention - ephemeral only)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'memory__*', true, NULL, 'Legal memory is ephemeral (retention_days: 0)');

-- Deny: All cloud providers (OpenRouter, OpenAI, Anthropic)
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'provider__openrouter', false, NULL, 'Attorney-client privilege: local-only inference');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'provider__openai', false, NULL, 'Attorney-client privilege: local-only inference');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'provider__anthropic', false, NULL, 'Attorney-client privilege: local-only inference');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'provider__google', false, NULL, 'Attorney-client privilege: local-only inference');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'provider__azure', false, NULL, 'Attorney-client privilege: local-only inference');

INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'provider__bedrock', false, NULL, 'Attorney-client privilege: local-only inference');

-- Deny: Any non-local provider pattern
INSERT INTO policies (role, tool_pattern, allow, conditions, reason)
VALUES ('legal', 'provider__*', false, NULL, 'Legal must use local-only Ollama (attorney-client privilege)');

---

-- Verification queries
DO $$
DECLARE
    policy_count INTEGER;
    role_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO policy_count FROM policies;
    SELECT COUNT(DISTINCT role) INTO role_count FROM policies;
    
    RAISE NOTICE '=== Policy Seed Complete ===';
    RAISE NOTICE 'Total policies: %', policy_count;
    RAISE NOTICE 'Roles with policies: %', role_count;
    RAISE NOTICE '';
    RAISE NOTICE 'Role breakdown:';
    FOR rec IN (SELECT role, COUNT(*) as count FROM policies GROUP BY role ORDER BY role) LOOP
        RAISE NOTICE '  % - % policies', rec.role, rec.count;
    END LOOP;
END $$;

-- Test queries (commented out - uncomment to run manually)
-- SELECT * FROM policies WHERE role = 'finance';
-- SELECT * FROM policies WHERE role = 'analyst' AND conditions IS NOT NULL;
-- SELECT * FROM policies WHERE allow = false;
