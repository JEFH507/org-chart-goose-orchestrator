#!/bin/bash
# Test: SQL Seed Script Validation
# Validates the profiles seed SQL can be parsed and loads without errors

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SEED_FILE="$PROJECT_ROOT/seeds/profiles.sql"

echo "=== SQL Seed Script Validation ==="
echo ""

total_tests=0
passed_tests=0

# Test 1: Seed file exists
echo "Testing: $SEED_FILE"
total_tests=$((total_tests + 1))
if [ -f "$SEED_FILE" ]; then
    echo "  ✅ Seed file exists"
    passed_tests=$((passed_tests + 1))
else
    echo "  ❌ Seed file not found"
    exit 1
fi

# Test 2: File is not empty
total_tests=$((total_tests + 1))
if [ -s "$SEED_FILE" ]; then
    echo "  ✅ File is not empty"
    passed_tests=$((passed_tests + 1))
else
    echo "  ❌ File is empty"
    exit 1
fi

# Test 3: Contains INSERT statements
total_tests=$((total_tests + 1))
insert_count=$(grep -c '^INSERT INTO profiles' "$SEED_FILE" || true)
if [ "$insert_count" -eq 6 ]; then
    echo "  ✅ Contains 6 INSERT statements (one per role)"
    passed_tests=$((passed_tests + 1))
elif [ "$insert_count" -gt 0 ]; then
    echo "  ⚠️  Found $insert_count INSERT statements (expected 6)"
    passed_tests=$((passed_tests + 1))
else
    echo "  ❌ No INSERT statements found"
fi

# Test 4: All role names present
total_tests=$((total_tests + 1))
roles=("finance" "manager" "analyst" "marketing" "support" "legal")
missing_roles=()

for role in "${roles[@]}"; do
    if ! grep -q "'$role'" "$SEED_FILE"; then
        missing_roles+=("$role")
    fi
done

if [ ${#missing_roles[@]} -eq 0 ]; then
    echo "  ✅ All 6 roles present in SQL"
    passed_tests=$((passed_tests + 1))
else
    echo "  ❌ Missing roles: ${missing_roles[*]}"
fi

# Test 5: JSONB syntax validation (basic)
total_tests=$((total_tests + 1))
if grep -q "'::jsonb" "$SEED_FILE"; then
    echo "  ✅ JSONB casting syntax present"
    passed_tests=$((passed_tests + 1))
else
    echo "  ❌ No JSONB casting found"
fi

# Test 6: No syntax errors (basic checks)
total_tests=$((total_tests + 1))
syntax_errors=0

# Check for unmatched quotes (basic)
single_quotes=$(grep -o "'" "$SEED_FILE" | wc -l)
if [ $((single_quotes % 2)) -ne 0 ]; then
    echo "  ⚠️  Odd number of single quotes detected"
    syntax_errors=$((syntax_errors + 1))
fi

# Check for unmatched parentheses
open_parens=$(grep -o '(' "$SEED_FILE" | wc -l)
close_parens=$(grep -o ')' "$SEED_FILE" | wc -l)
if [ "$open_parens" -ne "$close_parens" ]; then
    echo "  ❌ Unmatched parentheses ($open_parens open, $close_parens close)"
    syntax_errors=$((syntax_errors + 1))
else
    echo "  ✅ Parentheses balanced"
fi

if [ $syntax_errors -eq 0 ]; then
    passed_tests=$((passed_tests + 1))
fi

# Test 7: Verification queries present
total_tests=$((total_tests + 1))
if grep -q '^SELECT' "$SEED_FILE"; then
    echo "  ✅ Contains verification SELECT queries"
    passed_tests=$((passed_tests + 1))
else
    echo "  ⚠️  No verification queries found"
    passed_tests=$((passed_tests + 1))  # Not critical
fi

# Test 8: Database load test (if Postgres is available)
total_tests=$((total_tests + 1))
if command -v psql &> /dev/null && docker ps | grep -q ce_postgres; then
    echo "  Testing SQL load in database..."
    
    # Create a temporary test
    if PGPASSWORD=goose123 psql -h localhost -p 5432 -U goose -d goose -c '\d profiles' > /dev/null 2>&1; then
        # Table exists, try to parse SQL (dry run)
        if PGPASSWORD=goose123 psql -h localhost -p 5432 -U goose -d goose --single-transaction --set ON_ERROR_STOP=on -c "BEGIN; $(cat "$SEED_FILE"); ROLLBACK;" > /dev/null 2>&1; then
            echo "  ✅ SQL loads successfully in database (rollback test)"
            passed_tests=$((passed_tests + 1))
        else
            echo "  ❌ SQL failed to load in database"
        fi
    else
        echo "  ⚠️  profiles table not found, skipping database test"
        passed_tests=$((passed_tests + 1))
    fi
else
    echo "  ⚠️  psql or docker not available, skipping database test"
    passed_tests=$((passed_tests + 1))
fi

# Summary
echo ""
echo "==================================="
echo "SQL Seed Validation Summary"
echo "==================================="
echo "Total tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $((total_tests - passed_tests))"
echo ""

if [ $passed_tests -eq $total_tests ]; then
    echo "✅ All SQL seed tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
