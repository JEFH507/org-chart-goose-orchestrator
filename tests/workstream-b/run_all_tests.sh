#!/bin/bash
# Workstream B Test Suite Runner
# Executes all structural validation tests for Phase 5 Workstream B deliverables

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "Phase 5 Workstream B Test Suite"
echo "=========================================="
echo "Structural Validation Tests"
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

total_suites=5
passed_suites=0
failed_suites=()

# Test 1: Profile Schemas
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test Suite 1/5: Profile YAML Schemas"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/test_profile_schemas.sh"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites+=("Profile Schemas")
fi
echo ""

# Test 2: Recipe Schemas
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test Suite 2/5: Recipe YAML Schemas"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/test_recipe_schemas.sh"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites+=("Recipe Schemas")
fi
echo ""

# Test 3: Goosehints Syntax
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test Suite 3/5: Goosehints Markdown Syntax"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/test_goosehints_syntax.sh"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites+=("Goosehints Syntax")
fi
echo ""

# Test 4: Gooseignore Patterns
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test Suite 4/5: Gooseignore Pattern Validation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/test_gooseignore_patterns.sh"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites+=("Gooseignore Patterns")
fi
echo ""

# Test 5: SQL Seed
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test Suite 5/5: SQL Seed Script"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/test_sql_seed.sh"; then
    passed_suites=$((passed_suites + 1))
else
    failed_suites+=("SQL Seed")
fi
echo ""

# Final Summary
echo "=========================================="
echo "Workstream B Test Suite - Final Summary"
echo "=========================================="
echo "Test Suites Run: $total_suites"
echo "Passed: $passed_suites"
echo "Failed: $((total_suites - passed_suites))"
echo ""

if [ ${#failed_suites[@]} -gt 0 ]; then
    echo "❌ Failed Suites:"
    for suite in "${failed_suites[@]}"; do
        echo "   - $suite"
    done
    echo ""
    exit 1
else
    echo "✅ All test suites passed!"
    echo ""
    echo "Deliverables validated:"
    echo "  - 6 role profiles (YAML schemas)"
    echo "  - 18 recipes (cron schedules, tool refs)"
    echo "  - 8 goosehints templates (Markdown syntax)"
    echo "  - 8 gooseignore templates (glob patterns)"
    echo "  - 1 SQL seed script (Postgres JSONB)"
    echo ""
    exit 0
fi
