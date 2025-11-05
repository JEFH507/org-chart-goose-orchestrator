# Finance - Budget Analysis Local Hints
# Use this template for finance/budgets/ directory

## Budget Context

### Current Fiscal Year
- Fiscal Year: FY2026
- Start Date: January 1, 2026
- End Date: December 31, 2026
- Budget Cycle: Monthly close on 5th business day

### Budget Structure
Departments:
- Engineering (40% of budget)
- Sales & Marketing (30% of budget)
- Operations (20% of budget)
- Finance & Admin (10% of budget)

Categories:
- Headcount (60% of total budget)
- Infrastructure (20% of total budget)
- Marketing (10% of total budget)
- Other (10% of total budget)

## Budget Files

### Primary Spreadsheets
- `fy2026-budget.xlsx`: Annual budget plan by department/category
- `fy2026-actuals.xlsx`: Monthly actuals tracking
- `fy2026-forecast.xlsx`: Rolling forecast (updated quarterly)

### Data Layout
Budget spreadsheet structure:
- Sheet: "Budget Plan" - Annual budget by month
- Sheet: "YTD Summary" - Year-to-date rollup
- Sheet: "Variance Analysis" - Budget vs actual

Actuals spreadsheet structure:
- Sheet: "Monthly Actuals" - Actual spend by month
- Sheet: "Weekly Spend" - Weekly spend tracking
- Sheet: "Department Detail" - Drill-down by department

## Analysis Guidelines

### Variance Analysis Process
1. Fetch budget from `fy2026-budget.xlsx`
2. Fetch actuals from `fy2026-actuals.xlsx`
3. Calculate variance: `(Actual - Budget) / Budget * 100`
4. Flag variances >5% for review
5. Provide explanations for material variances

### Key Calculations
```
Budget Utilization % = YTD Actuals / YTD Budget * 100
Burn Rate = YTD Actuals / Days Elapsed * 365
Projected Year-End = Burn Rate * 365
Variance to Budget = (Projected - Annual Budget) / Annual Budget * 100
```

### Threshold Rules
- Green: Variance ±5% or less
- Yellow: Variance >5% and ≤10%
- Orange: Variance >10% and ≤15%
- Red: Variance >15% (requires immediate escalation)

## Approval Workflows

### Budget Request Process
1. Department head submits request via GitHub issue
2. Finance reviews budget availability
3. Finance approves/denies with rationale
4. If approved, update forecast spreadsheet
5. Notify department head of decision

### Reforecast Process (Quarterly)
1. Review YTD actuals vs budget
2. Analyze burn rate trends
3. Adjust forecast for remaining quarters
4. Get CFO approval
5. Communicate changes to departments

## Data Quality Checks

### Before Analysis
- [ ] Verify actuals data is complete (no missing months)
- [ ] Check for duplicate entries
- [ ] Validate department codes match budget
- [ ] Ensure category mappings are consistent

### After Analysis
- [ ] Cross-check totals with GL system
- [ ] Verify variance calculations (spot-check 3-5 departments)
- [ ] Confirm unusual variances with department heads
- [ ] Document assumptions in report

## Common Issues & Solutions

### Missing Actuals Data
- Check source system for data export
- Contact Finance Operations for manual extract
- Use prior month trend for estimate (note assumption)

### Large Variances
- One-time expenses: Document in variance report
- Timing differences: Adjust for accruals
- Budget errors: Propose reforecast

### Department Coding Errors
- Create mapping table for common errors
- Update source system coding
- Document corrections in audit trail

## Templates

### Variance Report Template
```
Department: [Name]
Period: [Month Year]
Budget: $[Amount]
Actual: $[Amount]
Variance: $[Amount] ([%])

Explanation:
[Detailed explanation of variance]

Recommended Action:
[Action item if variance material]
```

### Budget Request Template
```
Requestor: [Department Head]
Amount: $[Amount]
Category: [Category]
Justification: [Business case]
Budget Impact: [YTD utilization after approval]

Finance Review:
- [ ] Budget available
- [ ] Business case valid
- [ ] Approval authority confirmed

Decision: [Approved/Denied]
Rationale: [Explanation]
```

## Collaboration Notes

### GitHub Issues
- Use repo: `finance/budget-requests`
- Labels: `budget-request`, `variance`, `forecast`
- Assign: @finance-team for review

### Notification Protocol
- Budget approved >$25K: Notify CFO
- Variance >10%: Notify department head
- Forecast change >5%: Notify executives

## Compliance Notes

### Audit Requirements
- All budget decisions documented
- Variance explanations required for >5%
- Monthly close checklist completed
- Quarterly SOX controls tested

### Data Retention
- Budget spreadsheets: 7 years
- Actuals data: 7 years
- Variance reports: 7 years
- Approval emails: 7 years
