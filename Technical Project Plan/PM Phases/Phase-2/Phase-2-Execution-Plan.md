# Phase 2 — Privacy Guard — Execution Plan (Detailed)

**Status:** Approved  
**Owner:** Phase 2 Orchestrator  
**Size:** Medium (3-5 days)  
**Date:** 2025-11-03

---

## Objectives

Implement the foundational Privacy Guard service with:
- Regex-based PII detection (8 entity types)
- Deterministic pseudonymization (HMAC-SHA256)
- Format-preserving encryption for phone/SSN
- HTTP API (scan, mask, reidentify)
- Docker Compose integration
- Performance target: P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s

---

## Scope

### In Scope (Phase 2)
- ✅ Rust HTTP service (Axum framework)
- ✅ Regex detection engine with confidence scoring
- ✅ HMAC-based pseudonymization with per-tenant salt
- ✅ Format-preserving encryption for PHONE and SSN
- ✅ In-memory mapping state (session-scoped)
- ✅ Configuration via YAML (rules.yaml, policy.yaml)
- ✅ Docker Compose `privacy-guard` service
- ✅ Controller integration (optional via env flag)
- ✅ Synthetic test data for validation
- ✅ Performance benchmarking

### Out of Scope (Deferred)
- ❌ Local LLM/NER integration (Phase 2.2)
- ❌ Provider middleware wrapper (Phase 3+)
- ❌ Image/file redaction (Post-MVP)
- ❌ Persistent mapping state (not needed for MVP)
- ❌ Multi-tenant key isolation (single tenant for MVP)
- ❌ Dashboard/UI (Post-MVP)

---

## Deliverables

### Code
- [ ] `src/privacy-guard/` Rust workspace
- [ ] Detection engine (`detection.rs`)
- [ ] Pseudonymization logic (`pseudonym.rs`)
- [ ] FPE implementation (`redaction.rs`)
- [ ] HTTP API (`main.rs`, `api.rs`)
- [ ] Policy engine (`policy.rs`)
- [ ] State management (`state.rs`)
- [ ] Audit logging (`audit.rs`)
- [ ] Unit tests (>80% coverage target)
- [ ] Integration tests (API endpoints)

### Configuration
- [ ] `deploy/compose/guard-config/rules.yaml` (8 entity types)
- [ ] `deploy/compose/guard-config/policy.yaml` (modes, FPE settings)
- [ ] `tests/fixtures/pii_samples.txt` (synthetic test data)
- [ ] `tests/fixtures/clean_samples.txt` (no-PII baseline)

### Deployment
- [ ] `src/privacy-guard/Dockerfile` (multi-stage Rust build)
- [ ] Compose service in `deploy/compose/ce.dev.yml`
- [ ] Healthcheck script (`deploy/compose/healthchecks/guard_health.sh`)
- [ ] Environment variables in `.env.ce.example`

### Documentation
- [ ] `docs/guides/privacy-guard-config.md`
- [ ] `docs/guides/privacy-guard-integration.md`
- [ ] `docs/tests/smoke-phase2.md`
- [ ] `docs/architecture/mvp.md` (add guard flow diagram)
- [ ] ADR-0021 (Rust implementation)
- [ ] ADR-0022 (Detection rules and FPE)

### Project Tracking
- [ ] `Phase-2-Agent-State.json`
- [ ] `docs/tests/phase2-progress.md`
- [ ] `PROJECT_TODO.md` updates

---

## Dependencies

### Prerequisites (Already Complete)
- ✅ Vault wiring for PSEUDO_SALT (Phase 1.2, ADR-0020)
- ✅ Docker Compose infrastructure (Phase 0)
- ✅ Rust toolchain and patterns (Phase 1)
- ✅ Controller runtime with JWT (Phase 1.2)

### New Dependencies (Rust Crates)
- `axum` 0.7 - HTTP framework
- `tokio` 1.x - Async runtime
- `regex` 1.x - Pattern matching
- `hmac` 0.12, `sha2` 0.10 - Cryptographic hashing
- `fpe` 0.6 - Format-preserving encryption (AES-FFX)
- `serde`, `serde_json`, `serde_yaml` - Serialization
- `tracing` / `tracing-subscriber` - Logging
- `dashmap` 0.5 - Concurrent HashMap (for state)

---

## Workstreams and Tasks

### Workstream A: Core Guard Implementation

**Branch:** `feat/phase2-guard-core`

#### Task A1: Project Setup (1 hour)
**Objective:** Create Rust workspace and module structure

**Actions:**
1. Create `src/privacy-guard/` directory
2. Initialize Cargo workspace
3. Add to root `Cargo.toml` workspace members
4. Create module files (main, detection, pseudonym, redaction, policy, state, audit)
5. Add dependencies to `Cargo.toml`
6. Verify `cargo check` passes

**Acceptance:**
- Workspace compiles
- All module declarations resolve
- Dependencies fetched

**Artifacts:**
- `src/privacy-guard/Cargo.toml`
- `src/privacy-guard/src/*.rs` (empty modules)

---

#### Task A2: Detection Engine (4-6 hours)
**Objective:** Implement regex-based PII detection with confidence scoring

**Actions:**
1. Define entity type enum (SSN, EMAIL, PHONE, CREDIT_CARD, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER)
2. Implement rules loader from YAML (parse `rules.yaml`)
3. Create regex compiler with timeout safety
4. Implement detection function:
   ```rust
   fn detect(text: &str, rules: &Rules) -> Vec<Detection> {
       // Detection { start, end, entity_type, confidence, matched_text }
   }
   ```
5. Add confidence scoring logic
6. Implement Luhn check for credit cards
7. Unit tests for each entity type (50+ test cases)

**Acceptance:**
- All 8 entity types detect correctly on test samples
- Confidence levels match expectations (HIGH/MEDIUM/LOW)
- No false positives on clean text
- Unit tests pass

**Artifacts:**
- `src/privacy-guard/src/detection.rs`
- `src/privacy-guard/src/rules.rs`
- Unit tests

---

#### Task A3: Pseudonymization (3-4 hours)
**Objective:** Implement HMAC-SHA256 deterministic mapping

**Actions:**
1. Read `PSEUDO_SALT` from environment
2. Implement HMAC function:
   ```rust
   fn pseudonymize(
       text: &str,
       entity_type: EntityType,
       tenant_id: &str,
       salt: &str
   ) -> String {
       // HMAC-SHA256(salt, tenant_id || entity_type || text)
       // Return format: "{TYPE}_{hash}"
   }
   ```
3. Implement in-memory mapping store (DashMap)
4. Implement reverse lookup for reidentification
5. Unit tests for determinism (same input → same output)
6. Unit tests for uniqueness (different inputs → different outputs)

**Acceptance:**
- Same text produces same pseudonym (determinism test)
- Different texts produce different pseudonyms
- Reverse lookup works
- State is thread-safe

**Artifacts:**
- `src/privacy-guard/src/pseudonym.rs`
- `src/privacy-guard/src/state.rs`
- Unit tests

---

#### Task A4: Format-Preserving Encryption (4-5 hours)
**Objective:** Implement FPE for phone and SSN

**Actions:**
1. Add `fpe` crate dependency
2. Implement FPE wrapper:
   ```rust
   fn fpe_encrypt(
       text: &str,
       entity_type: EntityType,
       key: &[u8],
       preserve_config: &PreserveConfig
   ) -> Result<String> {
       // Use AES-FFX (FF3-1)
   }
   ```
3. Implement phone number FPE:
   - Parse format (xxx-xxx-xxxx, (xxx) xxx-xxxx, etc.)
   - Optionally preserve area code
   - Apply FPE to remaining digits
   - Reconstruct formatted string
4. Implement SSN FPE:
   - Parse format (xxx-xx-xxxx or xxxxxxxxx)
   - Optionally preserve last 4 digits
   - Apply FPE to remaining digits
   - Reconstruct formatted string
5. Unit tests for format preservation
6. Unit tests for determinism

**Acceptance:**
- Phone FPE preserves format: `555-123-4567` → `555-XXX-XXXX` (valid format)
- SSN FPE preserves format: `123-45-6789` → `XXX-XX-6789` (if last 4 preserved)
- Deterministic: same input → same output
- Reversible with same key

**Artifacts:**
- `src/privacy-guard/src/redaction.rs` (FPE module)
- Unit tests

---

#### Task A5: Masking Logic (3-4 hours)
**Objective:** Implement text replacement with pseudonyms or FPE

**Actions:**
1. Implement mask function:
   ```rust
   fn mask(
       text: &str,
       detections: Vec<Detection>,
       policy: &Policy,
       state: &MappingState
   ) -> MaskResult {
       // MaskResult { masked_text, redactions_count, entity_summary }
   }
   ```
2. Route to pseudonym or FPE based on entity type and policy
3. Handle overlapping detections (priority: higher confidence first)
4. Preserve text structure (newlines, whitespace)
5. Generate redaction summary (counts by entity type)
6. Integration tests (full text with multiple PII types)

**Acceptance:**
- Text with multiple PII types correctly masked
- Overlapping entities handled (no corruption)
- Redaction summary accurate
- Original text length approximately preserved

**Artifacts:**
- `src/privacy-guard/src/redaction.rs` (mask module)
- Integration tests

---

#### Task A6: Policy Engine (2-3 hours)
**Objective:** Load and apply policy configuration

**Actions:**
1. Define policy struct from `policy.yaml`
2. Implement mode logic (OFF, DETECT, MASK, STRICT)
3. Implement per-type strategy selection (PSEUDONYM, FPE, REDACT)
4. Implement confidence threshold filtering
5. Implement graceful degradation (missing config → defaults)
6. Unit tests for each mode

**Acceptance:**
- OFF mode: no masking
- DETECT mode: detection only, no masking
- MASK mode: full masking
- STRICT mode: error on detection
- Policy overrides work

**Artifacts:**
- `src/privacy-guard/src/policy.rs`
- Unit tests

---

#### Task A7: HTTP API (4-5 hours)
**Objective:** Implement REST endpoints with Axum

**Actions:**
1. Set up Axum server in `main.rs`
2. Implement endpoints:
   - `GET /status` - Healthcheck (config status, mode, rule count)
   - `POST /guard/scan` - Detection only (return entity list)
   - `POST /guard/mask` - Full masking (return masked text + summary)
   - `POST /guard/reidentify` - Reverse mapping (auth required)
   - `POST /internal/flush-session` - Clear state for session
3. Add request/response schemas (serde)
4. Add error handling (400, 401, 500)
5. Add request logging (tracing)
6. Integration tests (curl or reqwest)

**Request/Response Examples:**
```json
// POST /guard/scan
Request:
{
  "text": "Contact John Doe at 555-123-4567 or john.doe@example.com",
  "tenant_id": "org1"
}

Response:
{
  "detections": [
    {"start": 8, "end": 16, "type": "PERSON", "confidence": "MEDIUM"},
    {"start": 20, "end": 32, "type": "PHONE", "confidence": "HIGH"},
    {"start": 36, "end": 56, "type": "EMAIL", "confidence": "HIGH"}
  ]
}

// POST /guard/mask
Request:
{
  "text": "Contact John Doe at 555-123-4567",
  "tenant_id": "org1",
  "mode": "MASK"
}

Response:
{
  "masked_text": "Contact PERSON_a3f7b2c8 at 555-847-9201",
  "redactions": {
    "PERSON": 1,
    "PHONE": 1
  },
  "session_id": "sess_abc123"
}

// POST /guard/reidentify
Request:
{
  "pseudonym": "PERSON_a3f7b2c8",
  "session_id": "sess_abc123"
}
Headers:
  Authorization: Bearer <JWT>

Response:
{
  "original": "John Doe"
}
```

**Acceptance:**
- All endpoints return 200 for valid requests
- Error responses have correct status codes
- JWT auth works on `/guard/reidentify`
- Logs show request metadata (no PII)

**Artifacts:**
- `src/privacy-guard/src/main.rs`
- `src/privacy-guard/src/api.rs`
- Integration tests

---

#### Task A8: Audit Logging (2 hours)
**Objective:** Implement redaction event logging

**Actions:**
1. Define audit event schema:
   ```rust
   struct RedactionEvent {
       timestamp, tenant_id, session_id,
       mode, entity_counts, total_redactions,
       performance_ms, trace_id
   }
   ```
2. Emit structured logs on each `/guard/mask` call
3. Never log raw PII or pseudonym mappings
4. Log only counts and metadata
5. Add trace ID propagation (for OTLP later)

**Acceptance:**
- Logs contain counts but no PII
- Structured format (JSON)
- Trace IDs present

**Artifacts:**
- `src/privacy-guard/src/audit.rs`

---

### Workstream B: Configuration Files

**Branch:** `feat/phase2-guard-config`

#### Task B1: Rules YAML (3-4 hours)
**Objective:** Create baseline PII detection rules

**Actions:**
1. Create `deploy/compose/guard-config/rules.yaml`
2. Define 8 entity types with patterns:
   - SSN: 3 patterns (with/without hyphens, context)
   - CREDIT_CARD: 4 patterns (Visa, MC, Amex, Discover with Luhn)
   - EMAIL: 1 pattern (RFC-compliant)
   - PHONE: 5 patterns (US formats, international)
   - PERSON: 3 patterns (with titles, two-word names)
   - IP_ADDRESS: 2 patterns (IPv4, IPv6)
   - DATE_OF_BIRTH: 2 patterns (with context keywords)
   - ACCOUNT_NUMBER: 1 pattern (context-dependent)
3. Add metadata (descriptions, confidence levels, context keywords)
4. Validate YAML syntax
5. Test each pattern manually

**Acceptance:**
- YAML is valid
- Guard loads rules without errors
- Each pattern tested with 5+ samples
- Log shows rule count on startup

**Artifacts:**
- `deploy/compose/guard-config/rules.yaml`

---

#### Task B2: Policy YAML (2 hours)
**Objective:** Define masking policy defaults

**Actions:**
1. Create `deploy/compose/guard-config/policy.yaml`
2. Define global settings:
   - Default mode: MASK
   - Confidence threshold: MEDIUM
3. Define per-type strategies:
   - SSN: FPE (preserve last 4)
   - PHONE: FPE (preserve area code)
   - EMAIL: PSEUDONYM
   - PERSON: PSEUDONYM
   - CREDIT_CARD: REDACT (show last 4)
   - Others: PSEUDONYM
4. Define audit settings
5. Validate YAML syntax

**Acceptance:**
- YAML is valid
- Guard loads policy without errors
- Settings applied correctly in tests

**Artifacts:**
- `deploy/compose/guard-config/policy.yaml`

---

#### Task B3: Test Data (2 hours)
**Objective:** Create synthetic PII samples for testing

**Actions:**
1. Create `tests/fixtures/pii_samples.txt`
   - 100+ lines with known PII
   - Multiple entity types per line
   - Edge cases (formats, international)
2. Create `tests/fixtures/clean_samples.txt`
   - 50+ lines with no PII
   - Should produce zero detections
3. Document expected detections for each sample
4. Use in integration tests

**Acceptance:**
- Guard detects all PII in pii_samples.txt
- Guard detects zero PII in clean_samples.txt
- False positive rate < 5%

**Artifacts:**
- `tests/fixtures/pii_samples.txt`
- `tests/fixtures/clean_samples.txt`
- `tests/fixtures/expected_detections.json`

---

### Workstream C: Deployment Integration

**Branch:** `feat/phase2-guard-deploy`

#### Task C1: Dockerfile (2-3 hours)
**Objective:** Create multi-stage Docker build

**Actions:**
1. Create `src/privacy-guard/Dockerfile`
2. Multi-stage build:
   - Builder: `rust:1.83-bookworm`
   - Runtime: `debian:bookworm-slim` or `rust:1.83-slim`
3. Copy rules/policy config into image or mount
4. Expose port 8089
5. Healthcheck CMD
6. Non-root user
7. Test build locally

**Example:**
```dockerfile
FROM rust:1.83-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/privacy-guard /usr/local/bin/
COPY --from=builder /build/config /etc/guard-config
EXPOSE 8089
USER nobody
HEALTHCHECK CMD curl -f http://localhost:8089/status || exit 1
CMD ["privacy-guard"]
```

**Acceptance:**
- Docker build succeeds
- Image size < 100MB (optimized)
- Container starts and responds to health check

**Artifacts:**
- `src/privacy-guard/Dockerfile`

---

#### Task C2: Compose Service (2 hours)
**Objective:** Add privacy-guard to Docker Compose

**Actions:**
1. Update `deploy/compose/ce.dev.yml`
2. Add `privacy-guard` service:
   ```yaml
   privacy-guard:
     build:
       context: ../../src/privacy-guard
     environment:
       - PSEUDO_SALT=${PSEUDO_SALT}
       - GUARD_PORT=8089
       - GUARD_MODE=${GUARD_MODE:-MASK}
       - RUST_LOG=${GUARD_LOG_LEVEL:-info}
     ports:
       - "8089:8089"
     volumes:
       - ./guard-config:/etc/guard-config:ro
     healthcheck:
       test: ["CMD", "curl", "-f", "http://localhost:8089/status"]
       interval: 10s
       timeout: 3s
       retries: 3
     depends_on:
       vault:
         condition: service_healthy
     profiles:
       - privacy-guard
   ```
3. Update `.env.ce.example` with guard variables (if needed)
4. Test `docker compose --profile privacy-guard up`

**Acceptance:**
- Compose starts all services including guard
- Healthcheck passes
- Guard reachable at http://localhost:8089/status

**Artifacts:**
- `deploy/compose/ce.dev.yml`

---

#### Task C3: Healthcheck Script (1 hour)
**Objective:** Verify guard service health

**Actions:**
1. Create `deploy/compose/healthchecks/guard_health.sh`
2. Check `/status` endpoint
3. Verify response includes:
   - Mode (MASK)
   - Rule count (>0)
   - Config loaded: true
4. Exit 0 on success, 1 on failure

**Acceptance:**
- Script passes when guard is healthy
- Script fails when guard is down or misconfigured

**Artifacts:**
- `deploy/compose/healthchecks/guard_health.sh`

---

#### Task C4: Controller Integration (3 hours)
**Objective:** Optional guard call from controller

**Actions:**
1. Add environment variable to controller: `GUARD_ENABLED=false`
2. Add `GUARD_URL=http://privacy-guard:8089`
3. In controller `/audit/ingest` handler:
   ```rust
   if GUARD_ENABLED {
       let masked = call_guard_mask(&event.content).await?;
       event.content = masked.masked_text;
       event.redactions = masked.redactions;
   }
   ```
4. Log redaction counts
5. Handle guard unavailability gracefully (fail open or closed?)
6. Integration test with both enabled/disabled

**Acceptance:**
- Controller can call guard when enabled
- Audit events contain redaction counts
- Graceful degradation if guard is down (configurable: fail-open default)

**Artifacts:**
- `src/controller/src/guard_client.rs`
- Updated `src/controller/src/main.rs`
- Integration tests

---

### Workstream D: Documentation & Testing

**Branch:** `docs/phase2-guides`

#### Task D1: Configuration Guide (2-3 hours)
**Objective:** Document how to configure privacy guard

**Actions:**
1. Create `docs/guides/privacy-guard-config.md`
2. Document `rules.yaml` format and entity types
3. Document `policy.yaml` modes and strategies
4. Show how to add custom patterns
5. Show how to tune confidence thresholds
6. Link to ADR-0021, ADR-0022

**Acceptance:**
- Guide is complete with examples
- User can add a new entity type following the guide

**Artifacts:**
- `docs/guides/privacy-guard-config.md`

---

#### Task D2: Integration Guide (2-3 hours)
**Objective:** Document how to use privacy guard

**Actions:**
1. Create `docs/guides/privacy-guard-integration.md`
2. Show curl examples for each endpoint
3. Show controller integration pattern
4. Show future agent-side wrapper (conceptual, Phase 3)
5. Document error handling
6. Link to ADR-0002, ADR-0021

**Acceptance:**
- Guide includes working curl examples
- Integration patterns clear

**Artifacts:**
- `docs/guides/privacy-guard-integration.md`

---

#### Task D3: Smoke Test Procedure (3 hours)
**Objective:** E2E validation checklist

**Actions:**
1. Create `docs/tests/smoke-phase2.md`
2. Document test steps:
   - Start compose with privacy-guard profile
   - Verify healthcheck
   - POST to /guard/scan with sample PII
   - Verify detections
   - POST to /guard/mask
   - Verify masked output format
   - Test determinism (same input twice)
   - Test reidentify endpoint
   - Check audit logs (no PII)
   - Performance test (measure P50/P95/P99)
3. Include expected outputs
4. Include performance benchmarking commands

**Acceptance:**
- All smoke test steps pass locally
- Performance meets targets (P50 ≤ 500ms)

**Artifacts:**
- `docs/tests/smoke-phase2.md`

---

#### Task D4: Update Project Docs (2 hours)
**Objective:** Sync architecture and tracking docs

**Actions:**
1. Update `docs/architecture/mvp.md`:
   - Add privacy guard flow diagram
   - Update component list
2. Update `VERSION_PINS.md` if new images
3. Update `PROJECT_TODO.md`:
   - Mark Phase 2 tasks complete
   - Update completion status
4. Update `CHANGELOG.md` with Phase 2 changes

**Acceptance:**
- Architecture docs reflect Phase 2 changes
- PROJECT_TODO accurate

**Artifacts:**
- Updated docs

---

## Acceptance Criteria (Overall)

### Functional
- ✅ Privacy guard service starts and responds to health checks
- ✅ `/guard/scan` detects all 8 entity types correctly
- ✅ `/guard/mask` produces deterministic pseudonyms
- ✅ FPE preserves format for phone/SSN
- ✅ Same input produces same output (determinism test)
- ✅ Reidentify endpoint reverses mapping (with JWT)
- ✅ Controller integration works (optional flag)
- ✅ Compose integration stable

### Non-Functional
- ✅ Performance: P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s
- ✅ No raw PII in logs
- ✅ Unit test coverage >80%
- ✅ Integration tests pass
- ✅ Smoke tests pass locally
- ✅ Docker build < 100MB
- ✅ Graceful degradation when PSEUDO_SALT missing

### Documentation
- ✅ Configuration guide complete
- ✅ Integration guide complete
- ✅ Smoke test procedure complete
- ✅ ADR-0021 and ADR-0022 finalized
- ✅ Architecture diagrams updated

### Process
- ✅ All commits conventional format (feat/fix/docs/test)
- ✅ State JSON updated after each workstream
- ✅ Progress log entries with timestamps
- ✅ Feature branches ready for PR
- ✅ PROJECT_TODO reflects Phase 2 complete

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Regex performance issues | Low | Medium | Input size limits, timeout, benchmarking |
| FPE library bugs | Low | High | Use well-tested crate, extensive tests, manual verification |
| PSEUDO_SALT not set | Medium | High | Graceful degradation to OFF mode with clear error |
| False positives annoy users | Medium | Low | DETECT mode for tuning, confidence levels, user feedback |
| Controller integration breaks existing flows | Low | Medium | Optional flag, thorough integration tests, fail-open default |
| Timeline slip on FPE complexity | Medium | Low | FPE optional per entity type, can defer if needed |

---

## Timeline (Indicative)

**Week 1 (Days 1-2):**
- A1: Project setup
- A2: Detection engine
- B1: Rules YAML
- B3: Test data

**Week 1-2 (Days 2-3):**
- A3: Pseudonymization
- A4: FPE implementation
- B2: Policy YAML
- A6: Policy engine

**Week 2 (Days 3-4):**
- A5: Masking logic
- A7: HTTP API
- A8: Audit logging
- C1: Dockerfile
- C2: Compose service

**Week 2 (Day 4-5):**
- C3: Healthcheck
- C4: Controller integration
- D1-D4: Documentation and smoke tests
- Final acceptance and PR

**Buffer:** Day 5 for fixes, performance tuning, and polish

---

## ADR Alignment

- ✅ ADR-0002: Agent-side guard placement (HTTP service accessible to agents)
- ✅ ADR-0005: Metadata-only storage (no persistence of mappings)
- ✅ ADR-0008: Audit with redaction counts (no raw PII in logs)
- ✅ ADR-0009: Deterministic HMAC pseudonymization
- ✅ ADR-0020: PSEUDO_SALT from Vault via environment
- ✅ ADR-0021: Rust implementation, HTTP service, in-memory state
- ✅ ADR-0022: Regex detection, FPE for phone/SSN, extensible rules

---

## References

- Master Plan: `Technical Project Plan/master-technical-project-plan.md`
- Component Docs: `Technical Project Plan/components/privacy-guard/`
- Phase 1.2 Summary: `Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Completion-Summary.md`
- ADRs: `docs/adr/0002-*.md`, `docs/adr/0021-*.md`, `docs/adr/0022-*.md`
