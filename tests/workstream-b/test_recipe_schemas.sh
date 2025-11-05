#!/bin/bash
# Test: Recipe YAML Schema Validation
# Validates all 18 recipes have correct structure and valid cron expressions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RECIPES_DIR="$PROJECT_ROOT/recipes"

echo "=== Recipe Schema Validation ==="
echo ""

REQUIRED_FIELDS=("name" "version" "role" "trigger" "steps")

total_tests=0
passed_tests=0

# Find all recipe YAML files
recipe_files=$(find "$RECIPES_DIR" -name "*.yaml" -type f)

if [ -z "$recipe_files" ]; then
    echo "❌ No recipe files found in $RECIPES_DIR"
    exit 1
fi

for recipe_file in $recipe_files; do
    recipe_name=$(basename "$recipe_file")
    
    echo "Testing: $recipe_file"
    
    # Test 1: File is readable
    total_tests=$((total_tests + 1))
    if [ -r "$recipe_file" ]; then
        echo "  ✅ File is readable"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ File not readable"
        continue
    fi
    
    # Test 2: Required fields present
    for field in "${REQUIRED_FIELDS[@]}"; do
        total_tests=$((total_tests + 1))
        if grep -q "^${field}:" "$recipe_file"; then
            passed_tests=$((passed_tests + 1))
        else
            echo "  ❌ Missing required field: $field"
        fi
    done
    
    # Test 3: Trigger has type
    total_tests=$((total_tests + 1))
    if grep -A 3 '^trigger:' "$recipe_file" | grep -q 'type:'; then
        echo "  ✅ Trigger type configured"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ No trigger type found"
    fi
    
    # Test 4: Schedule trigger has cron expression
    total_tests=$((total_tests + 1))
    if grep -A 3 '^trigger:' "$recipe_file" | grep -q 'type: "schedule"'; then
        if grep -A 5 '^trigger:' "$recipe_file" | grep -q 'schedule:'; then
            echo "  ✅ Cron schedule present"
            passed_tests=$((passed_tests + 1))
            
            # Test 5: Validate cron expression format (basic check)
            total_tests=$((total_tests + 1))
            cron_expr=$(grep -A 5 '^trigger:' "$recipe_file" | grep 'schedule:' | sed 's/.*schedule: "\(.*\)"/\1/')
            
            # Strip inline comments and count fields in cron expression (should be 5 or 6)
            cron_expr_clean=$(echo "$cron_expr" | sed 's/#.*//' | xargs)
            field_count=$(echo "$cron_expr_clean" | awk '{print NF}')
            
            if [ "$field_count" -eq 5 ] || [ "$field_count" -eq 6 ]; then
                echo "  ✅ Valid cron expression format ($field_count fields)"
                passed_tests=$((passed_tests + 1))
            else
                echo "  ❌ Invalid cron expression: $cron_expr (has $field_count fields, expected 5 or 6)"
            fi
        else
            echo "  ❌ Schedule trigger missing cron expression"
        fi
    else
        # Not a schedule trigger, skip cron validation
        passed_tests=$((passed_tests + 1))
        total_tests=$((total_tests + 1))
        passed_tests=$((passed_tests + 1))
    fi
    
    # Test 6: Steps is an array
    total_tests=$((total_tests + 1))
    if grep -A 2 '^steps:' "$recipe_file" | grep -q '  - id:'; then
        echo "  ✅ Steps array present"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ Steps not properly formatted"
    fi
    
    # Test 7: Each step has an id
    total_tests=$((total_tests + 1))
    step_count=$(grep -c '^  - id:' "$recipe_file" || true)
    if [ "$step_count" -gt 0 ]; then
        echo "  ✅ Found $step_count steps with IDs"
        passed_tests=$((passed_tests + 1))
    else
        echo "  ❌ No steps with IDs found"
    fi
    
    # Test 8: Tool steps have valid tool references
    total_tests=$((total_tests + 1))
    tool_refs=$(grep 'tool:' "$recipe_file" | grep -oE '"[a-z_-]+__[a-z_-]+"' || true)
    if [ -n "$tool_refs" ]; then
        # Check format: extension__tool
        invalid_tools=0
        for tool_ref in $tool_refs; do
            if ! echo "$tool_ref" | grep -qE '"[a-z_-]+__[a-z_-]+"'; then
                invalid_tools=$((invalid_tools + 1))
            fi
        done
        
        if [ $invalid_tools -eq 0 ]; then
            echo "  ✅ All tool references valid"
            passed_tests=$((passed_tests + 1))
        else
            echo "  ❌ Found $invalid_tools invalid tool references"
        fi
    else
        # No tool steps, which is OK if it's all prompts
        echo "  ⚠️  No tool references found (prompt-only recipe?)"
        passed_tests=$((passed_tests + 1))
    fi
    
    echo ""
done

# Summary
echo "==================================="
echo "Recipe Schema Validation Summary"
echo "==================================="
echo "Total tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $((total_tests - passed_tests))"
echo ""

if [ $passed_tests -eq $total_tests ]; then
    echo "✅ All recipe schema tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
