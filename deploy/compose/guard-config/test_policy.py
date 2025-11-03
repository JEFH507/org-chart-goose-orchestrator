#!/usr/bin/env python3
"""
Policy YAML Validation Script
Tests that policy.yaml has all required sections and valid values
"""

import yaml
import sys
from pathlib import Path

def validate_policy():
    """Validate policy.yaml structure and content"""
    
    policy_path = Path(__file__).parent / "policy.yaml"
    
    print(f"📄 Loading policy from: {policy_path}")
    with open(policy_path) as f:
        policy = yaml.safe_load(f)
    
    errors = []
    warnings = []
    
    # Check version
    if policy.get("version") != "1.0":
        errors.append("Missing or invalid 'version' field")
    
    # Check required top-level sections
    required_sections = ["detection", "masking", "audit", "session"]
    for section in required_sections:
        if section not in policy:
            errors.append(f"Missing required section: {section}")
    
    # Validate detection section
    detection = policy.get("detection", {})
    valid_modes = ["OFF", "DETECT", "MASK", "STRICT"]
    if detection.get("mode") not in valid_modes:
        errors.append(f"Invalid detection.mode, must be one of {valid_modes}")
    
    valid_confidence = ["LOW", "MEDIUM", "HIGH"]
    if detection.get("confidence_threshold") not in valid_confidence:
        errors.append(f"Invalid confidence_threshold, must be one of {valid_confidence}")
    
    # Validate masking section
    masking = policy.get("masking", {})
    valid_strategies = ["PSEUDONYM", "REDACT", "FPE"]
    if masking.get("default_strategy") not in valid_strategies:
        errors.append(f"Invalid default_strategy, must be one of {valid_strategies}")
    
    # Check all 8 entity types are configured
    required_types = [
        "SSN", "PHONE", "EMAIL", "PERSON", "CREDIT_CARD", 
        "IP_ADDRESS", "DATE_OF_BIRTH", "ACCOUNT_NUMBER"
    ]
    per_type = masking.get("per_type", {})
    
    for entity_type in required_types:
        if entity_type not in per_type:
            warnings.append(f"Entity type '{entity_type}' not configured in masking.per_type")
        else:
            config = per_type[entity_type]
            if "strategy" not in config:
                errors.append(f"Missing 'strategy' for {entity_type}")
            elif config["strategy"] not in valid_strategies:
                errors.append(f"Invalid strategy for {entity_type}: {config['strategy']}")
    
    # Validate specific entity configurations
    # SSN should use FPE with preserve_last
    ssn = per_type.get("SSN", {})
    if ssn.get("strategy") == "FPE":
        if "fpe_preserve_last" not in ssn:
            warnings.append("SSN uses FPE but fpe_preserve_last not set")
        elif not isinstance(ssn["fpe_preserve_last"], int):
            errors.append("SSN fpe_preserve_last must be an integer")
    
    # PHONE should use FPE with preserve_area_code
    phone = per_type.get("PHONE", {})
    if phone.get("strategy") == "FPE":
        if "fpe_preserve_area_code" not in phone:
            warnings.append("PHONE uses FPE but fpe_preserve_area_code not set")
    
    # EMAIL should use PSEUDONYM with format
    email = per_type.get("EMAIL", {})
    if email.get("strategy") == "PSEUDONYM":
        if "pseudonym_format" not in email:
            warnings.append("EMAIL uses PSEUDONYM but pseudonym_format not set")
        elif "@redacted.local" not in email.get("pseudonym_format", ""):
            warnings.append("EMAIL pseudonym_format should include @redacted.local")
    
    # CREDIT_CARD should use REDACT
    cc = per_type.get("CREDIT_CARD", {})
    if cc.get("strategy") != "REDACT":
        warnings.append("CREDIT_CARD recommended strategy is REDACT (preserve last 4)")
    
    # Validate audit section
    audit = policy.get("audit", {})
    if not audit.get("log_redactions"):
        warnings.append("audit.log_redactions is false, no redaction events will be logged")
    
    valid_log_levels = ["error", "warn", "info", "debug", "trace"]
    if audit.get("audit_log_level") not in valid_log_levels:
        errors.append(f"Invalid audit_log_level, must be one of {valid_log_levels}")
    
    # Validate session section
    session = policy.get("session", {})
    if "mapping_ttl_seconds" not in session:
        warnings.append("session.mapping_ttl_seconds not set, mappings won't expire")
    
    # Validate fallback section (important for graceful degradation)
    fallback = policy.get("fallback", {})
    if "missing_salt_mode" not in fallback:
        warnings.append("fallback.missing_salt_mode not set, behavior undefined when PSEUDO_SALT missing")
    
    # Print results
    print("\n" + "=" * 60)
    print("POLICY VALIDATION RESULTS")
    print("=" * 60)
    
    if errors:
        print("\n❌ ERRORS:")
        for error in errors:
            print(f"  - {error}")
    else:
        print("\n✅ No errors found")
    
    if warnings:
        print(f"\n⚠️  WARNINGS ({len(warnings)}):")
        for warning in warnings:
            print(f"  - {warning}")
    else:
        print("\n✅ No warnings")
    
    # Summary statistics
    print("\n" + "-" * 60)
    print("SUMMARY:")
    print(f"  Version: {policy.get('version')}")
    print(f"  Mode: {detection.get('mode')}")
    print(f"  Confidence Threshold: {detection.get('confidence_threshold')}")
    print(f"  Default Strategy: {masking.get('default_strategy')}")
    print(f"  Configured Entity Types: {len(per_type)}/8")
    print(f"  Audit Enabled: {audit.get('log_redactions')}")
    print(f"  FPE Enabled: {policy.get('features', {}).get('enable_fpe', False)}")
    print("-" * 60)
    
    # Count strategies
    strategy_counts = {}
    for entity_type, config in per_type.items():
        strategy = config.get("strategy", "UNKNOWN")
        strategy_counts[strategy] = strategy_counts.get(strategy, 0) + 1
    
    print("\nSTRATEGY DISTRIBUTION:")
    for strategy, count in sorted(strategy_counts.items()):
        print(f"  {strategy}: {count} entity types")
    
    return len(errors) == 0

if __name__ == "__main__":
    success = validate_policy()
    sys.exit(0 if success else 1)
