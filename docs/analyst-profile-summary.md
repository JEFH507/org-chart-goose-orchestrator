# Business Analyst Profile - Phase 5 Workstream B
## Profile Creation Summary

**Created:** 2025-11-05  
**Status:** Complete ✅  
**Phase:** Phase 5 Workstream B

---

## Overview

The Business Analyst role profile has been successfully created for the org-chart goose orchestrator. This profile enables data analysis, process optimization, and operational efficiency workflows.

## Files Created

### 1. Profile Configuration
**File:** `profiles/analyst.yaml` (6.8 KB)

**Key Features:**
- **Role:** analyst
- **Display Name:** Business Analyst
- **Description:** Data analysis, process optimization, time studies, and KPI tracking
- **Providers:** 
  - Primary: OpenRouter GPT-4o (data analysis)
  - Planner: Claude 3.5 Sonnet (analytical planning)
  - Worker: OpenRouter GPT-4o (insights generation)
- **Extensions:** developer, excel-mcp, sql-mcp, agent_mesh, memory
- **Privacy:** Moderate mode (hybrid rules + NER)
- **Retention:** 90 days (quarterly trend analysis)

### 2. Automated Recipes

#### Recipe 1: Daily KPI Report
**File:** `recipes/analyst/daily-kpi-report.yaml` (5.4 KB)  
**Schedule:** `0 9 * * 1-5` (9am Mon-Fri)

**Workflow:**
1. Extract daily operational metrics from analytics database
2. Calculate week-over-week and month-over-month trends
3. Detect statistical anomalies using control chart methodology
4. Create executive KPI dashboard summary
5. Save report to analytics/reports/
6. Notify Manager with summary
7. Escalate critical anomalies if detected

**Key Metrics:**
- Operational health (green/yellow/red)
- Department KPIs with trends
- Anomaly detection (>3σ deviations)
- Statistical confidence intervals

#### Recipe 2: Process Bottleneck Analysis
**File:** `recipes/analyst/process-bottleneck-analysis.yaml` (7.3 KB)  
**Schedule:** `0 10 * * 1` (Monday 10am)

**Workflow:**
1. Extract process execution data for previous week
2. Calculate cycle time statistics (mean, p50, p95)
3. Identify bottlenecks using Theory of Constraints
4. Perform root cause analysis (5 Whys, Fishbone)
5. Generate optimization recommendations (short/medium/long-term)
6. Create comprehensive bottleneck analysis report
7. Save report and notify Manager

**Bottleneck Criteria:**
- Highest p95 duration (slowest step)
- Highest variance (unstable process)
- Utilization >80% (capacity constrained)
- Error rate >5% (quality issue)

#### Recipe 3: Time Study Analysis
**File:** `recipes/analyst/time-study-analysis.yaml` (11 KB)  
**Schedule:** `0 9 1 * *` (1st of month 9am)

**Workflow:**
1. Extract time study observations from previous month
2. Calculate standard times using performance rating methodology
3. Analyze theoretical vs actual capacity
4. Compare efficiency across experience levels and tools
5. Identify improvement opportunities
6. Create comprehensive monthly time study report
7. Save report and standard times reference (Excel)
8. Notify Manager with key findings

**Methodology:**
- Observed Time (OT) → Normal Time (NT) → Standard Time (ST)
- Performance rating normalization
- Standard allowances (Fatigue 5%, Delays 10%, Personal 5%)
- 95% confidence intervals
- Minimum sample size: n ≥ 30

### 3. Global Hints
**File:** `goosehints/templates/analyst-global.md` (11 KB)

**Contents:**
- Role context and responsibilities
- Analytical methodology and standards
- Statistical best practices (p50, p95, CI, sample sizes)
- Process analysis framework (Theory of Constraints, time studies)
- Data sources and key metrics
- Reporting standards (executive summary, detailed analysis)
- Process optimization workflow (Measure → Analyze → Improve → Control → Sustain)
- Root cause analysis methods (5 Whys, Fishbone, Pareto)
- Collaboration guidelines
- Tool usage best practices (SQL, Excel, Python/R)
- Privacy and data handling (anonymization, aggregation)
- Quality assurance checklist
- Common pitfalls to avoid

### 4. Global Ignore
**File:** `gooseignore/templates/analyst-global.txt` (4.3 KB)

**Privacy Protection:**
- Environment & secrets
- Database credentials
- Employee personal data (MUST ANONYMIZE)
- HR & performance data
- Compensation data
- Time study raw data (unanonymized)
- Process data with PII
- Financial data (employee-related)
- API keys & tokens
- Cloud provider credentials
- Business confidential data
- Customer PII
- Compliance & audit documents
- Backup files
- Logs (may contain sensitive data)

---

## Profile Capabilities

### Allowed Operations
✅ Read-only SQL queries (analytics_ro database)  
✅ Excel spreadsheet analysis (excel-mcp)  
✅ Data processing scripts (Python, R, awk, sed)  
✅ Text editor for reports and scripts  
✅ Agent mesh notifications  
✅ Memory for session context (no PII)

### Denied Operations
❌ Write operations to production databases  
❌ GitHub code commits/PRs  
❌ Privacy Guard override (compliance requirement)  
❌ Arbitrary shell commands (restricted to data processing)

---

## RBAC/ABAC Policies

### Allowed Tools
- `excel-mcp__*` - Spreadsheet operations
- `sql-mcp__query` - Read-only queries (analytics_ro only)
- `developer__shell` - Limited to: python, Rscript, awk, sed, grep, sort, uniq
- `developer__text_editor` - Analysis scripts and reports
- `agent_mesh__notify` - Stakeholder notifications

### Denied Tools
- `sql-mcp__execute` - No write operations
- `github__create_pr` - No code commits

---

## Privacy Configuration

**Mode:** Hybrid (rules + NER)  
**Strictness:** Moderate  
**Override:** Allowed (for anonymized data)

**PII Anonymization Rules:**
- Employee IDs: `[EMP_ID]`
- Email addresses: `[EMAIL]`
- Phone numbers: `[PHONE]`
- SSN: `[SSN]`

**Aggregation Requirements:**
- Minimum group size: 5 individuals
- Report aggregates only
- Suppress cells with n < 5

---

## Data Sources

### Primary Database
- **Database:** `analytics_ro` (read-only)
- **Tables:** 
  - `operational_metrics` - KPI tracking
  - `process_executions` - Process performance data
  - `time_studies` - Time and motion observations
- **Access:** SELECT only

### Reference Documents
- `@analytics/dashboards/kpi-definitions.md`
- `@analytics/data/operational-metrics.xlsx`
- `@analytics/sql/reporting-views.sql`
- `@analytics/methodology/statistical-standards.md`

---

## Reporting Standards

### Executive Summary (1-2 pages)
1. Key Findings (3-5 bullets)
2. Overall Assessment (health, trend, confidence)
3. Top Priorities (ranked by impact × urgency)

### Detailed Analysis
1. Objective
2. Methodology
3. Data Sources
4. Findings
5. Insights
6. Recommendations
7. Next Steps

### Statistical Requirements
- Always include sample size (n)
- Report 95% confidence intervals
- Use median (p50) for robustness
- Report both statistical and practical significance
- Minimum sample size: n ≥ 30

---

## Integration Points

### Notifications
- **Manager:** Daily KPI reports, weekly bottleneck analysis, monthly time studies
- **Finance:** (Future) Budget impact analysis
- **Operations:** Process optimization recommendations

### Escalations
Escalate to Manager when:
- Critical anomaly detected (>3σ deviation)
- Process degradation >15% from baseline
- Capacity utilization >90%
- SLA breach imminent
- Data quality issues preventing analysis

---

## Quality Assurance

### Before Publishing Reports
- [ ] Data sources documented
- [ ] Sample sizes reported (n)
- [ ] Methodology explained
- [ ] Assumptions stated clearly
- [ ] Statistical significance tested
- [ ] Practical significance assessed
- [ ] Anomalies flagged and explained
- [ ] Recommendations prioritized
- [ ] Next steps defined
- [ ] Charts labeled and titled
- [ ] PII anonymized
- [ ] Manager notified

---

## Environment Variables

```bash
SESSION_RETENTION_DAYS=90
PRIVACY_GUARD_MODE=hybrid
DEFAULT_MODEL=openrouter/openai/gpt-4o
ANALYTICS_DB=analytics_ro
REPORT_OUTPUT_DIR=analytics/reports
MIN_SAMPLE_SIZE=30
```

---

## Comparison with Other Roles

| Feature | Finance | Manager | Analyst |
|---------|---------|---------|---------|
| **Primary Provider** | Claude 3.5 | Claude 3.5 | GPT-4o |
| **Focus** | Budget/Compliance | Team Oversight | Data Analysis |
| **Privacy Mode** | Strict | Moderate | Moderate |
| **SQL Access** | Denied | Denied | Read-only |
| **Excel** | Yes | Limited | Yes |
| **Shell Access** | Denied | Denied | Limited |
| **Retention** | 90 days | 90 days | 90 days |

---

## Next Steps

### Phase 5 Workstream B Continuation
1. ✅ Analyst profile created
2. ⏳ Test analyst recipes in staging environment
3. ⏳ Create Developer role profile
4. ⏳ Create Operations role profile
5. ⏳ Integration testing across all roles

### Testing Checklist
- [ ] Test daily-kpi-report recipe with sample data
- [ ] Test process-bottleneck-analysis recipe
- [ ] Test time-study-analysis recipe
- [ ] Validate PII anonymization rules
- [ ] Verify SQL read-only access
- [ ] Test Manager notification flow
- [ ] Validate report generation and storage

### Documentation
- [ ] Add analyst role to Architecture Decision Records (ADRs)
- [ ] Update master technical project plan
- [ ] Document integration with existing roles
- [ ] Create analyst role onboarding guide

---

## References

**Templates Used:**
- `profiles/finance.yaml` - Financial role structure
- `profiles/manager.yaml` - Team oversight structure
- `recipes/finance/weekly-spend-report.yaml` - Recipe structure
- `recipes/manager/daily-standup-summary.yaml` - Workflow patterns

**Key Methodologies:**
- Theory of Constraints (bottleneck analysis)
- Time and Motion Study (standard times)
- Statistical Process Control (control charts)
- Root Cause Analysis (5 Whys, Fishbone, Pareto)

**Standards:**
- 95% confidence intervals
- Minimum sample size: n ≥ 30
- PII anonymization required
- Aggregation minimum: 5 individuals

---

**Profile Created By:** Goose Agent  
**Creation Date:** 2025-11-05  
**Version:** 1.0.0  
**Status:** Ready for Testing ✅
