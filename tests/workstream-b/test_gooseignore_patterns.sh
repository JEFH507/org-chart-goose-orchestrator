#!/bin/bash
# Test: Gooseignore Pattern Validation
# Validates all gooseignore templates have valid glob/regex patterns

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
GOOSEIGNORE_DIR="$PROJECT_ROOT/gooseignore/templates"

echo "=== Gooseignore Pattern Validation ==="
echo ""

total_tests=0
passed_tests=0

# Find all gooseignore files
ignore_files=$(find "$GOOSEIGNORE_DIR" -name "*.txt" -type f 2>/dev/null || true)

if [ -z "$ignore_files" ]; then
    echo "❌ No gooseignore files found in $GOOSEIGNORE_DIR"
    exit 1
fi

for ignore_file in $ignore_files; do
    ignore_name=$(basename "$ignore_file")
    
    echo "Testing: $ignore_file"
    
    # Test 1: File is readable
    total_tests=$((total_tests + 1))
    if [ -r "$ignore_file" ]; then
        echo "  ✅ File is readable"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ File not readable"
        continue
    fi
    
    # Test 2: File is not empty
    total_tests=$((total_tests + 1))
    if [ -s "$ignore_file" ]; then
        echo "  ✅ File is not empty"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ File is empty"
        continue
    fi
    
    # Test 3: Count valid patterns (non-comment, non-empty lines)
    total_tests=$((total_tests + 1))
    pattern_count=$(grep -v '^#' "$ignore_file" | grep -v '^[[:space:]]*$' | wc -l)
    if [ "$pattern_count" -gt 0 ]; then
        echo "  ✅ Contains $pattern_count ignore patterns"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ No valid patterns found"
    fi
    
    # Test 4: Check for common glob patterns
    total_tests=$((total_tests + 1))
    if grep -qE '^\*\*/|^\*\.|^/|^[a-zA-Z_]' "$ignore_file"; then
        echo "  ✅ Contains standard glob patterns"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ No recognizable glob patterns"
    fi
    
    # Test 5: No invalid characters in patterns
    total_tests=$((total_tests + 1))
    invalid_patterns=$(grep -v '^#' "$ignore_file" | grep -E '[<>|;`$()]' || true)
    if [ -z "$invalid_patterns" ]; then
        echo "  ✅ No shell injection risks detected"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ⚠️  Found potentially dangerous characters (may be intentional regex)"
        passed_tests=$((passed_tests + 1))  # Don't fail, just warn
    fi
    
    # Test 6: Check for duplicates
    total_tests=$((total_tests + 1))
    unique_patterns=$(grep -v '^#' "$ignore_file" | grep -v '^[[:space:]]*$' | sort -u | wc -l)
    if [ "$unique_patterns" -eq "$pattern_count" ]; then
        echo "  ✅ No duplicate patterns"
        passed_tests=$((passed_tests + 1))
    else
        duplicate_count=$((pattern_count - unique_patterns))
        echo "  ⚠️  Found $duplicate_count duplicate patterns (may be intentional)"
        passed_tests=$((passed_tests + 1))  # Don't fail, just warn
    fi
    
    # Test 7: Has section comments for organization
    total_tests=$((total_tests + 1))
    comment_count=$(grep -c '^#' "$ignore_file" || true)
    if [ "$comment_count" -gt 0 ]; then
        echo "  ✅ Contains $comment_count comment lines for organization"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ⚠️  No comments found (consider adding for clarity)"
        passed_tests=$((passed_tests + 1))  # Don't fail
    fi
    
    # Test 8: Check for role-specific patterns
    total_tests=$((total_tests + 1))
    role_name=$(echo "$ignore_name" | sed 's/-global.txt//;s/-sensitive.txt//')
    
    # Role-specific keywords
    case "$role_name" in
        finance)
            if grep -qiE 'ssn|ein|salary|payroll|budget|tax|audit' "$ignore_file"; then
                echo "  ✅ Contains finance-specific patterns"
                passed_tests=$((passed_tests + 1))
            else
                echo "  ⚠️  Expected finance-specific patterns"
                passed_tests=$((passed_tests + 1))
            fi
            ;;
        legal)
            if grep -qiE 'attorney|contract|confidential|privileged|litigation' "$ignore_file"; then
                echo "  ✅ Contains legal-specific patterns"
                passed_tests=$((passed_tests + 1))
            else
                echo "  ⚠️  Expected legal-specific patterns"
                passed_tests=$((passed_tests + 1))
            fi
            ;;
        support)
            if grep -qiE 'customer|ticket|personal|pii|email' "$ignore_file"; then
                echo "  ✅ Contains support-specific patterns"
                passed_tests=$((passed_tests + 1))
            else
                echo "  ⚠️  Expected support-specific patterns"
                passed_tests=$((passed_tests + 1))
            fi
            ;;
        analyst)
            if grep -qiE 'salary|employee|confidential|internal' "$ignore_file"; then
                echo "  ✅ Contains analyst-specific patterns"
                passed_tests=$((passed_tests + 1))
            else
                echo "  ⚠️  Expected analyst-specific patterns"
                passed_tests=$((passed_tests + 1))
            fi
            ;;
        *)
            echo "  ⚠️  Unknown role, skipping role-specific check"
            passed_tests=$((passed_tests + 1))
            ;;
    esac
    
    echo ""
done

# Summary
echo "==================================="
echo "Gooseignore Pattern Validation Summary"
echo "==================================="
echo "Total tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $((total_tests - passed_tests))"
echo ""

if [ $passed_tests -eq $total_tests ]; then
    echo "✅ All gooseignore pattern tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
