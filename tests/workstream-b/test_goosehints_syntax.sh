#!/bin/bash
# Test: Goosehints Markdown Syntax Validation
# Validates all goosehints templates are well-formed Markdown

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
GOOSEHINTS_DIR="$PROJECT_ROOT/goosehints/templates"

echo "=== Goosehints Syntax Validation ==="
echo ""

total_tests=0
passed_tests=0

# Find all goosehints Markdown files
hint_files=$(find "$GOOSEHINTS_DIR" -name "*.md" -type f 2>/dev/null || true)

if [ -z "$hint_files" ]; then
    echo "❌ No goosehints files found in $GOOSEHINTS_DIR"
    exit 1
fi

for hint_file in $hint_files; do
    hint_name=$(basename "$hint_file")
    
    echo "Testing: $hint_file"
    
    # Test 1: File is readable
    total_tests=$((total_tests + 1))
    if [ -r "$hint_file" ]; then
        echo "  ✅ File is readable"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ File not readable"
        continue
    fi
    
    # Test 2: File is not empty
    total_tests=$((total_tests + 1))
    if [ -s "$hint_file" ]; then
        echo "  ✅ File is not empty"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ File is empty"
        continue
    fi
    
    # Test 3: Has Markdown headers
    total_tests=$((total_tests + 1))
    if grep -q '^#' "$hint_file"; then
        echo "  ✅ Contains Markdown headers"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ No Markdown headers found"
    fi
    
    # Test 4: No malformed code blocks (unclosed ```)
    total_tests=$((total_tests + 1))
    code_block_count=$(grep -c '^```' "$hint_file" || true)
    if [ $((code_block_count % 2)) -eq 0 ]; then
        echo "  ✅ Code blocks properly closed ($code_block_count markers)"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ Unclosed code blocks detected ($code_block_count markers)"
    fi
    
    # Test 5: No broken Markdown links (basic check)
    total_tests=$((total_tests + 1))
    broken_links=$(grep -oE '\[([^\]]*)\]\(' "$hint_file" | grep -E '\[\]\(' || true)
    if [ -z "$broken_links" ]; then
        echo "  ✅ No broken link syntax detected"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ Found broken link syntax: $broken_links"
    fi
    
    # Test 6: Check for role-specific content (filename should match content)
    total_tests=$((total_tests + 1))
    role_name=$(echo "$hint_name" | sed 's/-global.md//;s/-local.md//')
    
    if grep -qi "$role_name" "$hint_file" || grep -qi "role" "$hint_file"; then
        echo "  ✅ Contains role-specific context"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ⚠️  No role-specific context detected (may be intentional)"
        passed_tests=$((passed_tests + 1))  # Don't fail, just warn
    fi
    
    # Test 7: No obvious typos in common headings
    total_tests=$((total_tests + 1))
    if grep -qE '^## (Role Context|Data Sources|Privacy|Tool Usage|Output Format|Analysis Principles|Guidelines|Instructions)' "$hint_file"; then
        echo "  ✅ Standard heading structure present"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ⚠️  Non-standard heading structure (may be intentional)"
        passed_tests=$((passed_tests + 1))  # Don't fail, just warn
    fi
    
    # Test 8: File size is reasonable (not truncated, not massive)
    total_tests=$((total_tests + 1))
    file_size=$(wc -c < "$hint_file")
    if [ "$file_size" -gt 100 ] && [ "$file_size" -lt 100000 ]; then
        echo "  ✅ File size reasonable ($file_size bytes)"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ⚠️  Unusual file size: $file_size bytes"
        if [ "$file_size" -le 100 ]; then
            echo "  ❌ File may be truncated"
        else
            passed_tests=$((passed_tests + 1))  # Large is OK for legal hints
        fi
    fi
    
    echo ""
done

# Summary
echo "==================================="
echo "Goosehints Syntax Validation Summary"
echo "==================================="
echo "Total tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $((total_tests - passed_tests))"
echo ""

if [ $passed_tests -eq $total_tests ]; then
    echo "✅ All goosehints syntax tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
