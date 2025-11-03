#!/usr/bin/env python3
"""
Test script to validate rules.yaml patterns against example data.
This ensures all regex patterns compile and match their documented examples.
"""

import yaml
import re
import sys

def test_rules():
    """Test all patterns in rules.yaml against their examples."""
    
    with open('rules.yaml', 'r') as f:
        data = yaml.safe_load(f)
    
    total_tests = 0
    passed_tests = 0
    failed_tests = []
    
    print("=" * 70)
    print("Testing Privacy Guard Detection Rules")
    print("=" * 70)
    
    for entity_type, config in data['entity_types'].items():
        print(f"\n📋 Testing {entity_type} ({config['display_name']})")
        print(f"   Category: {config['category']}")
        
        for i, pattern in enumerate(config['patterns'], 1):
            regex = pattern['regex']
            confidence = pattern['confidence']
            examples = pattern.get('examples', [])
            
            print(f"\n   Pattern {i}/{len(config['patterns'])}: {pattern['description']}")
            print(f"   Confidence: {confidence}")
            print(f"   Regex: {regex}")
            
            # Compile regex
            try:
                compiled = re.compile(regex)
            except re.error as e:
                print(f"   ❌ REGEX COMPILATION FAILED: {e}")
                failed_tests.append(f"{entity_type} - Pattern {i}: Regex compilation failed")
                continue
            
            # Test against examples
            if examples:
                print(f"   Testing {len(examples)} example(s):")
                for example in examples:
                    total_tests += 1
                    match = compiled.search(example)
                    if match:
                        print(f"      ✓ '{example}' -> MATCHED")
                        passed_tests += 1
                    else:
                        print(f"      ✗ '{example}' -> NO MATCH")
                        failed_tests.append(f"{entity_type} - Pattern {i}: Example '{example}' didn't match")
            else:
                print(f"   ⚠️  No examples provided")
    
    # Summary
    print("\n" + "=" * 70)
    print("Test Summary")
    print("=" * 70)
    print(f"Total tests: {total_tests}")
    print(f"Passed: {passed_tests} ({100*passed_tests//total_tests if total_tests > 0 else 0}%)")
    print(f"Failed: {len(failed_tests)}")
    
    if failed_tests:
        print("\n❌ Failed Tests:")
        for failure in failed_tests:
            print(f"   - {failure}")
        return 1
    else:
        print("\n✅ All tests passed!")
        return 0

if __name__ == '__main__':
    sys.exit(test_rules())
