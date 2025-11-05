#!/bin/bash
# Test: Profile YAML Schema Validation
# Validates all 6 role profiles have correct structure and required fields

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PROFILES_DIR="$PROJECT_ROOT/profiles"

echo "=== Profile Schema Validation ==="
echo ""

# Check if yq is available (YAML processor)
if ! command -v yq &> /dev/null; then
    echo "⚠️  yq not found, attempting basic validation with grep/sed"
    USE_YQ=false
else
    USE_YQ=true
fi

PROFILES=("finance" "manager" "analyst" "marketing" "support" "legal")
REQUIRED_FIELDS=("role" "display_name" "providers" "extensions" "recipes" "privacy" "policies" "signature")

total_tests=0
passed_tests=0

for profile in "${PROFILES[@]}"; do
    profile_file="$PROFILES_DIR/${profile}.yaml"
    
    echo "Testing: $profile_file"
    
    # Test 1: File exists
    total_tests=$((total_tests + 1))
    if [ -f "$profile_file" ]; then
        echo "  ✅ File exists"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ File not found"
        continue
    fi
    
    # Test 2: Valid YAML syntax
    total_tests=$((total_tests + 1))
    if $USE_YQ; then
        if yq eval '.' "$profile_file" > /dev/null 2>&1; then
            echo "  ✅ Valid YAML syntax"
            passed_tests=$((passed_tests + 1))
        else
            echo "  ❌ Invalid YAML syntax"
            continue
        fi
    else
        # Basic check: file is readable and has key-value pairs
        if grep -q '^[a-z_]*:' "$profile_file"; then
            echo "  ✅ Basic YAML structure detected"
            passed_tests=$((passed_tests + 1))
        else
            echo "  ❌ No YAML structure found"
            continue
        fi
    fi
    
    # Test 3: Required fields present
    for field in "${REQUIRED_FIELDS[@]}"; do
        total_tests=$((total_tests + 1))
        if grep -q "^${field}:" "$profile_file"; then
            passed_tests=$((passed_tests + 1))
        else
            echo "  ❌ Missing required field: $field"
        fi
    done
    
    # Test 4: Role name matches filename
    total_tests=$((total_tests + 1))
    if grep -q "^role: \"${profile}\"" "$profile_file" || grep -q "^role: ${profile}" "$profile_file"; then
        echo "  ✅ Role name matches filename"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ Role name doesn't match filename"
    fi
    
    # Test 5: Providers section has primary
    total_tests=$((total_tests + 1))
    if grep -A 10 '^providers:' "$profile_file" | grep -q 'primary:'; then
        echo "  ✅ Primary provider configured"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ No primary provider found"
    fi
    
    # Test 6: Extensions is an array
    total_tests=$((total_tests + 1))
    if grep -A 2 '^extensions:' "$profile_file" | grep -q '  - name:'; then
        echo "  ✅ Extensions array present"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ Extensions not properly formatted"
    fi
    
    # Test 7: Privacy mode is valid
    total_tests=$((total_tests + 1))
    if grep -A 5 '^privacy:' "$profile_file" | grep -E 'mode: "(strict|hybrid|moderate|rules|permissive)"'; then
        echo "  ✅ Valid privacy mode"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ Invalid or missing privacy mode"
    fi
    
    # Test 8: Signature algorithm present
    total_tests=$((total_tests + 1))
    if grep -A 3 '^signature:' "$profile_file" | grep -q 'algorithm:'; then
        echo "  ✅ Signature algorithm configured"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ No signature algorithm found"
    fi
    
    echo ""
done

# Summary
echo "==================================="
echo "Profile Schema Validation Summary"
echo "==================================="
echo "Total tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $((total_tests - passed_tests))"
echo ""

if [ $passed_tests -eq $total_tests ]; then
    echo "✅ All profile schema tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
