# Phase 2 — Privacy Guard — Checklist

**Status:** Ready to Execute  
**Last Updated:** 2025-11-03

This checklist mirrors the state in `Phase-2-Agent-State.json` and tracks completion of all Phase 2 deliverables.

---

## Workstream A: Core Guard Implementation

### A1: Project Setup
- [ ] Create `src/privacy-guard/` directory structure
- [ ] Initialize Cargo workspace
- [ ] Add to root workspace members
- [ ] Create module files (main, detection, pseudonym, redaction, policy, state, audit)
- [ ] Add dependencies to Cargo.toml
- [ ] Verify `cargo check` passes

### A2: Detection Engine
- [ ] Define entity type enum (8 types)
- [ ] Implement rules loader from YAML
- [ ] Create regex compiler with timeout safety
- [ ] Implement detection function
- [ ] Add confidence scoring logic
- [ ] Implement Luhn check for credit cards
- [ ] Write unit tests (50+ test cases)
- [ ] All tests pass

### A3: Pseudonymization
- [ ] Read PSEUDO_SALT from environment
- [ ] Implement HMAC-SHA256 function
- [ ] Implement in-memory mapping store (DashMap)
- [ ] Implement reverse lookup for reidentification
- [ ] Write determinism tests
- [ ] Write uniqueness tests
- [ ] All tests pass

### A4: Format-Preserving Encryption
- [ ] Add `fpe` crate dependency
- [ ] Implement FPE wrapper function
- [ ] Implement phone number FPE (preserve area code option)
- [ ] Implement SSN FPE (preserve last 4 option)
- [ ] Write format preservation tests
- [ ] Write determinism tests
- [ ] All tests pass

### A5: Masking Logic
- [ ] Implement mask function
- [ ] Route to pseudonym or FPE based on policy
- [ ] Handle overlapping detections
- [ ] Preserve text structure
- [ ] Generate redaction summary
- [ ] Write integration tests
- [ ] All tests pass

### A6: Policy Engine
- [ ] Define policy struct from policy.yaml
- [ ] Implement mode logic (OFF, DETECT, MASK, STRICT)
- [ ] Implement per-type strategy selection
- [ ] Implement confidence threshold filtering
- [ ] Implement graceful degradation
- [ ] Write unit tests for each mode
- [ ] All tests pass

### A7: HTTP API
- [ ] Set up Axum server
- [ ] Implement GET /status endpoint
- [ ] Implement POST /guard/scan endpoint
- [ ] Implement POST /guard/mask endpoint
- [ ] Implement POST /guard/reidentify endpoint
- [ ] Implement POST /internal/flush-session endpoint
- [ ] Add request/response schemas
- [ ] Add error handling (400, 401, 500)
- [ ] Add request logging
- [ ] Write integration tests
- [ ] All tests pass

### A8: Audit Logging
- [ ] Define audit event schema
- [ ] Emit structured logs on /guard/mask
- [ ] Ensure no raw PII in logs (counts only)
- [ ] Add trace ID propagation
- [ ] Verify logs in test run

---

## Workstream B: Configuration Files

### B1: Rules YAML
- [ ] Create deploy/compose/guard-config/rules.yaml
- [ ] Define SSN patterns (3 variations)
- [ ] Define CREDIT_CARD patterns (4 types with Luhn)
- [ ] Define EMAIL pattern
- [ ] Define PHONE patterns (5 formats)
- [ ] Define PERSON patterns (3 types)
- [ ] Define IP_ADDRESS patterns (IPv4, IPv6)
- [ ] Define DATE_OF_BIRTH patterns (2 types)
- [ ] Define ACCOUNT_NUMBER pattern
- [ ] Add metadata (descriptions, confidence, context)
- [ ] Validate YAML syntax
- [ ] Test each pattern with samples
- [ ] Guard loads rules without errors

### B2: Policy YAML
- [ ] Create deploy/compose/guard-config/policy.yaml
- [ ] Define global settings (mode, confidence threshold)
- [ ] Define per-type strategies (SSN, PHONE, EMAIL, etc.)
- [ ] Define audit settings
- [ ] Validate YAML syntax
- [ ] Guard loads policy without errors
- [ ] Settings applied correctly in tests

### B3: Test Data
- [ ] Create tests/fixtures/pii_samples.txt (100+ lines)
- [ ] Create tests/fixtures/clean_samples.txt (50+ lines)
- [ ] Document expected detections
- [ ] Use in integration tests
- [ ] Guard detects all PII in samples
- [ ] Guard detects zero PII in clean samples
- [ ] False positive rate < 5%

---

## Workstream C: Deployment Integration

### C1: Dockerfile
- [ ] Create src/privacy-guard/Dockerfile
- [ ] Multi-stage build (builder + runtime)
- [ ] Copy config files
- [ ] Expose port 8089
- [ ] Add healthcheck CMD
- [ ] Use non-root user
- [ ] Test build locally
- [ ] Image size < 100MB

### C2: Compose Service
- [ ] Update deploy/compose/ce.dev.yml
- [ ] Add privacy-guard service definition
- [ ] Configure environment variables
- [ ] Map port 8089
- [ ] Mount config volume
- [ ] Add healthcheck
- [ ] Add depends_on (vault)
- [ ] Add profile: privacy-guard
- [ ] Update .env.ce.example (if needed)
- [ ] Test `docker compose --profile privacy-guard up`
- [ ] All services start successfully

### C3: Healthcheck Script
- [ ] Create deploy/compose/healthchecks/guard_health.sh
- [ ] Check /status endpoint
- [ ] Verify response includes mode, rule count, config status
- [ ] Exit codes correct (0 success, 1 failure)
- [ ] Script passes when guard is healthy

### C4: Controller Integration
- [ ] Add GUARD_ENABLED env var to controller
- [ ] Add GUARD_URL env var
- [ ] Implement guard client in controller
- [ ] Call guard in /audit/ingest handler (if enabled)
- [ ] Log redaction counts
- [ ] Handle guard unavailability gracefully
- [ ] Write integration tests
- [ ] Test with GUARD_ENABLED=true
- [ ] Test with GUARD_ENABLED=false
- [ ] All tests pass

---

## Workstream D: Documentation & Testing

### D1: Configuration Guide
- [ ] Create docs/guides/privacy-guard-config.md
- [ ] Document rules.yaml format
- [ ] Document policy.yaml options
- [ ] Show how to add custom patterns
- [ ] Show how to tune confidence thresholds
- [ ] Link to ADRs (0021, 0022)
- [ ] Review for completeness

### D2: Integration Guide
- [ ] Create docs/guides/privacy-guard-integration.md
- [ ] Include curl examples for each endpoint
- [ ] Document controller integration pattern
- [ ] Document future agent-side wrapper (conceptual)
- [ ] Document error handling
- [ ] Link to ADRs (0002, 0021)
- [ ] Review for completeness

### D3: Smoke Test Procedure
- [ ] Create docs/tests/smoke-phase2.md
- [ ] Document startup steps
- [ ] Document healthcheck verification
- [ ] Document /guard/scan test
- [ ] Document /guard/mask test
- [ ] Document determinism test
- [ ] Document reidentify test
- [ ] Document audit log check
- [ ] Document performance benchmarking
- [ ] Include expected outputs
- [ ] Run full smoke test locally
- [ ] Performance meets targets (P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s)

### D4: Update Project Docs
- [ ] Update docs/architecture/mvp.md (add guard flow diagram)
- [ ] Update VERSION_PINS.md (if new images)
- [ ] Update PROJECT_TODO.md (mark Phase 2 complete)
- [ ] Update CHANGELOG.md with Phase 2 changes
- [ ] Review all doc updates

---

## ADRs

- [ ] ADR-0021: Privacy Guard Rust Implementation finalized
- [ ] ADR-0022: PII Detection Rules and FPE finalized

---

## Project Tracking

- [ ] Phase-2-Agent-State.json created and maintained
- [ ] docs/tests/phase2-progress.md entries added
- [ ] All branches follow naming convention
- [ ] All commits use conventional format
- [ ] PR ready for review

---

## Final Acceptance

### Functional
- [ ] Privacy guard service starts and responds
- [ ] /guard/scan detects all 8 entity types
- [ ] /guard/mask produces deterministic pseudonyms
- [ ] FPE preserves format for phone/SSN
- [ ] Same input produces same output (determinism verified)
- [ ] Reidentify endpoint works (with JWT)
- [ ] Controller integration works (optional)
- [ ] Compose integration stable

### Non-Functional
- [ ] Performance: P50 ≤ 500ms measured
- [ ] Performance: P95 ≤ 1s measured
- [ ] Performance: P99 ≤ 2s measured
- [ ] No raw PII in logs (verified)
- [ ] Unit test coverage >80%
- [ ] Integration tests pass
- [ ] Smoke tests pass
- [ ] Docker build < 100MB

### Documentation
- [ ] Configuration guide complete
- [ ] Integration guide complete
- [ ] Smoke test procedure complete
- [ ] ADRs finalized
- [ ] Architecture diagrams updated

---

**Total Tasks:** ~90  
**Completion:** 0% (Ready to start)

**Next Action:** Begin Workstream A, Task A1 (Project Setup)
