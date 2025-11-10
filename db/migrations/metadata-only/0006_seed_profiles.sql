-- Profile Seed Data
-- Generated from YAML files in /profiles/
-- Run after 0002_create_profiles.sql migration
-- This migration is idempotent (ON CONFLICT DO UPDATE)


-- Business Analyst
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'analyst',
  'Business Analyst',
  '{
  "role": "analyst",
  "display_name": "Business Analyst",
  "description": "Data analysis, process optimization, time studies, and KPI tracking",
  "providers": {
    "primary": {
      "provider": "openrouter",
      "model": "openai/gpt-4o",
      "temperature": 0.3
    },
    "planner": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.2
    },
    "worker": {
      "provider": "openrouter",
      "model": "openai/gpt-4o",
      "temperature": 0.4
    },
    "allowed_providers": [
      "openrouter"
    ],
    "forbidden_providers": []
  },
  "extensions": [
    {
      "name": "developer",
      "enabled": true,
      "tools": [
        "shell",
        "text_editor"
      ]
    },
    {
      "name": "excel-mcp",
      "enabled": true
    },
    {
      "name": "sql-mcp",
      "enabled": true
    },
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 90,
        "auto_summarize": true,
        "include_pii": false
      }
    }
  ],
  "goosehints": {
    "global": "# Business Analyst Role Context\nYou are the Business Analyst for the organization.\nYour primary responsibilities are:\n- Data analysis and KPI tracking\n- Process optimization and bottleneck identification\n- Time and motion studies\n- Operational efficiency reporting\n\nWhen performing analysis:\n- Always validate data sources and quality\n- Document methodology and assumptions\n- Provide statistical confidence levels\n- Flag anomalies and outliers\n- Recommend actionable insights\n\nData Sources:\n@analytics/dashboards/kpi-definitions.md\n@analytics/data/operational-metrics.xlsx\n@analytics/sql/reporting-views.sql\n\nKey Metrics to Track:\n- Process cycle time (avg, p50, p95)\n- Throughput (transactions/hour)\n- Error rates (by process, by team)\n- Capacity utilization (%)\n- Bottleneck identification\n\nAnalysis Standards:\n- Use median (p50) for central tendency (robust to outliers)\n- Report p95 for tail latency analysis\n- Always include sample size (n) and confidence intervals\n- Normalize data for apples-to-apples comparisons\n- Use control charts for trend detection\n\nReporting Guidelines:\n- Executive summary: Key findings + recommendations (3-5 bullets)\n- Methodology: Data sources, time period, filters\n- Detailed analysis: Charts, tables, statistical tests\n- Insights: Actionable recommendations with priority\n- Next steps: Follow-up analysis or actions\n"
  },
  "gooseignore": {
    "global": "# Analyst-specific privacy (operational data)\n**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n\n# Employee personal data (anonymize for analysis)\n**/employee_personal_*\n**/employee_names_*\n**/performance_reviews_*\n**/salary_*\n**/compensation_*\n\n# Database credentials\n**/db_credentials.*\n**/database_production.*\n",
    "local_templates": [
      {
        "path": "analytics/reports",
        "content": "# Analytics-specific exclusions\n**/raw_employee_data.*\n**/unanonymized_*\n**/pii_records.*\n"
      }
    ]
  },
  "recipes": [
    {
      "name": "daily-kpi-report",
      "description": "Daily KPI dashboard report - weekday mornings",
      "path": "recipes/analyst/daily-kpi-report.yaml",
      "schedule": "0 9 * * 1-5",
      "enabled": true
    },
    {
      "name": "process-bottleneck-analysis",
      "description": "Weekly process bottleneck identification and analysis",
      "path": "recipes/analyst/process-bottleneck-analysis.yaml",
      "schedule": "0 10 * * 1",
      "enabled": true
    },
    {
      "name": "time-study-analysis",
      "description": "Monthly time and motion study analysis",
      "path": "recipes/analyst/time-study-analysis.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    }
  ],
  "automated_tasks": [
    {
      "name": "hourly-metrics-collection",
      "recipe": "recipes/analyst/hourly-metrics.yaml",
      "schedule": "0 * * * *",
      "enabled": true,
      "notify_on_failure": true
    }
  ],
  "policies": [
    {
      "allow_tool": "excel-mcp__*",
      "reason": "Analyst needs spreadsheet operations"
    },
    {
      "allow_tool": "sql-mcp__query",
      "conditions": [
        {
          "database": "analytics_ro"
        }
      ],
      "reason": "Analyst needs data extraction (read-only)"
    },
    {
      "allow_tool": "developer__shell",
      "conditions": [
        {
          "allowed_commands": [
            "python",
            "Rscript",
            "awk",
            "sed",
            "grep",
            "sort",
            "uniq"
          ]
        }
      ],
      "reason": "Analyst needs data processing scripts"
    },
    {
      "allow_tool": "developer__text_editor",
      "reason": "Analyst creates analysis scripts and reports"
    },
    {
      "allow_tool": "agent_mesh__notify",
      "reason": "Analyst notifies stakeholders of findings"
    },
    {
      "deny_tool": "sql-mcp__execute",
      "reason": "No write operations to production databases"
    },
    {
      "deny_tool": "github__create_pr",
      "reason": "Analyst doesn''t push code changes"
    }
  ],
  "privacy": {
    "mode": "hybrid",
    "strictness": "moderate",
    "allow_override": true,
    "rules": [
      {
        "pattern": "\\b[A-Z]{2}\\d{6,8}\\b",
        "replacement": "[EMP_ID]",
        "category": "EMPLOYEE_ID"
      },
      {
        "pattern": "\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z|a-z]{2,}\\b",
        "replacement": "[EMAIL]",
        "category": "EMAIL"
      },
      {
        "pattern": "\\b\\d{3}[-.\\s]?\\d{3}[-.\\s]?\\d{4}\\b",
        "replacement": "[PHONE]",
        "category": "PHONE"
      },
      {
        "pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b",
        "replacement": "[SSN]",
        "category": "SSN"
      }
    ],
    "pii_categories": [
      "EMPLOYEE_ID",
      "EMAIL",
      "PHONE",
      "SSN"
    ]
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "90",
    "PRIVACY_GUARD_MODE": "hybrid",
    "DEFAULT_MODEL": "openrouter/openai/gpt-4o",
    "ANALYTICS_DB": "analytics_ro",
    "REPORT_OUTPUT_DIR": "analytics/reports",
    "MIN_SAMPLE_SIZE": "30"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();


-- Developer Team Agent
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'developer',
  'Developer Team Agent',
  '{
  "role": "developer",
  "display_name": "Developer Team Agent",
  "description": "Software development, code review, debugging, and technical implementation",
  "providers": {
    "primary": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.3
    },
    "planner": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.2
    },
    "worker": {
      "provider": "openrouter",
      "model": "openai/gpt-4o",
      "temperature": 0.4
    },
    "allowed_providers": [
      "openrouter"
    ],
    "forbidden_providers": []
  },
  "extensions": [
    {
      "name": "github",
      "enabled": true,
      "tools": [
        "list_issues",
        "create_issue",
        "add_comment",
        "create_pr",
        "review_pr",
        "merge_pr"
      ]
    },
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "request_approval",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 90,
        "auto_summarize": true,
        "include_pii": false
      }
    },
    {
      "name": "developer",
      "enabled": true,
      "tools": [
        "shell",
        "text_editor",
        "analyze"
      ]
    }
  ],
  "goosehints": {
    "global": "# Developer Role Context\nYou are a software developer agent for the organization.\nYour primary responsibilities are:\n- Implement features and fix bugs\n- Code review and quality assurance\n- Write and maintain tests\n- Debug production issues\n- Technical documentation\n\nWhen writing code:\n- Follow project coding standards and conventions\n- Write comprehensive tests (unit, integration, e2e)\n- Document complex logic with comments\n- Create descriptive commit messages (conventional commits)\n- Request code review before merging\n\nDevelopment Sources:\n@docs/contributing.md\n@docs/architecture/\n@README.md\n\nKey Responsibilities:\n- Feature implementation from approved specs\n- Bug fixes and debugging\n- Code review for team PRs\n- Test coverage maintenance (>80%)\n- CI/CD pipeline maintenance\n- Technical debt reduction\n\nCode Quality Standards:\n- Test coverage: >80% for new code\n- Linting: Zero errors, minimal warnings\n- Security: No secrets in code, dependency scanning\n- Performance: Profile before optimizing\n- Documentation: All public APIs documented\n\nGit Workflow:\n- Branch naming: feature/*, fix/*, chore/*, docs/*\n- Commit format: conventional commits (feat:, fix:, docs:, etc.)\n- PR size: <400 lines preferred, <1000 max\n- Review required: 1 approval minimum\n- CI must pass: All tests green before merge\n\nTech Stack:\n- Backend: Rust 1.83+, Axum, SQLx, Tokio\n- Frontend: React, TypeScript, Vite\n- Database: PostgreSQL 17, Redis 7\n- Infrastructure: Docker, Docker Compose\n- CI/CD: GitHub Actions\n- Security: Vault, Keycloak, Privacy Guard\n"
  },
  "gooseignore": {
    "global": "# Developer-specific privacy\n**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n**/.git/\n**/node_modules/\n**/target/\n**/.venv/\n**/dist/\n**/build/\n\n# Sensitive data patterns\n**/test-data/production_*\n**/fixtures/customer_*\n**/mock-data/real_*\n",
    "local_templates": [
      {
        "path": "backend",
        "content": "# Backend exclusions\n**/migrations/production_*\n**/seeds/production_*\n"
      },
      {
        "path": "frontend",
        "content": "# Frontend exclusions\n**/.env.production\n**/public/analytics_*\n"
      }
    ]
  },
  "recipes": [
    {
      "name": "daily-pr-review",
      "description": "Daily automated code review for open PRs",
      "path": "recipes/developer/daily-pr-review.yaml",
      "schedule": "0 10 * * 1-5",
      "enabled": true
    },
    {
      "name": "weekly-tech-debt",
      "description": "Weekly technical debt assessment and prioritization",
      "path": "recipes/developer/weekly-tech-debt.yaml",
      "schedule": "0 14 * * 5",
      "enabled": true
    },
    {
      "name": "monthly-dependency-audit",
      "description": "Monthly dependency version and security audit",
      "path": "recipes/developer/monthly-dependency-audit.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    }
  ],
  "automated_tasks": [
    {
      "name": "morning-build-check",
      "recipe": "recipes/developer/morning-build-check.yaml",
      "schedule": "0 8 * * 1-5",
      "enabled": true,
      "notify_on_failure": true
    }
  ],
  "policies": [
    {
      "allow_tool": "developer__*",
      "reason": "Developer role needs code execution capabilities"
    },
    {
      "allow_tool": "github__*",
      "reason": "Developer needs full GitHub access for development workflow"
    },
    {
      "allow_tool": "agent_mesh__*",
      "reason": "Developer coordinates with Manager for approvals and Support for bugs"
    },
    {
      "allow_tool": "memory__*",
      "reason": "Developer needs context for code changes"
    },
    {
      "allow_tool": "database__query",
      "conditions": [
        {
          "environment": "development"
        },
        {
          "environment": "staging"
        }
      ],
      "reason": "Developer can query dev/staging databases, not production"
    },
    {
      "deny_tool": "database__execute",
      "conditions": [
        {
          "environment": "production"
        }
      ],
      "reason": "No direct production database writes (use migrations)"
    },
    {
      "deny_tool": "privacy-guard__disable",
      "reason": "Cannot bypass privacy protection"
    }
  ],
  "privacy": {
    "mode": "hybrid",
    "strictness": "moderate",
    "allow_override": true,
    "rules": [
      {
        "pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b",
        "replacement": "[SSN]",
        "category": "SSN"
      },
      {
        "pattern": "\\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}\\b",
        "replacement": "[EMAIL]",
        "category": "EMAIL"
      },
      {
        "pattern": "\\b[A-Z]{2}\\d{6,8}\\b",
        "replacement": "[EMP_ID]",
        "category": "EMPLOYEE_ID"
      },
      {
        "pattern": "eyJ[A-Za-z0-9-_]+\\.eyJ[A-Za-z0-9-_]+\\.[A-Za-z0-9-_]+",
        "replacement": "[JWT_TOKEN]",
        "category": "JWT"
      },
      {
        "pattern": "(sk-|pk_live_|pk_test_)[A-Za-z0-9]{20,}",
        "replacement": "[API_KEY]",
        "category": "API_KEY"
      }
    ],
    "pii_categories": [
      "SSN",
      "EMAIL",
      "PHONE",
      "EMPLOYEE_ID",
      "JWT",
      "API_KEY",
      "PASSWORD"
    ],
    "retention_days": 90
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "90",
    "PRIVACY_GUARD_MODE": "hybrid",
    "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet",
    "CODE_REVIEW_AUTO_APPROVE": "false",
    "TEST_COVERAGE_THRESHOLD": "80",
    "MAX_PR_SIZE_LINES": "1000"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();


-- Finance Team Agent
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'finance',
  'Finance Team Agent',
  '{
  "role": "finance",
  "display_name": "Finance Team Agent",
  "description": "Budget approvals, compliance reporting, financial analysis, and regulatory oversight",
  "providers": {
    "primary": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.3
    },
    "planner": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.2
    },
    "worker": {
      "provider": "openrouter",
      "model": "openai/gpt-4o-mini",
      "temperature": 0.4
    },
    "allowed_providers": [
      "openrouter"
    ],
    "forbidden_providers": []
  },
  "extensions": [
    {
      "name": "github",
      "enabled": true,
      "tools": [
        "list_issues",
        "create_issue",
        "add_comment"
      ]
    },
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "request_approval",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 90,
        "auto_summarize": true,
        "include_pii": false
      }
    },
    {
      "name": "excel-mcp",
      "enabled": true
    }
  ],
  "goosehints": {
    "global": "# Finance Role Context\nYou are the Finance team agent for the organization.\nYour primary responsibilities are:\n- Budget compliance and spend tracking\n- Regulatory reporting (SOX, GAAP)\n- Financial forecasting and variance analysis\n- Approval workflows for budget requests\n\nWhen analyzing budgets:\n- Always verify budget availability before approving spend requests\n- Document all approval decisions with rationale\n- Flag unusual spending patterns for review\n- Maintain audit trail for compliance\n\nFinancial Data Sources:\n@finance/policies/approval-matrix.md\n@finance/budgets/fy2026-budget.xlsx\n\nCompliance Requirements:\n- All spend >$10K requires Manager approval\n- All spend >$50K requires Finance + Manager approval\n- Quarterly variance reports due on 5th business day\n- Monthly close process documented in runbook\n\nKey Metrics to Track:\n- Budget utilization % by department\n- Burn rate vs forecast\n- Variance to plan (\u00b15% threshold)\n- Days cash on hand\n"
  },
  "gooseignore": {
    "global": "# Financial data - NEVER share these patterns\n**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n**/config/production.*\n\n# Finance-specific exclusions\n**/salary_data.*\n**/bonus_plans.*\n**/tax_records.*\n**/employee_compensation.*\n**/payroll_*\n**/ssn_*\n**/banking_credentials.*\n",
    "local_templates": [
      {
        "path": "finance/budgets",
        "content": "# Budget-specific exclusions\n**/employee_salaries.*\n**/bonus_data.*\n**/executive_comp.*\n"
      },
      {
        "path": "finance/audits",
        "content": "# Audit-specific exclusions\n**/sox_controls.*\n**/audit_findings.*\n**/remediation_plans.*\n"
      }
    ]
  },
  "recipes": [
    {
      "name": "monthly-budget-close",
      "description": "Automated monthly budget close process - runs on 5th business day",
      "path": "recipes/finance/monthly-budget-close.yaml",
      "schedule": "0 9 5 * *",
      "enabled": true
    },
    {
      "name": "weekly-spend-report",
      "description": "Weekly departmental spend summary and variance analysis",
      "path": "recipes/finance/weekly-spend-report.yaml",
      "schedule": "0 10 * * 1",
      "enabled": true
    },
    {
      "name": "quarterly-forecast",
      "description": "Quarterly financial forecast and budget reforecast",
      "path": "recipes/finance/quarterly-forecast.yaml",
      "schedule": "0 9 1 1,4,7,10 *",
      "enabled": true
    }
  ],
  "automated_tasks": [
    {
      "name": "daily-spend-alerts",
      "recipe": "recipes/finance/daily-spend-alerts.yaml",
      "schedule": "0 8 * * 1-5",
      "enabled": true,
      "notify_on_failure": true
    }
  ],
  "policies": [
    {
      "allow_tool": "excel-mcp__*",
      "reason": "Finance needs spreadsheet operations"
    },
    {
      "allow_tool": "github__list_issues",
      "conditions": [
        {
          "repo": "finance/*"
        }
      ],
      "reason": "Read budget tracking issues"
    },
    {
      "allow_tool": "github__create_issue",
      "conditions": [
        {
          "repo": "finance/budget-requests"
        }
      ],
      "reason": "Create budget request issues"
    },
    {
      "allow_tool": "agent_mesh__*",
      "reason": "Finance routes approval workflows"
    },
    {
      "deny_tool": "developer__shell",
      "reason": "No arbitrary code execution for Finance role"
    },
    {
      "deny_tool": "developer__exec",
      "reason": "No arbitrary code execution for Finance role"
    },
    {
      "deny_tool": "sql-mcp__query",
      "reason": "Finance should not run arbitrary SQL (use read-only views)"
    }
  ],
  "privacy": {
    "mode": "hybrid",
    "strictness": "strict",
    "allow_override": false,
    "rules": [
      {
        "pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b",
        "replacement": "[SSN]",
        "category": "SSN"
      },
      {
        "pattern": "\\b[A-Z]{2}\\d{6,8}\\b",
        "replacement": "[EMP_ID]",
        "category": "EMPLOYEE_ID"
      },
      {
        "pattern": "\\b\\d{4}[- ]?\\d{4}[- ]?\\d{4}[- ]?\\d{4}\\b",
        "replacement": "[CREDIT_CARD]",
        "category": "CREDIT_CARD"
      },
      {
        "pattern": "\\b\\d{9}\\b",
        "replacement": "[ROUTING_NUM]",
        "category": "ROUTING_NUMBER"
      },
      {
        "pattern": "\\$\\d{1,3}(,\\d{3})*(\\.\\d{2})?(?=\\s*(salary|compensation|bonus))",
        "replacement": "[SALARY_AMOUNT]",
        "category": "COMPENSATION"
      }
    ],
    "pii_categories": [
      "SSN",
      "EMAIL",
      "PHONE",
      "EMPLOYEE_ID",
      "CREDIT_CARD",
      "ROUTING_NUMBER",
      "COMPENSATION"
    ]
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "90",
    "PRIVACY_GUARD_MODE": "hybrid",
    "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet",
    "BUDGET_APPROVAL_THRESHOLD": "10000",
    "FINANCE_MANAGER_THRESHOLD": "50000"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();


-- HR Team Agent
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'hr',
  'HR Team Agent',
  '{
  "role": "hr",
  "display_name": "HR Team Agent",
  "description": "Employee relations, benefits administration, compliance, and onboarding workflows",
  "providers": {
    "primary": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.3
    },
    "planner": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.2
    },
    "worker": {
      "provider": "openrouter",
      "model": "openai/gpt-4o-mini",
      "temperature": 0.4
    },
    "allowed_providers": [
      "openrouter"
    ],
    "forbidden_providers": []
  },
  "extensions": [
    {
      "name": "github",
      "enabled": true,
      "tools": [
        "list_issues",
        "create_issue",
        "add_comment"
      ]
    },
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "request_approval",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 180,
        "auto_summarize": true,
        "include_pii": false
      }
    }
  ],
  "goosehints": {
    "global": "# HR Role Context\nYou are the HR team agent for the organization.\nYour primary responsibilities are:\n- Employee onboarding and offboarding\n- Benefits enrollment and administration\n- Compliance monitoring (labor laws, policies)\n- Employee relations and conflict resolution\n- Performance review coordination\n\nWhen handling HR workflows:\n- Always maintain strict confidentiality\n- Follow compliance requirements (EEOC, ADA, FMLA, etc.)\n- Document all employee interactions appropriately\n- Route sensitive issues to Legal when needed\n- Maintain audit trail for compliance\n\nHR Data Sources:\n@hr/policies/employee-handbook.md\n@hr/compliance/labor-regulations.md\n@hr/processes/onboarding-checklist.md\n\nKey Responsibilities:\n- New hire onboarding (I9, benefits, equipment)\n- Benefits open enrollment coordination\n- Leave administration (PTO, FMLA, parental)\n- Performance review cycle management\n- Employee relations investigations\n- Compliance training tracking\n\nCompliance Requirements:\n- All employee data must be Privacy Guard protected\n- No PII in chat logs or external systems\n- Manager approval required for disciplinary actions\n- Legal review required for terminations\n- Document retention: 7 years for employee records\n\nKey Metrics to Track:\n- Time to hire (target: <30 days)\n- Employee satisfaction scores\n- Benefits enrollment completion rate\n- Compliance training completion %\n- Turnover rate by department\n"
  },
  "gooseignore": {
    "global": "# HR-specific privacy (HIGHEST SENSITIVITY)\n**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n\n# Employee personal data - NEVER share\n**/employee_records_*\n**/ssn_*\n**/salary_*\n**/compensation_*\n**/performance_*\n**/disciplinary_*\n**/medical_*\n**/benefits_*\n**/background_check_*\n**/i9_*\n**/tax_*\n",
    "local_templates": [
      {
        "path": "hr/employee-files",
        "content": "# Employee file exclusions\n**/personnel_file_*\n**/pip_*\n**/termination_*\n**/investigation_*\n"
      },
      {
        "path": "hr/compliance",
        "content": "# Compliance data exclusions\n**/eeoc_*\n**/ada_*\n**/fmla_*\n**/workers_comp_*\n"
      }
    ]
  },
  "recipes": [
    {
      "name": "weekly-onboarding-check",
      "description": "Weekly check on new hire onboarding progress",
      "path": "recipes/hr/weekly-onboarding-check.yaml",
      "schedule": "0 9 * * 1",
      "enabled": true
    },
    {
      "name": "monthly-compliance-report",
      "description": "Monthly compliance training and certification status",
      "path": "recipes/hr/monthly-compliance-report.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    },
    {
      "name": "quarterly-benefits-summary",
      "description": "Quarterly benefits utilization and enrollment summary",
      "path": "recipes/hr/quarterly-benefits-summary.yaml",
      "schedule": "0 9 1 1,4,7,10 *",
      "enabled": true
    }
  ],
  "automated_tasks": [
    {
      "name": "daily-leave-alerts",
      "recipe": "recipes/hr/daily-leave-alerts.yaml",
      "schedule": "0 8 * * 1-5",
      "enabled": true,
      "notify_on_failure": true
    }
  ],
  "policies": [
    {
      "allow_tool": "github__*",
      "reason": "HR needs GitHub for policy documentation and workflows"
    },
    {
      "allow_tool": "agent_mesh__*",
      "reason": "HR coordinates cross-functional processes (Legal, Manager, Finance)"
    },
    {
      "allow_tool": "memory__*",
      "reason": "HR needs context for employee relations"
    },
    {
      "allow_tool": "database__query",
      "conditions": [
        {
          "table": "employees"
        },
        {
          "operation": "SELECT"
        }
      ],
      "reason": "Read-only access to employee data"
    },
    {
      "deny_tool": "developer__shell",
      "reason": "No arbitrary code execution for HR role"
    },
    {
      "deny_tool": "developer__exec",
      "reason": "No arbitrary code execution for HR role"
    },
    {
      "deny_tool": "privacy-guard__disable",
      "reason": "Cannot bypass privacy protection for employee data"
    }
  ],
  "privacy": {
    "mode": "hybrid",
    "strictness": "strict",
    "allow_override": false,
    "rules": [
      {
        "pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b",
        "replacement": "[SSN]",
        "category": "SSN"
      },
      {
        "pattern": "\\b[A-Z]{2}\\d{6,8}\\b",
        "replacement": "[EMP_ID]",
        "category": "EMPLOYEE_ID"
      },
      {
        "pattern": "\\b\\d{4}[- ]?\\d{4}[- ]?\\d{4}[- ]?\\d{4}\\b",
        "replacement": "[CREDIT_CARD]",
        "category": "CREDIT_CARD"
      },
      {
        "pattern": "\\$\\d{1,3}(,\\d{3})*(\\.\\d{2})?(?=\\s*(salary|compensation|bonus|raise))",
        "replacement": "[SALARY_AMOUNT]",
        "category": "COMPENSATION"
      },
      {
        "pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b|\\b\\d{2}-\\d{7}\\b",
        "replacement": "[TAX_ID]",
        "category": "TAX_ID"
      }
    ],
    "pii_categories": [
      "SSN",
      "EMAIL",
      "PHONE",
      "EMPLOYEE_ID",
      "CREDIT_CARD",
      "COMPENSATION",
      "TAX_ID",
      "MEDICAL",
      "ADDRESS"
    ],
    "retention_days": 2555
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "180",
    "PRIVACY_GUARD_MODE": "hybrid",
    "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet",
    "COMPLIANCE_RETENTION_YEARS": "7",
    "MAX_EMPLOYEE_RECORDS_PER_QUERY": "100"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();


-- Legal Team Agent
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'legal',
  'Legal Team Agent',
  '{
  "role": "legal",
  "display_name": "Legal Team Agent",
  "description": "Contract review, compliance monitoring, risk assessment, and legal advisory - ATTORNEY-CLIENT PRIVILEGE PROTECTED",
  "providers": {
    "primary": {
      "provider": "ollama",
      "model": "llama3.2",
      "temperature": 0.2,
      "base_url": "http://localhost:11434"
    },
    "planner": {
      "provider": "ollama",
      "model": "llama3.2",
      "temperature": 0.1,
      "base_url": "http://localhost:11434"
    },
    "worker": {
      "provider": "ollama",
      "model": "llama3.2",
      "temperature": 0.2,
      "base_url": "http://localhost:11434"
    },
    "allowed_providers": [
      "ollama"
    ],
    "forbidden_providers": [
      "openrouter",
      "openai",
      "anthropic",
      "google",
      "cohere",
      "azure"
    ]
  },
  "extensions": [
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "request_approval",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 0,
        "auto_summarize": false,
        "include_pii": false,
        "local_only": true
      }
    }
  ],
  "goosehints": {
    "global": "# Legal Role Context - ATTORNEY-CLIENT PRIVILEGE PROTECTED\nYou are the Legal team agent for the organization.\n\n\u2696\ufe0f CRITICAL: All communications and documents are subject to attorney-client privilege.\nYou operate EXCLUSIVELY on local infrastructure (Ollama). NO cloud providers permitted.\n\nYour primary responsibilities are:\n- Contract review and negotiation support\n- Compliance monitoring and risk assessment\n- Legal advisory and policy development\n- Regulatory analysis and reporting\n\n## Attorney-Client Privilege Requirements\n\nNEVER:\n- Send any legal documents to cloud providers\n- Share privileged communications externally\n- Store legal data in non-ephemeral memory\n- Include case details in GitHub issues\n- Reference specific legal matters in public channels\n\nALWAYS:\n- Keep all legal work product local\n- Use generic references when coordinating with other roles\n- Redact identifying information before sharing\n- Verify local-only operation before processing documents\n- Maintain strict confidentiality\n\n## Legal Review Process\n\nWhen reviewing contracts:\n1. Verify document is stored locally (never in cloud repos)\n2. Analyze key terms: liability, indemnification, IP, termination\n3. Identify risk areas and red flags\n4. Document recommendations (stored locally only)\n5. Coordinate redlined versions with business stakeholders\n6. Never share contract text with cloud LLMs\n\n## Compliance Monitoring\n\nWeekly compliance scans:\n- Review public-facing documentation for compliance issues\n- Monitor regulatory changes affecting the organization\n- Flag potential compliance risks\n- Coordinate with Finance for SOX/audit requirements\n\nMonthly risk assessments:\n- Identify emerging legal risks\n- Review incident reports\n- Update risk register\n- Recommend policy updates\n\n## Key Metrics to Track\n\n- Contract review turnaround time (target: <5 business days)\n- Compliance issues identified (trend over time)\n- Contract expiration tracking (90-day notice)\n- Policy review currency (annual update cycle)\n\n## Communication Protocol\n\nWhen coordinating with other roles:\n- Use generic references: \"Contract A\", \"Matter 2024-03\"\n- Never share specific legal terms or case details\n- Escalate through approved channels only\n- Confirm local-only status before document processing\n\n## Tool Restrictions\n\nALLOWED (Local Only):\n- agent_mesh: Internal coordination (redacted communications)\n- memory: Ephemeral session context (retention_days: 0)\n- local file operations: Contract review, document analysis\n\nFORBIDDEN (Privacy Violations):\n- Any cloud LLM providers (OpenAI, Anthropic, etc.)\n- GitHub operations (legal docs never in version control)\n- SQL queries (legal data in separate, air-gapped systems)\n- Web scraping (use approved legal research platforms)\n"
  },
  "gooseignore": {
    "global": "# LEGAL TEAM GLOBAL IGNORE - ATTORNEY-CLIENT PRIVILEGE\n# These patterns protect privileged and confidential legal materials\n\n# ALL legal directories - NEVER share\n**/legal/**\n**/Legal/**\n**/contracts/**\n**/Contracts/**\n**/agreements/**\n**/litigation/**\n**/Litigation/**\n**/legal_memos/**\n**/legal-memos/**\n**/attorney_work_product/**\n**/privileged/**\n**/Privileged/**\n\n# Contract files - ALL formats\n**/contract_*\n**/Contract_*\n**/*_contract.*\n**/*_Contract.*\n**/*agreement.*\n**/*Agreement.*\n**/NDA_*\n**/nda_*\n**/*_NDA.*\n**/*_nda.*\n**/MSA_*\n**/msa_*\n**/SOW_*\n**/sow_*\n\n# Legal memos and opinions\n**/legal_memo_*\n**/Legal_Memo_*\n**/legal_opinion_*\n**/attorney_memo_*\n**/counsel_memo_*\n**/privilege_log.*\n\n# Litigation and disputes\n**/litigation_*\n**/Litigation_*\n**/lawsuit_*\n**/complaint_*\n**/Complaint_*\n**/discovery_*\n**/deposition_*\n**/settlement_*\n**/Settlement_*\n**/arbitration_*\n**/mediation_*\n\n# Regulatory and compliance - confidential\n**/regulatory_filing_*\n**/sec_filing_*\n**/ftc_*\n**/doj_*\n**/investigation_*\n**/subpoena_*\n**/warrant_*\n\n# IP and proprietary\n**/patent_*\n**/Patent_*\n**/trademark_*\n**/copyright_*\n**/trade_secret_*\n**/ip_assignment_*\n**/invention_disclosure_*\n\n# Employment legal\n**/employment_agreement_*\n**/Employment_Agreement_*\n**/severance_*\n**/Severance_*\n**/termination_agreement_*\n**/non_compete_*\n**/non_solicitation_*\n\n# Corporate governance\n**/board_minutes_*\n**/Board_Minutes_*\n**/shareholder_agreement_*\n**/stockholder_*\n**/bylaws_*\n**/articles_incorporation_*\n\n# Insurance and claims\n**/insurance_claim_*\n**/claim_*\n**/insurance_policy_*\n**/d_and_o_*\n**/liability_claim_*\n\n# Due diligence\n**/due_diligence_*\n**/Due_Diligence_*\n**/dd_*\n**/diligence_report_*\n\n# Confidential communications\n**/attorney_client_*\n**/privileged_communication_*\n**/outside_counsel_*\n**/legal_advice_*\n\n# Standard security patterns\n**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n**/*.pem\n**/*.key\n**/.secrets/\n\n# Backup files (may contain privileged data)\n**/*.bak\n**/*.backup\n**/backup_*\n**/archive_legal_*\n\n# Temporary files (may contain drafts)\n**/*.tmp\n**/*.temp\n**/~$*\n**/.~*\n",
    "local_templates": [
      {
        "path": "legal/contracts",
        "content": "# All contract files - attorney-client privilege\n*.*\n"
      },
      {
        "path": "legal/litigation",
        "content": "# All litigation files - attorney-client privilege\n*.*\n"
      },
      {
        "path": "legal/compliance",
        "content": "# Compliance-specific exclusions\n**/audit_findings.*\n**/remediation_*\n**/violation_*\n"
      }
    ]
  },
  "recipes": [
    {
      "name": "weekly-compliance-scan",
      "description": "Weekly compliance monitoring of public documentation and policies",
      "path": "recipes/legal/weekly-compliance-scan.yaml",
      "schedule": "0 9 * * 1",
      "enabled": true
    },
    {
      "name": "contract-expiry-alerts",
      "description": "Monthly contract expiration tracking and renewal alerts (90-day notice)",
      "path": "recipes/legal/contract-expiry-alerts.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    },
    {
      "name": "monthly-risk-assessment",
      "description": "Monthly legal risk assessment and policy review",
      "path": "recipes/legal/monthly-risk-assessment.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    }
  ],
  "automated_tasks": [
    {
      "name": "weekly-compliance-scan",
      "recipe": "recipes/legal/weekly-compliance-scan.yaml",
      "schedule": "0 9 * * 1",
      "enabled": true,
      "notify_on_failure": true
    },
    {
      "name": "contract-expiry-alerts",
      "recipe": "recipes/legal/contract-expiry-alerts.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true,
      "notify_on_failure": true
    }
  ],
  "policies": [
    {
      "allow_tool": "agent_mesh__send_task",
      "conditions": [
        {
          "content_type": "redacted"
        }
      ],
      "reason": "Legal coordinates with other roles using redacted communications"
    },
    {
      "allow_tool": "agent_mesh__notify",
      "conditions": [
        {
          "content_type": "redacted"
        }
      ],
      "reason": "Legal sends notifications without privileged details"
    },
    {
      "allow_tool": "memory__*",
      "conditions": [
        {
          "retention_days": 0
        }
      ],
      "reason": "Legal uses ephemeral memory for session context"
    },
    {
      "deny_tool": "github__*",
      "reason": "Legal documents never stored in version control (attorney-client privilege)"
    },
    {
      "deny_tool": "web_scrape__*",
      "reason": "Legal uses approved legal research platforms, not generic web scraping"
    },
    {
      "deny_tool": "developer__shell",
      "reason": "No arbitrary code execution for Legal role (security)"
    },
    {
      "deny_tool": "sql-mcp__*",
      "reason": "Legal data in air-gapped systems, no SQL access"
    },
    {
      "deny_provider": "openai",
      "reason": "Attorney-client privilege - no cloud providers"
    },
    {
      "deny_provider": "anthropic",
      "reason": "Attorney-client privilege - no cloud providers"
    },
    {
      "deny_provider": "openrouter",
      "reason": "Attorney-client privilege - no cloud providers"
    },
    {
      "deny_provider": "google",
      "reason": "Attorney-client privilege - no cloud providers"
    },
    {
      "deny_provider": "azure",
      "reason": "Attorney-client privilege - no cloud providers"
    }
  ],
  "privacy": {
    "mode": "strict",
    "strictness": "maximum",
    "allow_override": false,
    "local_only": true,
    "retention_days": 0,
    "rules": [
      {
        "pattern": "\\b(?:Contract|Agreement|NDA|MSA|SOW)\\s+#?\\d{4,}\\b",
        "replacement": "[CONTRACT_ID]",
        "category": "CONTRACT_IDENTIFIER"
      },
      {
        "pattern": "\\b(?:Case|Matter|Litigation)\\s+#?\\d{2,4}-\\d{2,4}\\b",
        "replacement": "[CASE_NUMBER]",
        "category": "CASE_NUMBER"
      },
      {
        "pattern": "\\b(?:Attorney|Counsel|Law Firm):\\s+[A-Z][a-z]+(?:\\s+[A-Z][a-z]+){1,3}\\b",
        "replacement": "[ATTORNEY_NAME]",
        "category": "ATTORNEY"
      },
      {
        "pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b",
        "replacement": "[SSN]",
        "category": "SSN"
      },
      {
        "pattern": "\\b[A-Z]{2}\\d{6,8}\\b",
        "replacement": "[EMP_ID]",
        "category": "EMPLOYEE_ID"
      },
      {
        "pattern": "\\$\\d{1,3}(,\\d{3})*(\\.\\d{2})?",
        "replacement": "[AMOUNT]",
        "category": "CONTRACT_AMOUNT"
      },
      {
        "pattern": "\\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\\s+\\d{1,2},\\s+\\d{4}\\b",
        "replacement": "[DATE]",
        "category": "LITIGATION_DATE"
      },
      {
        "pattern": "\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z|a-z]{2,}\\b",
        "replacement": "[EMAIL]",
        "category": "EMAIL"
      },
      {
        "pattern": "\\b(?:Party A|Party B|Buyer|Seller|Vendor|Client):\\s+[A-Z][a-z]+(?:\\s+[A-Z][a-z]+){0,3}\\s+(?:Inc|LLC|Corp|Ltd)\\.?\\b",
        "replacement": "[PARTY_NAME]",
        "category": "CONTRACT_PARTY"
      }
    ],
    "pii_categories": [
      "CONTRACT_IDENTIFIER",
      "CASE_NUMBER",
      "ATTORNEY",
      "SSN",
      "EMAIL",
      "PHONE",
      "EMPLOYEE_ID",
      "CONTRACT_AMOUNT",
      "LITIGATION_DATE",
      "CONTRACT_PARTY"
    ]
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "0",
    "PRIVACY_GUARD_MODE": "strict",
    "LOCAL_ONLY_ENFORCEMENT": "true",
    "DEFAULT_MODEL": "ollama/llama3.2",
    "OLLAMA_BASE_URL": "http://localhost:11434",
    "ATTORNEY_CLIENT_PRIVILEGE": "enforced",
    "CLOUD_PROVIDERS_FORBIDDEN": "true"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();


-- Manager Team Agent
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'manager',
  'Manager Team Agent',
  '{
  "role": "manager",
  "display_name": "Manager Team Agent",
  "description": "Team oversight, approval workflows, delegation, and cross-functional coordination",
  "providers": {
    "primary": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.4
    },
    "planner": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.3
    },
    "worker": {
      "provider": "openrouter",
      "model": "openai/gpt-4o",
      "temperature": 0.5
    },
    "allowed_providers": [
      "openrouter"
    ],
    "forbidden_providers": []
  },
  "extensions": [
    {
      "name": "github",
      "enabled": true,
      "tools": [
        "list_issues",
        "create_issue",
        "add_comment",
        "update_issue",
        "assign_issue"
      ]
    },
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "request_approval",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 90,
        "auto_summarize": true,
        "include_pii": false
      }
    }
  ],
  "goosehints": {
    "global": "# Manager Role Context\nYou are a team manager for the organization.\nYour primary responsibilities are:\n- Team oversight and performance management\n- Approval workflows (budget, time off, hiring)\n- Delegation and task routing\n- Cross-functional coordination\n\nWhen managing workflows:\n- Always document approval decisions with rationale\n- Route tasks to appropriate team members/roles\n- Track team capacity and workload balance\n- Escalate blockers proactively\n\nTeam Management Sources:\n@team/handbook.md\n@team/processes/approval-workflows.md\n\nKey Responsibilities:\n- Approve budget requests <$50K (Finance approves >$50K)\n- Approve time-off requests\n- Delegate tasks to team members\n- Weekly team sync and standup summaries\n- Monthly 1-on-1 preparation\n- Quarterly performance reviews\n\nDecision-Making Framework:\n1. Gather context (what, why, who, when)\n2. Assess impact (team, budget, timeline)\n3. Consider alternatives\n4. Make decision with rationale\n5. Communicate clearly\n6. Follow up on execution\n"
  },
  "gooseignore": {
    "global": "# Manager-specific privacy (team data)\n**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n\n# Team member personal data\n**/employee_personal_*\n**/performance_reviews_*\n**/salary_*\n**/hr_records_*\n",
    "local_templates": [
      {
        "path": "team/reviews",
        "content": "# Performance review exclusions\n**/review_notes_*\n**/pip_*\n**/termination_*\n"
      }
    ]
  },
  "recipes": [
    {
      "name": "daily-standup-summary",
      "description": "Automated daily standup summary from team updates",
      "path": "recipes/manager/daily-standup-summary.yaml",
      "schedule": "0 9 * * 1-5",
      "enabled": true
    },
    {
      "name": "weekly-team-metrics",
      "description": "Weekly team velocity and capacity analysis",
      "path": "recipes/manager/weekly-team-metrics.yaml",
      "schedule": "0 10 * * 1",
      "enabled": true
    },
    {
      "name": "monthly-1on1-prep",
      "description": "Monthly 1-on-1 prep with team member highlights",
      "path": "recipes/manager/monthly-1on1-prep.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    }
  ],
  "automated_tasks": [
    {
      "name": "morning-team-digest",
      "recipe": "recipes/manager/morning-digest.yaml",
      "schedule": "0 8 * * 1-5",
      "enabled": true,
      "notify_on_failure": true
    }
  ],
  "policies": [
    {
      "allow_tool": "github__*",
      "reason": "Manager needs full GitHub access for team coordination"
    },
    {
      "allow_tool": "agent_mesh__*",
      "reason": "Manager routes tasks and approvals"
    },
    {
      "allow_tool": "memory__*",
      "reason": "Manager needs context for decision-making"
    },
    {
      "allow_tool": "finance__approve_spend",
      "conditions": [
        {
          "amount_usd": "<50000"
        }
      ],
      "reason": "Manager can approve <$50K (Finance for >$50K)"
    },
    {
      "deny_tool": "privacy-guard__disable",
      "reason": "Cannot bypass privacy protection"
    },
    {
      "deny_tool": "developer__shell",
      "reason": "No arbitrary code execution for Manager role"
    }
  ],
  "privacy": {
    "mode": "hybrid",
    "strictness": "moderate",
    "allow_override": true,
    "rules": [
      {
        "pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b",
        "replacement": "[SSN]",
        "category": "SSN"
      },
      {
        "pattern": "\\b[A-Z]{2}\\d{6,8}\\b",
        "replacement": "[EMP_ID]",
        "category": "EMPLOYEE_ID"
      }
    ],
    "pii_categories": [
      "SSN",
      "EMAIL",
      "PHONE",
      "EMPLOYEE_ID"
    ]
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "90",
    "PRIVACY_GUARD_MODE": "hybrid",
    "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet",
    "BUDGET_APPROVAL_LIMIT": "50000"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();


-- Marketing Team Agent
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'marketing',
  'Marketing Team Agent',
  '{
  "role": "marketing",
  "display_name": "Marketing Team Agent",
  "description": "Campaign management, content creation, competitor analysis, and marketing analytics",
  "providers": {
    "primary": {
      "provider": "openrouter",
      "model": "openai/gpt-4o",
      "temperature": 0.7
    },
    "planner": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.5
    },
    "worker": {
      "provider": "openrouter",
      "model": "openai/gpt-4o-mini",
      "temperature": 0.8
    },
    "allowed_providers": [
      "openrouter"
    ],
    "forbidden_providers": []
  },
  "extensions": [
    {
      "name": "github",
      "enabled": true,
      "tools": [
        "list_issues",
        "create_issue",
        "add_comment"
      ]
    },
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "request_approval",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 60,
        "auto_summarize": true,
        "include_pii": false
      }
    },
    {
      "name": "web-scraper",
      "enabled": true
    }
  ],
  "goosehints": {
    "global": "# Marketing Role Context\nYou are the Marketing team agent.\nYour primary responsibilities are:\n- Campaign management and performance tracking\n- Content creation and editorial calendar\n- Competitor analysis and market intelligence\n- Marketing analytics and reporting\n\nWhen creating content:\n- Align with brand voice and guidelines\n- Focus on value proposition and benefits\n- Use data-driven insights\n- Optimize for target audience\n\nMarketing Sources:\n@marketing/brand-guidelines.md\n@marketing/content-calendar.xlsx\n@marketing/competitor-analysis.md\n\nKey Metrics to Track:\n- Campaign ROI and conversion rates\n- Website traffic and engagement\n- Lead generation and quality\n- Brand awareness and sentiment\n\nContent Guidelines:\n- SEO-optimized headlines and copy\n- Clear call-to-actions\n- Mobile-first design\n- A/B test messaging variants\n"
  },
  "gooseignore": {
    "global": "**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n\n# Marketing-specific exclusions\n**/customer_lists_*\n**/email_campaigns_private.*\n**/unreleased_campaigns.*\n**/budget_details.*\n"
  },
  "recipes": [
    {
      "name": "weekly-campaign-report",
      "description": "Weekly campaign performance summary and optimization recommendations",
      "path": "recipes/marketing/weekly-campaign-report.yaml",
      "schedule": "0 10 * * 1",
      "enabled": true
    },
    {
      "name": "monthly-content-calendar",
      "description": "Monthly content calendar generation with SEO topics",
      "path": "recipes/marketing/monthly-content-calendar.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    },
    {
      "name": "competitor-analysis",
      "description": "Monthly competitor analysis and market intelligence",
      "path": "recipes/marketing/competitor-analysis.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    }
  ],
  "policies": [
    {
      "allow_tool": "web-scraper__*",
      "reason": "Marketing needs web scraping for competitive intelligence"
    },
    {
      "allow_tool": "github__*",
      "conditions": [
        {
          "repo": "marketing/*"
        }
      ],
      "reason": "Marketing manages content repos"
    },
    {
      "allow_tool": "agent_mesh__*",
      "reason": "Marketing collaborates across teams"
    },
    {
      "deny_tool": "developer__shell",
      "reason": "No code execution for Marketing role"
    }
  ],
  "privacy": {
    "mode": "rules",
    "strictness": "permissive",
    "allow_override": true,
    "rules": [
      {
        "pattern": "\\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}\\b",
        "replacement": "[EMAIL]",
        "category": "EMAIL"
      }
    ],
    "pii_categories": [
      "EMAIL",
      "PHONE"
    ]
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "60",
    "PRIVACY_GUARD_MODE": "rules",
    "DEFAULT_MODEL": "openrouter/openai/gpt-4o"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();


-- Support Team Agent
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  'support',
  'Support Team Agent',
  '{
  "role": "support",
  "display_name": "Support Team Agent",
  "description": "Customer support, ticket triage, knowledge base management, and satisfaction tracking",
  "providers": {
    "primary": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.5
    },
    "planner": {
      "provider": "openrouter",
      "model": "anthropic/claude-3.5-sonnet",
      "temperature": 0.4
    },
    "worker": {
      "provider": "openrouter",
      "model": "openai/gpt-4o-mini",
      "temperature": 0.6
    },
    "allowed_providers": [
      "openrouter"
    ],
    "forbidden_providers": []
  },
  "extensions": [
    {
      "name": "github",
      "enabled": true,
      "tools": [
        "list_issues",
        "create_issue",
        "add_comment",
        "update_issue",
        "assign_issue"
      ]
    },
    {
      "name": "agent_mesh",
      "enabled": true,
      "tools": [
        "send_task",
        "request_approval",
        "notify",
        "fetch_status"
      ]
    },
    {
      "name": "memory",
      "enabled": true,
      "preferences": {
        "retention_days": 30,
        "auto_summarize": true,
        "include_pii": false
      }
    }
  ],
  "goosehints": {
    "global": "# Support Role Context\nYou are the Support team agent.\nYour primary responsibilities are:\n- Customer issue triage and resolution\n- Knowledge base article creation and updates\n- Support metrics tracking and reporting\n- Customer satisfaction improvement\n\nWhen handling support tickets:\n- Empathize with customer frustration\n- Provide clear, actionable solutions\n- Escalate complex issues promptly\n- Document resolutions in knowledge base\n- Follow up to ensure satisfaction\n\nSupport Sources:\n@support/kb-articles/\n@support/troubleshooting-guides/\n@support/sla-guidelines.md\n\nSLA Targets:\n- P0 (Critical): Response <1hr, Resolution <4hr\n- P1 (High): Response <4hr, Resolution <24hr\n- P2 (Medium): Response <24hr, Resolution <3 days\n- P3 (Low): Response <48hr, Resolution <7 days\n\nEscalation Criteria:\n- Security issues \u2192 Escalate immediately\n- Data loss/corruption \u2192 P0 escalation\n- Recurring issues (>3 tickets) \u2192 Root cause analysis\n- Customer VIP status \u2192 Priority handling\n"
  },
  "gooseignore": {
    "global": "**/.env\n**/.env.*\n**/secrets.*\n**/credentials.*\n\n# Customer data protection\n**/customer_data_*\n**/support_tickets_private.*\n**/customer_emails_*\n**/payment_info_*\n**/personal_identifiable_*\n"
  },
  "recipes": [
    {
      "name": "daily-ticket-summary",
      "description": "Daily support ticket summary and triage recommendations",
      "path": "recipes/support/daily-ticket-summary.yaml",
      "schedule": "0 9 * * 1-5",
      "enabled": true
    },
    {
      "name": "weekly-kb-updates",
      "description": "Weekly knowledge base article suggestions from recurring tickets",
      "path": "recipes/support/weekly-kb-updates.yaml",
      "schedule": "0 10 * * 5",
      "enabled": true
    },
    {
      "name": "monthly-satisfaction-report",
      "description": "Monthly customer satisfaction analysis and improvement recommendations",
      "path": "recipes/support/monthly-satisfaction-report.yaml",
      "schedule": "0 9 1 * *",
      "enabled": true
    }
  ],
  "policies": [
    {
      "allow_tool": "github__*",
      "conditions": [
        {
          "repo": "support/*"
        }
      ],
      "reason": "Support manages ticket tracking and KB"
    },
    {
      "allow_tool": "agent_mesh__*",
      "reason": "Support escalates to other teams"
    },
    {
      "deny_tool": "developer__shell",
      "reason": "No code execution for Support role"
    },
    {
      "deny_tool": "sql-mcp__*",
      "reason": "Support should not access databases directly"
    }
  ],
  "privacy": {
    "mode": "hybrid",
    "strictness": "strict",
    "allow_override": false,
    "rules": [
      {
        "pattern": "\\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}\\b",
        "replacement": "[CUSTOMER_EMAIL]",
        "category": "EMAIL"
      },
      {
        "pattern": "\\b\\d{3}-\\d{3}-\\d{4}\\b",
        "replacement": "[CUSTOMER_PHONE]",
        "category": "PHONE"
      },
      {
        "pattern": "\\b\\d{4}[- ]?\\d{4}[- ]?\\d{4}[- ]?\\d{4}\\b",
        "replacement": "[CREDIT_CARD]",
        "category": "CREDIT_CARD"
      }
    ],
    "pii_categories": [
      "EMAIL",
      "PHONE",
      "CREDIT_CARD",
      "ADDRESS"
    ]
  },
  "env_vars": {
    "SESSION_RETENTION_DAYS": "30",
    "PRIVACY_GUARD_MODE": "hybrid",
    "DEFAULT_MODEL": "openrouter/anthropic/claude-3.5-sonnet",
    "SLA_P0_RESPONSE_HOURS": "1",
    "SLA_P1_RESPONSE_HOURS": "4"
  },
  "signature": {
    "algorithm": "sha2-256",
    "vault_key": "transit/keys/profile-signing",
    "signed_at": null,
    "signed_by": null,
    "value": null
  }
}'::jsonb,
  NULL
)
ON CONFLICT (role) DO UPDATE
SET 
  data = EXCLUDED.data,
  updated_at = NOW();

