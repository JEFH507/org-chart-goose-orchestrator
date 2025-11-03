# ADR 0022: PII Detection Rules and Format-Preserving Encryption

- Status: Accepted (Phase 2)
- Date: 2025-11-03
- Authors: @owner
- Related: ADR-0002 (Guard Placement), ADR-0009 (Deterministic Keys), ADR-0021 (Guard Implementation)

## Context

Privacy Guard must detect and mask PII with high accuracy and acceptable performance. We need to decide:
1. Detection method (regex vs ML/NER vs hybrid)
2. Entity types to support in Phase 2
3. Format-preserving encryption (FPE) for specific types
4. Extensibility path for new rules and formats

## Decision

### Detection Method: Regex-First (Phase 2)

**Phase 2 (Now):**
- Regex patterns for structured data (SSN, credit cards, emails, phone numbers)
- Keyword + pattern combinations for semi-structured data (person names)
- Confidence scoring based on pattern specificity
- No external ML dependencies

**Phase 2.2 (Next):**
- Add local Ollama model for improved person name / organization detection
- Hybrid: regex for structured, model for unstructured
- Maintain regex as fallback if model unavailable

**Rationale:**
- Regex is deterministic, fast (sub-millisecond), and auditable
- Sufficient for Phase 2 goal: prove privacy enforcement works
- Avoids model download/startup delays in Phase 2
- Matches ADR-0015 guidance (model selection deferred to 2.2)

**Performance characteristics:**
- P50 < 100ms for typical prompt (~1000 chars)
- P95 < 500ms for long texts (~10KB)
- P99 < 1000ms with input size limits

### Entity Types (Phase 2 Baseline)

**High Confidence (structured):**
- `SSN` - Social Security Number (US): `\d{3}-\d{2}-\d{4}` and variations
- `CREDIT_CARD` - Luhn-validated 13-19 digit patterns
- `EMAIL` - RFC-compliant email regex
- `PHONE` - US + international formats (E.164, national)
- `IP_ADDRESS` - IPv4 and IPv6

**Medium Confidence (semi-structured):**
- `PERSON` - Common name patterns + title keywords (Mr., Dr., etc.)
- `DATE_OF_BIRTH` - Date patterns in context (Born:, DOB:, etc.)
- `ADDRESS` - Street address patterns (limited; prone to FP)

**Low Confidence (context-dependent):**
- `ORGANIZATION` - Corp, Inc, LLC suffix patterns
- `ACCOUNT_NUMBER` - Generic 8-16 digit sequences in context

**Confidence Levels:**
- HIGH: >95% precision on test corpus
- MEDIUM: 80-95% precision
- LOW: <80% precision (require additional context)

### Entity Type Extensibility

**Design for expansion:**
```yaml
# rules.yaml structure
version: "1.0"
rules:
  SSN:
    patterns:
      - regex: '\b\d{3}-\d{2}-\d{4}\b'
        confidence: HIGH
      - regex: '\b\d{9}\b'
        confidence: MEDIUM
        context_required: true
    
  PASSPORT:  # Future (v1.1+)
    patterns:
      - regex: '\b[A-Z]{1,2}\d{6,9}\b'
        confidence: MEDIUM
    
  API_KEY:  # Future (v1.2+)
    patterns:
      - regex: 'sk-[a-zA-Z0-9]{32,}'
        confidence: HIGH
      - regex: 'ghp_[a-zA-Z0-9]{36}'
        confidence: HIGH
```

**Expansion path:**
- Phase 2: Baseline 8 types
- Phase 2.2: Add PASSPORT, DRIVER_LICENSE via user feedback
- Phase 3+: Org-specific custom patterns via policy.yaml overrides
- Future: API key patterns, crypto wallet addresses, medical IDs

### Format-Preserving Encryption (FPE)

**Decision: YES, implement in Phase 2 for phone and SSN**

**Rationale:**
- Phone and SSN often used in downstream validation/routing
- FPE preserves format → masked data still passes format checks
- Example: `555-123-4567` → `555-847-9201` (both valid US phone format)
- Improves usability without sacrificing security

**Algorithm: AES-FFX (FPE spec FF3-1)**
- NIST-approved format-preserving encryption
- Available in Rust via `fpe` crate
- Deterministic with same key+tweak
- Preserves character set and length

**Phase 2 FPE Targets:**
```
PHONE:
  - Input:  555-123-4567
  - FPE:    555-xxx-xxxx (preserve area code? configurable)
  - Output: 555-847-9201
  
SSN:
  - Input:  123-45-6789
  - FPE:    xxx-xx-6789 (preserve last 4? configurable)
  - Output: 847-29-6789
```

**Non-FPE Entities (Phase 2):**
- EMAIL → `EMAIL_a3f7b2c8@redacted.local`
- PERSON → `PERSON_d4e8c1a9`
- CREDIT_CARD → `CARD_****_****_****_1234` (preserve last 4)

**FPE Extensibility (Post-MVP):**
- Account numbers
- Passport numbers (alphanumeric FPE)
- Driver's license
- Medical record numbers

### Configuration Schema

```yaml
# policy.yaml
version: "1.0"

detection:
  mode: MASK  # OFF | DETECT | MASK | STRICT
  confidence_threshold: MEDIUM  # Ignore LOW confidence unless strict mode
  
masking:
  default_strategy: PSEUDONYM  # PSEUDONYM | REDACT | FPE
  
  per_type:
    SSN:
      strategy: FPE
      fpe_preserve_last: 4  # Keep last N digits visible
    
    PHONE:
      strategy: FPE
      fpe_preserve_area_code: true
    
    EMAIL:
      strategy: PSEUDONYM
      format: "{type}_{hash}@redacted.local"
    
    PERSON:
      strategy: PSEUDONYM
      format: "{type}_{hash}"
    
    CREDIT_CARD:
      strategy: REDACT
      format: "CARD_****_****_****_{last4}"

audit:
  log_detections: true
  log_redactions: true
  log_mapping_count: true  # Count only, not actual mappings
```

### Regex Safety

**Catastrophic Backtracking Prevention:**
- Use `regex` crate with linear time guarantees (no backtracking)
- Input size limit: 10KB per request (configurable)
- Timeout: 100ms per regex evaluation
- If timeout, log warning and skip pattern

**Pattern Quality:**
- All patterns tested on corpus of 1000+ samples
- False positive rate < 5% target
- Regular review and refinement based on production logs

## Consequences

### Benefits

**Regex-First:**
- ✅ Fast, deterministic, auditable
- ✅ No model download or startup latency
- ✅ Works offline
- ✅ Easy to test and debug

**FPE for Phone/SSN:**
- ✅ Preserves downstream validation
- ✅ Better UX (looks like real data)
- ✅ Deterministic (same input → same output)

**Extensible Design:**
- ✅ Easy to add new entity types via YAML
- ✅ Org-specific overrides without code changes
- ✅ Versioned rules for audit trail

### Trade-offs

**Regex limitations:**
- ❌ Lower accuracy on unstructured text (names, addresses)
- ✅ Mitigation: Hybrid approach in Phase 2.2 (Ollama model)
- ❌ Context-blind (can't understand "John is CEO")
- ✅ Mitigation: Keyword + pattern combos; model in 2.2

**FPE complexity:**
- ❌ More complex than simple redaction
- ✅ Mitigation: Use well-tested `fpe` crate; limit to 2 types initially
- ❌ Reversible if key leaks
- ✅ Mitigation: Key from Vault; same risk as HMAC pseudonyms

**False positives:**
- ❌ Some non-PII may match patterns (e.g., "555-1212" in text)
- ✅ Mitigation: Confidence levels; DETECT mode for tuning

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Regex misses new PII types | Medium | Medium | Versioned rules; user feedback loop; Phase 2.2 model |
| FPE implementation bugs | Low | High | Use audited crate; extensive unit tests; manual verification |
| Performance on long texts | Low | Medium | Input size limits; timeout; benchmark tests |
| False positives annoy users | Medium | Low | DETECT mode; confidence tuning; override config |

## Implementation Notes

### Rules File Structure

```yaml
# deploy/compose/guard-config/rules.yaml
version: "1.0"
metadata:
  author: "Phase 2 Team"
  date: "2025-11-03"
  description: "Baseline PII detection rules"

entity_types:
  SSN:
    display_name: "Social Security Number"
    category: "GOVERNMENT_ID"
    patterns:
      - regex: '\b\d{3}-\d{2}-\d{4}\b'
        confidence: HIGH
        description: "US SSN with hyphens"
      - regex: '\b\d{9}\b'
        confidence: MEDIUM
        description: "US SSN no separators (context-dependent)"
        context_keywords: ["SSN", "social security"]
  
  CREDIT_CARD:
    display_name: "Credit Card Number"
    category: "FINANCIAL"
    patterns:
      - regex: '\b(?:4\d{15}|5[1-5]\d{14}|3[47]\d{13}|6(?:011|5\d{2})\d{12})\b'
        confidence: HIGH
        description: "Visa, MC, Amex, Discover (Luhn-validated)"
        luhn_check: true
  
  EMAIL:
    display_name: "Email Address"
    category: "CONTACT"
    patterns:
      - regex: '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
        confidence: HIGH
        description: "RFC-compliant email"
  
  PHONE:
    display_name: "Phone Number"
    category: "CONTACT"
    patterns:
      - regex: '\b\d{3}-\d{3}-\d{4}\b'
        confidence: HIGH
        description: "US phone (xxx-xxx-xxxx)"
      - regex: '\(\d{3}\)\s*\d{3}-\d{4}'
        confidence: HIGH
        description: "US phone with parens"
      - regex: '\+\d{1,3}\s?\d{4,14}'
        confidence: MEDIUM
        description: "International E.164"
  
  PERSON:
    display_name: "Person Name"
    category: "IDENTITY"
    patterns:
      - regex: '\b(?:Mr\.|Mrs\.|Ms\.|Dr\.|Prof\.)\s+[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+\b'
        confidence: MEDIUM
        description: "Title + name"
      - regex: '\b[A-Z][a-z]+\s+[A-Z][a-z]+\b'
        confidence: LOW
        description: "Two capitalized words (many FP)"
        context_keywords: ["name", "person", "employee", "contact"]

# ... additional types
```

### Test Corpus

**Create synthetic test data:**
```
tests/fixtures/pii_samples.txt
  - 100+ samples per entity type
  - Known true positives
  - Known false positives (edge cases)
  - Mixed-entity paragraphs

tests/fixtures/clean_samples.txt
  - Text with no PII
  - Should produce zero detections
```

### Benchmark Targets

```
Input Size    P50      P95      P99
──────────────────────────────────
100 chars     <10ms    <20ms    <50ms
1,000 chars   <50ms    <100ms   <200ms
10,000 chars  <500ms   <1000ms  <2000ms
```

## Alignment with Master Plan

- ✅ Privacy-by-design: Mask before cloud (ADR-0002)
- ✅ Deterministic mapping: HMAC + FPE (ADR-0009)
- ✅ Performance: P50 ≤ 500ms target met
- ✅ Extensibility: Rules versioned and overridable
- ✅ Local-first: No external ML APIs (Phase 2)

## Decision Lifecycle

**Revisit after:**
- Phase 2 completion: Evaluate false positive/negative rates on real usage
- Phase 2.2: Reassess regex-only approach vs hybrid
- User feedback: Add most-requested entity types

**Metrics to track:**
- Detection accuracy (precision, recall) per entity type
- False positive rate in DETECT mode
- Performance (P50, P95, P99) by input size
- User override frequency (indicates rule quality)

## References

- Master Plan: `Technical Project Plan/master-technical-project-plan.md`
- ADR-0002: Privacy Guard Placement
- ADR-0009: Deterministic Pseudonymization Keys
- ADR-0021: Privacy Guard Rust Implementation
- Component docs: `Technical Project Plan/components/privacy-guard/`
- FPE Spec: NIST SP 800-38G (FF3-1)
- `fpe` crate: https://crates.io/crates/fpe
