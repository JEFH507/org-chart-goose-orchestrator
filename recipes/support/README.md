# Support Role Recipes

Automated workflows for the Support team agent role.

## Recipe Schedule Summary

| Recipe | Schedule | Description |
|--------|----------|-------------|
| `daily-ticket-summary.yaml` | 9am Mon-Fri | Triage open tickets, identify SLA violations, create daily summary |
| `weekly-kb-updates.yaml` | Friday 10am | Analyze ticket patterns, suggest KB updates, track coverage gaps |
| `monthly-satisfaction-report.yaml` | 1st of month 9am | Customer satisfaction analysis, SLA performance, improvement areas |

## Daily Ticket Summary
**Schedule:** `0 9 * * 1-5` (9am Mon-Fri)

Workflow:
1. Fetch all open support tickets from GitHub
2. Triage by priority (P0-P3)
3. Check for SLA violations (response and resolution)
4. Identify at-risk tickets (>75% of SLA)
5. Generate daily summary for support team
6. Post to GitHub support repo

## Weekly KB Updates
**Schedule:** `0 10 * * 5` (Friday 10am)

Workflow:
1. Fetch tickets closed last week
2. Analyze recurring patterns and common questions
3. Generate KB article suggestions (new and updates)
4. Identify coverage gaps
5. Create weekly KB update report
6. Post to GitHub knowledge base repo

## Monthly Satisfaction Report
**Schedule:** `0 9 1 * *` (1st of month 9am)

Workflow:
1. Fetch tickets closed last month
2. Analyze customer satisfaction scores (CSAT/NPS)
3. Track resolution times vs SLA targets
4. Identify improvement areas
5. Generate comprehensive satisfaction report
6. Post to GitHub and notify manager

## Global Configuration

### Hints
- **Location:** `goosehints/templates/support-global.md`
- **Content:** Customer service guidelines, SLA targets, escalation criteria, KB best practices, triage workflow

### Ignore Patterns
- **Location:** `gooseignore/templates/support-global.txt`
- **Content:** Customer data, support tickets (private), payment info, PII, customer emails, security incidents

## SLA Targets Reference

### Response Times
- P0 (Critical): 1 hour
- P1 (High): 4 hours
- P2 (Medium): 24 hours
- P3 (Low): 72 hours

### Resolution Times
- P0 (Critical): 4 hours
- P1 (High): 24 hours
- P2 (Medium): 5 business days
- P3 (Low): 10 business days

## Usage

These recipes are automatically scheduled and executed by the Goose platform. Manual execution:

```bash
goose recipe run recipes/support/daily-ticket-summary.yaml
goose recipe run recipes/support/weekly-kb-updates.yaml
goose recipe run recipes/support/monthly-satisfaction-report.yaml
```

## Dependencies

All recipes require:
- GitHub MCP access (ticket management)
- Agent mesh (cross-role notifications)
- Support team repo access

## Notes

- All recipes include error handling and manager notifications
- Outputs are not included in audit logs (customer privacy)
- Reports are posted to GitHub for team visibility
- Retention: Daily (90 days), Weekly (90 days), Monthly (365 days)
