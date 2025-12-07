# Phase 2 — Privacy Guard (M) — Agent Prompts

**Purpose:** Implement baseline privacy guard with regex detection, deterministic pseudonymization, FPE for phone/SSN, and HTTP API to enforce mask-and-forward by default.

**Builds on:** Phase 0 (infra), Phase 1 (controller), Phase 1.2 (JWT, Vault)

---

## How to Use This Prompt

### Starting a New Session (First Time)
Copy the entire "Master Orchestrator Prompt" section below and paste it into a new goose session.

### Resuming Work (Returning Later)
Copy the "Resume Prompt" section below and paste it into goose. It will read your state and continue where you left off.

---

## Resume Prompt — Copy this block when resuming Phase 2

```markdown
You are resuming Phase 2 orchestration for goose-org-twin.

**Context:**
- Phase: 2 — Privacy Guard (Medium)
- Repository: /home/papadoc/Gooseprojects/goose-org-twin

**Required Actions:**
1. Read state from: `Technical Project Plan/PM Phases/Phase-2/Phase-2-Agent-State.json`
2. Read last progress entry from: `docs/tests/phase2-progress.md`
3. Review tracking validation: `Technical Project Plan/PM Phases/Phase-2/RESUME-VALIDATION.md` (optional but helpful)
4. **Review deviations:** `Technical Project Plan/PM Phases/Phase-2/DEVIATIONS-LOG.md` - documents all hiccups, fixes, and lessons learned
5. Must Re-read authoritative documents:
   - `Technical Project Plan/master-technical-project-plan.md`
   - `Technical Project Plan/PM Phases/Phase-2/Phase-2-Agent-Prompts.md`
   - `Technical Project Plan/PM Phases/Phase-2/Phase-2-Checklist.md`
   - `Technical Project Plan/PM Phases/Phase-2/Phase-2-Execution-Plan.md`
   - Relevant ADRs: 0002, 0005, 0008, 0009, 0015, 0020, 0021, 0022

**Summarize for me:**
- Current workstream and task_id from state JSON
- Last step completed
- Pending questions (if any)
- Next unchecked item in checklist

**Then proceed with:**
- If pending_questions exist: ask them and wait for my answers
- Otherwise: continue with the next step in the execution sequence
- Maintain the same guardrails, state persistence, and progress logging protocols

**Guardrails (DO NOT VIOLATE):**
- HTTP-only orchestrator; metadata-only server model
- No secrets in git; .env.ce samples only
- No raw PII in logs (counts and types only)
- Keep CI stable; run tests locally
- Update state JSON, checklist, RESUME-VALIDATION, and progress log after each milestone
```

---

## Master Orchestrator Prompt — Copy this block for a new session

**Role:** Phase 2 Orchestrator for goose-org-twin

You are an engineering orchestrator responsible for executing Phase 2: Privacy Guard. You will implement a Rust-based HTTP service with regex PII detection, HMAC pseudonymization, format-preserving encryption, and integration with existing infrastructure. Maintain HTTP-only posture, metadata-only storage, and never log raw PII. Be pause/resume capable and persist state.

### Project Context

**Project root:** `/home/papadoc/Gooseprojects/goose-org-twin`

**Always read these source documents by absolute path at start and after resume:**
- `Technical Project Plan/master-technical-project-plan.md`
- `Technical Project Plan/PM Phases/Phase-2/Phase-2-Execution-Plan.md`
- `Technical Project Plan/PM Phases/Phase-2/Phase-2-Checklist.md`
- Prior phase summaries:
  - `Technical Project Plan/PM Phases/Phase-0/Phase-0-Summary.md` (if exists)
  - `Technical Project Plan/PM Phases/Phase-1/Phase-1-Completion-Summary.md`
  - `Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Completion-Summary.md`
- Relevant ADRs:
  - `docs/adr/0002-privacy-guard-placement.md`
  - `docs/adr/0005-data-retention-and-redaction.md`
  - `docs/adr/0008-audit-schema-and-redaction.md`
  - `docs/adr/0009-deterministic-pseudonymization-keys.md`
  - `docs/adr/0015-guard-model-policy-and-selection.md`
  - `docs/adr/0020-vault-oss-wiring.md`
  - `docs/adr/0021-privacy-guard-rust-implementation.md`
  - `docs/adr/0022-pii-detection-rules-and-fpe.md`
- Component docs:
  - `Technical Project Plan/components/privacy-guard/requirements.md`
  - `Technical Project Plan/components/privacy-guard/plan.md`
- Guides:
  - `docs/guides/guard-model-selection.md`
  - `docs/security/secrets-bootstrap.md`
- Version pins: `VERSION_PINS.md`

### State Persistence (Mandatory)

**State file:**
- `Technical Project Plan/PM Phases/Phase-2/Phase-2-Agent-State.json`

**Schema:**
```json
{
  "current_workstream": "INIT|A|B|C|D|DONE",
  "current_task_id": "A1|A2|...|D4",
  "last_step_completed": "free text",
  "branches": {
    "A": "feat/phase2-guard-core",
    "B": "feat/phase2-guard-config",
    "C": "feat/phase2-guard-deploy",
    "D": "docs/phase2-guides"
  },
  "user_inputs": {
    "os": "linux",
    "docker_available": true,
    "guard_port": 8089,
    "enable_controller_integration": true,
    "performance_targets": {"p50_ms": 500, "p95_ms": 1000, "p99_ms": 2000},
    "include_fpe": true,
    "create_test_data": true
  },
  "pending_questions": [],
  "checklist": {"A1": "todo|in-progress|done", ...},
  "artifacts": {"adrs": [], "docs": [], "code": [], "config": [], "tests": []},
  "performance_results": {"p50_ms": null, "p95_ms": null, "p99_ms": null}
}
```

**Log progress to:** `docs/tests/phase2-progress.md` (append entries with timestamps, branches, commits, acceptance checks)

### Pause/Resume Protocol

When you need user input:
1. Write/update the state file with pending question(s) and current position (workstream, task)
2. Append note to `docs/tests/phase2-progress.md` describing what you're waiting for
3. Stop and ask the question clearly
4. After user responds, re-read state and continue

### Extensions & Tools

**Assumed available:**
- `developer` (file I/O + shell)
- `todo` (optional; mirror phase checklist)
- `github` (for PR ops) if available; otherwise provide web UI instructions

### Git/GitHub Workflow (SSH-first, minimal prompts)

**Policy:**
- Detect current branch and remotes automatically; store in state
- Use sensible defaults:
  - base_branch = main
  - Use current branch for commits
  - Infer tags from VERSION_PINS.md
- SSH-first for remote actions; prefer GNOME askpass:
  ```bash
  export DISPLAY=${DISPLAY:-:0}
  export SSH_ASKPASS_REQUIRE=force
  SSH_ASKPASS="$(command -v ssh-askpass-gnome || command -v ssh-askpass || true)"
  if [ -n "$SSH_ASKPASS" ]; then setsid -w ssh-add ~/.ssh/id_ed25519 < /dev/null; fi
  ```
- Prefer fast-forward pulls on main. **Never force-push shared branches**

**Per workstream:**
- Create feature branch (naming: `feat/phase2-*` or `docs/phase2-*`)
- Commit with conventional commits (feat/fix/docs/test/chore)
- Push if remote exists; proceed locally and note in progress log otherwise
- Provide ready-to-paste PR title and body

### Global Guardrails (DO NOT VIOLATE)

- HTTP-only posture (no message bus)
- Metadata-only server model (no PII persistence)
- **No raw PII in logs** (counts and entity types only)
- **No secrets in git** (PSEUDO_SALT via env only)
- Pin container images (no `:latest`)
- Do not commit local `.env.ce`
- Local-first processing (guard runs in org zone, not cloud)
- Performance targets: P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s

### Phase 2 Specific Guardrails

- Default mode = MASK (mask-and-forward per ADR-0002)
- Deterministic mapping (same input → same pseudonym per tenant)
- In-memory state only (no persistence of mappings)
- Graceful degradation if PSEUDO_SALT missing (warn and use OFF mode)
- Format-preserving encryption for PHONE and SSN
- Test data must be synthetic (no real PII)

### Before Starting Workstream A

**User inputs already confirmed (from state JSON):**
- OS: linux
- Docker: available
- Git identity: Javier / 132608441+JEFH507@users.noreply.github.com
- Remote: git@github.com:JEFH507/org-chart-goose-orchestrator.git
- Guard port: 8089
- Controller integration: enabled
- Performance targets: P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s
- Include FPE: yes
- Create test data: yes

**If any are missing, ask before proceeding.**

### Execution Sequence

Execute workstreams in order. Update state JSON and progress log after each task.

**Workstream A: Core Guard Implementation**
- Branch: `feat/phase2-guard-core`
- Tasks: A1 (setup), A2 (detection), A3 (pseudonym), A4 (FPE), A5 (masking), A6 (policy), A7 (HTTP API), A8 (audit)
- Run all sub-prompts A1 through A8

**Workstream B: Configuration Files**
- Branch: `feat/phase2-guard-config`
- Tasks: B1 (rules.yaml), B2 (policy.yaml), B3 (test data)
- Run all sub-prompts B1 through B3

**Workstream C: Deployment Integration**
- Branch: `feat/phase2-guard-deploy`
- Tasks: C1 (Dockerfile), C2 (compose), C3 (healthcheck), C4 (controller integration)
- Run all sub-prompts C1 through C4

**Workstream D: Documentation & Testing**
- Branch: `docs/phase2-guides`
- Tasks: D1 (config guide), D2 (integration guide), D3 (smoke tests), D4 (project docs)
- Run all sub-prompts D1 through D4

**After all workstreams:**
- Set `current_workstream=DONE`
- Write completion summary to `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md`
- Update progress log with final status
- Update `PROJECT_TODO.md` to mark Phase 2 complete
- Suggest next steps (Phase 2.2 or Phase 3)

---

## Sub-Prompts (Detailed) — Use within orchestrator flow

All sub-prompts: Always read relevant docs by path, write state, and log progress. Ask for missing inputs and pause if necessary.

---

### Prompt A1 — Project Setup

**Objective:**
Create Rust workspace structure for privacy guard service

**Inputs and references:**
- Read: ADR-0021 (implementation decisions)
- Read: Phase-2-Execution-Plan.md (Task A1 section)
- User inputs: guard_port (8089)

**Tasks:**
1. Create directory: `src/privacy-guard/`
2. Create `src/privacy-guard/Cargo.toml` with dependencies:
   ```toml
   [package]
   name = "privacy-guard"
   version = "0.1.0"
   edition = "2021"
   
   [dependencies]
   axum = "0.7"
   tokio = { version = "1", features = ["full"] }
   regex = "1"
   hmac = "0.12"
   sha2 = "0.10"
   fpe = "0.6"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   serde_yaml = "0.9"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   dashmap = "5"
   base64 = "0.21"
   
   [dev-dependencies]
   reqwest = { version = "0.12", features = ["json"] }
   ```
3. Update root `Cargo.toml` to add workspace member: `"src/privacy-guard"`
4. Create module files:
   - `src/privacy-guard/src/main.rs` (minimal "Hello" server)
   - `src/privacy-guard/src/detection.rs` (empty, module declaration)
   - `src/privacy-guard/src/pseudonym.rs` (empty)
   - `src/privacy-guard/src/redaction.rs` (empty)
   - `src/privacy-guard/src/policy.rs` (empty)
   - `src/privacy-guard/src/state.rs` (empty)
   - `src/privacy-guard/src/audit.rs` (empty)
5. Add module declarations to main.rs:
   ```rust
   mod detection;
   mod pseudonym;
   mod redaction;
   mod policy;
   mod state;
   mod audit;
   ```
6. Run `cargo check` from `src/privacy-guard/` to verify compilation
7. Commit on branch `feat/phase2-guard-core` with message:
   ```
   feat(guard): initialize privacy-guard Rust workspace
   
   - Create module structure (detection, pseudonym, redaction, policy, state, audit)
   - Add dependencies (axum, regex, hmac, fpe, serde, etc.)
   - Add to root workspace
   - Verify cargo check passes
   
   Refs: ADR-0021
   ```

**Acceptance:**
- `cargo check` passes without errors
- All module files created
- Root workspace includes privacy-guard

**Output artifacts:**
- `src/privacy-guard/Cargo.toml`
- `src/privacy-guard/src/*.rs` (8 files)
- Root `Cargo.toml` updated

**Logging:**
- Append to `docs/tests/phase2-progress.md`:
  ```
  [2025-11-03 HH:MM] A1: Project setup complete
  - Branch: feat/phase2-guard-core
  - Commit: <hash>
  - Workspace compiles successfully
  ```
- Update state JSON: `current_task_id=A1`, `checklist.A1=done`

---

### Prompt A2 — Detection Engine

**Objective:**
Implement regex-based PII detection with 8 entity types and confidence scoring

**Inputs and references:**
- Read: ADR-0022 (detection rules, entity types)
- Read: Phase-2-Execution-Plan.md (Task A2 section)
- Rules will be loaded from YAML (created in B1), use hardcoded fallback for now

**Tasks:**
1. Define entity type enum in `detection.rs`:
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub enum EntityType {
       SSN, EMAIL, PHONE, CREDIT_CARD, PERSON,
       IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER
   }
   
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub enum Confidence { HIGH, MEDIUM, LOW }
   
   #[derive(Debug, Clone)]
   pub struct Detection {
       pub start: usize,
       pub end: usize,
       pub entity_type: EntityType,
       pub confidence: Confidence,
       pub matched_text: String,
   }
   ```

2. Implement rules loader skeleton (will load from YAML later):
   ```rust
   pub struct Rules {
       patterns: HashMap<EntityType, Vec<Pattern>>,
   }
   
   pub struct Pattern {
       regex: Regex,
       confidence: Confidence,
       context_keywords: Option<Vec<String>>,
   }
   
   impl Rules {
       pub fn load_from_yaml(path: &Path) -> Result<Self> { ... }
       pub fn default_rules() -> Self { ... }  // Hardcoded baseline
   }
   ```

3. Implement detection function:
   ```rust
   pub fn detect(text: &str, rules: &Rules) -> Vec<Detection> {
       let mut detections = Vec::new();
       for (entity_type, patterns) in &rules.patterns {
           for pattern in patterns {
               for mat in pattern.regex.find_iter(text) {
                   detections.push(Detection {
                       start: mat.start(),
                       end: mat.end(),
                       entity_type: entity_type.clone(),
                       confidence: pattern.confidence.clone(),
                       matched_text: mat.as_str().to_string(),
                   });
               }
           }
       }
       detections.sort_by_key(|d| d.start);
       detections
   }
   ```

4. Add hardcoded baseline patterns in `default_rules()`:
   - SSN: `r"\b\d{3}-\d{2}-\d{4}\b"` (HIGH)
   - EMAIL: `r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"` (HIGH)
   - PHONE: `r"\b\d{3}-\d{3}-\d{4}\b"` (HIGH)
   - CREDIT_CARD: `r"\b\d{13,19}\b"` (MEDIUM, add Luhn check)
   - PERSON: `r"\b(?:Mr\.|Mrs\.|Ms\.|Dr\.)\s+[A-Z][a-z]+\s+[A-Z][a-z]+\b"` (MEDIUM)
   - IP_ADDRESS: `r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b"` (HIGH)
   - DATE_OF_BIRTH: `r"\b\d{1,2}/\d{1,2}/\d{2,4}\b"` (LOW)
   - ACCOUNT_NUMBER: `r"\b\d{8,16}\b"` (LOW)

5. Implement Luhn check for credit cards:
   ```rust
   fn is_luhn_valid(number: &str) -> bool { ... }
   ```

6. Write unit tests in `src/privacy-guard/src/detection.rs`:
   - Test each entity type with 5-10 samples
   - Test clean text (no detections)
   - Test overlapping patterns
   - Test confidence levels

7. Run tests: `cargo test` from `src/privacy-guard/`

8. Commit:
   ```
   feat(guard): implement regex-based PII detection engine
   
   - Define EntityType enum (8 types)
   - Implement Rules struct with pattern loading
   - Add hardcoded baseline patterns
   - Implement detect() function
   - Add Luhn validation for credit cards
   - Add unit tests (50+ test cases)
   
   Refs: ADR-0022
   ```

**Acceptance:**
- All unit tests pass
- Each entity type detects correctly on samples
- No false positives on clean text
- Confidence levels work

**Output artifacts:**
- `src/privacy-guard/src/detection.rs` (~300 lines)
- Unit tests

**Logging:**
- Append to progress log with commit hash
- Update state JSON: `current_task_id=A2`, `checklist.A2=done`

---

### Prompt A3 — Pseudonymization

**Objective:**
Implement HMAC-SHA256 deterministic mapping with in-memory state

**Inputs and references:**
- Read: ADR-0009 (deterministic pseudonymization)
- Read: ADR-0020 (Vault wiring, PSEUDO_SALT)
- Read: Phase-2-Execution-Plan.md (Task A3)
- User input: PSEUDO_SALT from environment (set in compose)

**Tasks:**
1. Implement HMAC function in `pseudonym.rs`:
   ```rust
   use hmac::{Hmac, Mac};
   use sha2::Sha256;
   type HmacSha256 = Hmac<Sha256>;
   
   pub fn pseudonymize(
       text: &str,
       entity_type: &EntityType,
       tenant_id: &str,
       salt: &str
   ) -> String {
       let input = format!("{tenant_id}||{entity_type:?}||{text}");
       let mut mac = HmacSha256::new_from_slice(salt.as_bytes())
           .expect("HMAC can take key of any size");
       mac.update(input.as_bytes());
       let result = mac.finalize();
       let hash = base64::encode_config(result.into_bytes(), base64::URL_SAFE_NO_PAD);
       format!("{}_{}", entity_type_to_prefix(entity_type), &hash[..8])
   }
   
   fn entity_type_to_prefix(et: &EntityType) -> &str {
       match et {
           EntityType::SSN => "SSN",
           EntityType::EMAIL => "EMAIL",
           EntityType::PERSON => "PERSON",
           // ... etc
       }
   }
   ```

2. Implement in-memory mapping store in `state.rs`:
   ```rust
   use dashmap::DashMap;
   
   pub struct MappingState {
       forward: DashMap<String, String>,   // original -> pseudonym
       reverse: DashMap<String, String>,   // pseudonym -> original
       tenant_id: String,
   }
   
   impl MappingState {
       pub fn new(tenant_id: String) -> Self { ... }
       
       pub fn store(&self, original: String, pseudonym: String) {
           self.forward.insert(original.clone(), pseudonym.clone());
           self.reverse.insert(pseudonym, original);
       }
       
       pub fn lookup_forward(&self, original: &str) -> Option<String> { ... }
       pub fn lookup_reverse(&self, pseudonym: &str) -> Option<String> { ... }
       
       pub fn flush(&self) {
           self.forward.clear();
           self.reverse.clear();
       }
   }
   ```

3. Integrate pseudonymize with state:
   ```rust
   pub fn get_or_create_pseudonym(
       text: &str,
       entity_type: &EntityType,
       state: &MappingState,
       salt: &str
   ) -> String {
       if let Some(existing) = state.lookup_forward(text) {
           return existing;
       }
       let pseudonym = pseudonymize(text, entity_type, &state.tenant_id, salt);
       state.store(text.to_string(), pseudonym.clone());
       pseudonym
   }
   ```

4. Write unit tests:
   - Test determinism: same input produces same pseudonym
   - Test uniqueness: different inputs produce different pseudonyms
   - Test state storage: forward and reverse lookups work
   - Test thread safety: concurrent calls to same state

5. Run tests: `cargo test`

6. Commit:
   ```
   feat(guard): implement HMAC-SHA256 pseudonymization
   
   - Add HMAC function with tenant_id + entity_type + text
   - Implement in-memory mapping state (DashMap)
   - Add forward/reverse lookup
   - Add determinism and uniqueness tests
   
   Refs: ADR-0009
   ```

**Acceptance:**
- Determinism test passes (same input → same output)
- Uniqueness test passes (different inputs → different outputs)
- Reverse lookup works
- Thread-safe (DashMap tests)

**Output artifacts:**
- `src/privacy-guard/src/pseudonym.rs` (~150 lines)
- `src/privacy-guard/src/state.rs` (~100 lines)
- Unit tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A3`, `checklist.A3=done`

---

### Prompt A4 — Format-Preserving Encryption

**Objective:**
Implement FPE for phone and SSN using AES-FFX

**Inputs and references:**
- Read: ADR-0022 (FPE section)
- Read: Phase-2-Execution-Plan.md (Task A4)
- Crate: `fpe` for FF3-1 implementation

**Tasks:**
1. Add FPE logic in `redaction.rs`:
   ```rust
   use fpe::ff1::{FF1, BinaryNumeralString};
   
   pub fn fpe_phone(
       phone: &str,
       key: &[u8],
       preserve_area_code: bool
   ) -> Result<String> {
       // Parse phone: 555-123-4567 or (555) 123-4567
       let digits: String = phone.chars().filter(|c| c.is_digit(10)).collect();
       if digits.len() != 10 { return Err("Invalid phone format"); }
       
       let (area_code, rest) = digits.split_at(3);
       let rest_encrypted = if preserve_area_code {
           fpe_encrypt_digits(rest, key)?
       } else {
           fpe_encrypt_digits(&digits, key)?
       };
       
       // Reconstruct format
       if phone.contains('(') {
           Ok(format!("({}) {}-{}", area_code, &rest_encrypted[0..3], &rest_encrypted[3..7]))
       } else {
           Ok(format!("{}-{}-{}", area_code, &rest_encrypted[0..3], &rest_encrypted[3..7]))
       }
   }
   
   pub fn fpe_ssn(
       ssn: &str,
       key: &[u8],
       preserve_last_n: usize
   ) -> Result<String> {
       let digits: String = ssn.chars().filter(|c| c.is_digit(10)).collect();
       if digits.len() != 9 { return Err("Invalid SSN format"); }
       
       let split_at = 9 - preserve_last_n;
       let (prefix, suffix) = digits.split_at(split_at);
       let prefix_encrypted = fpe_encrypt_digits(prefix, key)?;
       
       // Reconstruct: xxx-xx-xxxx
       if ssn.contains('-') {
           Ok(format!("{}-{}-{}", &prefix_encrypted[0..3], &prefix_encrypted[3..5], suffix))
       } else {
           Ok(format!("{}{}", prefix_encrypted, suffix))
       }
   }
   
   fn fpe_encrypt_digits(digits: &str, key: &[u8]) -> Result<String> {
       let ff1 = FF1::<aes::Aes256>::new(key, 10)?; // Radix 10
       let plaintext = BinaryNumeralString::from_bytes_le(digits.as_bytes());
       let ciphertext = ff1.encrypt(&[], &plaintext)?;
       Ok(String::from_utf8(ciphertext.to_bytes_le())?)
   }
   ```

2. Add configuration for FPE options:
   ```rust
   pub struct FPEConfig {
       pub preserve_phone_area_code: bool,
       pub preserve_ssn_last_n: usize,
   }
   
   impl Default for FPEConfig {
       fn default() -> Self {
           Self {
               preserve_phone_area_code: true,
               preserve_ssn_last_n: 4,
           }
       }
   }
   ```

3. Write unit tests:
   - Test phone FPE preserves format: `555-123-4567` → `555-XXX-XXXX`
   - Test SSN FPE preserves last 4: `123-45-6789` → `XXX-XX-6789`
   - Test determinism: same input produces same output
   - Test various input formats

4. Run tests: `cargo test`

5. Commit:
   ```
   feat(guard): implement format-preserving encryption for phone and SSN
   
   - Use fpe crate (AES-FFX FF3-1)
   - Support phone FPE with optional area code preservation
   - Support SSN FPE with last N digit preservation
   - Add format reconstruction logic
   - Add unit tests for format preservation and determinism
   
   Refs: ADR-0022
   ```

**Acceptance:**
- Phone FPE preserves format
- SSN FPE preserves last 4 digits
- Deterministic (same input → same output)
- Tests pass

**Output artifacts:**
- `src/privacy-guard/src/redaction.rs` (FPE module, ~200 lines)
- Unit tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A4`, `checklist.A4=done`

---

### Prompt A5 — Masking Logic

**Objective:**
Implement text replacement using pseudonyms or FPE based on policy

**Inputs and references:**
- Read: ADR-0022 (masking strategies)
- Read: Phase-2-Execution-Plan.md (Task A5)
- Depends on: A2 (detection), A3 (pseudonym), A4 (FPE)

**Tasks:**
1. Implement mask function in `redaction.rs`:
   ```rust
   pub fn mask(
       text: &str,
       detections: Vec<Detection>,
       policy: &Policy,
       state: &MappingState,
       salt: &str,
       fpe_key: &[u8]
   ) -> MaskResult {
       let mut masked_text = text.to_string();
       let mut redactions_count: HashMap<EntityType, usize> = HashMap::new();
       
       // Sort by start position descending (to avoid offset issues)
       let mut sorted_detections = detections;
       sorted_detections.sort_by(|a, b| b.start.cmp(&a.start));
       
       for detection in sorted_detections {
           let strategy = policy.get_strategy(&detection.entity_type);
           let replacement = match strategy {
               Strategy::PSEUDONYM => {
                   get_or_create_pseudonym(
                       &detection.matched_text,
                       &detection.entity_type,
                       state,
                       salt
                   )
               },
               Strategy::FPE => {
                   match detection.entity_type {
                       EntityType::PHONE => fpe_phone(&detection.matched_text, fpe_key, true)?,
                       EntityType::SSN => fpe_ssn(&detection.matched_text, fpe_key, 4)?,
                       _ => get_or_create_pseudonym(...), // Fallback
                   }
               },
               Strategy::REDACT => {
                   format!("[REDACTED_{}]", detection.entity_type)
               }
           };
           
           // Replace in text
           masked_text.replace_range(detection.start..detection.end, &replacement);
           
           // Count redaction
           *redactions_count.entry(detection.entity_type.clone()).or_insert(0) += 1;
       }
       
       MaskResult {
           masked_text,
           redactions_count,
           total_redactions: detections.len(),
       }
   }
   
   pub struct MaskResult {
       pub masked_text: String,
       pub redactions_count: HashMap<EntityType, usize>,
       pub total_redactions: usize,
   }
   ```

2. Handle overlapping detections (priority by confidence):
   ```rust
   fn resolve_overlaps(detections: Vec<Detection>) -> Vec<Detection> {
       // Keep higher confidence if overlapping
       // ...
   }
   ```

3. Write integration tests:
   - Test text with multiple PII types
   - Test overlapping entities
   - Test redaction counts
   - Test format preservation

4. Run tests: `cargo test`

5. Commit:
   ```
   feat(guard): implement masking logic with strategy routing
   
   - Route to pseudonym, FPE, or redact based on policy
   - Handle overlapping detections (priority by confidence)
   - Generate redaction summary counts
   - Preserve text structure
   - Add integration tests
   
   Refs: ADR-0022
   ```

**Acceptance:**
- Multi-PII text correctly masked
- Overlapping handled
- Redaction counts accurate
- Tests pass

**Output artifacts:**
- `src/privacy-guard/src/redaction.rs` (updated, ~400 lines)
- Integration tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A5`, `checklist.A5=done`

---

### Prompt A6 — Policy Engine

**Objective:**
Load and apply policy configuration (modes, strategies, thresholds)

**Inputs and references:**
- Read: ADR-0022 (policy schema)
- Read: Phase-2-Execution-Plan.md (Task A6)
- YAML will be created in B2, use defaults for now

**Tasks:**
1. Define policy structs in `policy.rs`:
   ```rust
   #[derive(Debug, Clone)]
   pub struct Policy {
       pub mode: Mode,
       pub confidence_threshold: Confidence,
       pub strategies: HashMap<EntityType, Strategy>,
       pub fpe_config: FPEConfig,
   }
   
   #[derive(Debug, Clone)]
   pub enum Mode { OFF, DETECT, MASK, STRICT }
   
   #[derive(Debug, Clone)]
   pub enum Strategy { PSEUDONYM, FPE, REDACT }
   
   impl Policy {
       pub fn load_from_yaml(path: &Path) -> Result<Self> { ... }
       
       pub fn default() -> Self {
           let mut strategies = HashMap::new();
           strategies.insert(EntityType::SSN, Strategy::FPE);
           strategies.insert(EntityType::PHONE, Strategy::FPE);
           strategies.insert(EntityType::EMAIL, Strategy::PSEUDONYM);
           strategies.insert(EntityType::PERSON, Strategy::PSEUDONYM);
           strategies.insert(EntityType::CREDIT_CARD, Strategy::REDACT);
           
           Self {
               mode: Mode::MASK,
               confidence_threshold: Confidence::MEDIUM,
               strategies,
               fpe_config: FPEConfig::default(),
           }
       }
       
       pub fn get_strategy(&self, entity_type: &EntityType) -> Strategy {
           self.strategies.get(entity_type)
               .cloned()
               .unwrap_or(Strategy::PSEUDONYM)
       }
   }
   ```

2. Implement mode logic:
   ```rust
   pub fn apply_policy(
       text: &str,
       policy: &Policy,
       rules: &Rules,
       state: &MappingState,
       salt: &str,
       fpe_key: &[u8]
   ) -> Result<ProcessResult> {
       let detections = detect(text, rules);
       
       // Filter by confidence threshold
       let filtered: Vec<_> = detections.into_iter()
           .filter(|d| d.confidence >= policy.confidence_threshold)
           .collect();
       
       match policy.mode {
           Mode::OFF => Ok(ProcessResult::PassThrough(text.to_string())),
           Mode::DETECT => Ok(ProcessResult::Detections(filtered)),
           Mode::MASK => {
               let masked = mask(text, filtered, policy, state, salt, fpe_key)?;
               Ok(ProcessResult::Masked(masked))
           },
           Mode::STRICT => {
               if !filtered.is_empty() {
                   Err("PII detected in STRICT mode".into())
               } else {
                   Ok(ProcessResult::PassThrough(text.to_string()))
               }
           }
       }
   }
   
   pub enum ProcessResult {
       PassThrough(String),
       Detections(Vec<Detection>),
       Masked(MaskResult),
   }
   ```

3. Write unit tests for each mode:
   - OFF: no processing
   - DETECT: detections returned, no masking
   - MASK: full masking applied
   - STRICT: error on detection

4. Run tests: `cargo test`

5. Commit:
   ```
   feat(guard): implement policy engine with modes and strategies
   
   - Define Policy struct with mode, strategies, thresholds
   - Implement mode logic (OFF, DETECT, MASK, STRICT)
   - Add per-entity-type strategy selection
   - Add confidence filtering
   - Add unit tests for each mode
   
   Refs: ADR-0022
   ```

**Acceptance:**
- Each mode works correctly
- Strategy selection per entity type
- Confidence filtering
- Tests pass

**Output artifacts:**
- `src/privacy-guard/src/policy.rs` (~250 lines)
- Unit tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A6`, `checklist.A6=done`

---

### Prompt A7 — HTTP API

**Objective:**
Implement REST endpoints with Axum for scan, mask, reidentify

**Inputs and references:**
- Read: ADR-0021 (API design)
- Read: Phase-2-Execution-Plan.md (Task A7, API examples)
- Depends on: all previous A tasks

**Tasks:**
1. Set up Axum server in `main.rs`:
   ```rust
   use axum::{Router, routing::{get, post}, extract::State, Json};
   use std::sync::Arc;
   
   #[tokio::main]
   async fn main() {
       tracing_subscriber::fmt::init();
       
       let salt = std::env::var("PSEUDO_SALT").unwrap_or_else(|_| {
           tracing::warn!("PSEUDO_SALT not set, using OFF mode");
           String::new()
       });
       
       let app_state = Arc::new(AppState {
           rules: Rules::default(),
           policy: Policy::default(),
           salt,
       });
       
       let app = Router::new()
           .route("/status", get(status_handler))
           .route("/guard/scan", post(scan_handler))
           .route("/guard/mask", post(mask_handler))
           .route("/guard/reidentify", post(reidentify_handler))
           .route("/internal/flush-session", post(flush_session_handler))
           .with_state(app_state);
       
       let addr = format!("0.0.0.0:{}", std::env::var("GUARD_PORT").unwrap_or("8089".into()));
       tracing::info!("Privacy Guard listening on {}", addr);
       axum::Server::bind(&addr.parse().unwrap())
           .serve(app.into_make_service())
           .await
           .unwrap();
   }
   
   struct AppState {
       rules: Rules,
       policy: Policy,
       salt: String,
   }
   ```

2. Implement endpoints:
   ```rust
   async fn status_handler(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
       Json(StatusResponse {
           status: "healthy",
           mode: state.policy.mode,
           rule_count: state.rules.count(),
           config_loaded: true,
       })
   }
   
   async fn scan_handler(
       State(state): State<Arc<AppState>>,
       Json(req): Json<ScanRequest>
   ) -> Result<Json<ScanResponse>, AppError> {
       let detections = detect(&req.text, &state.rules);
       Ok(Json(ScanResponse { detections }))
   }
   
   async fn mask_handler(
       State(state): State<Arc<AppState>>,
       Json(req): Json<MaskRequest>
   ) -> Result<Json<MaskResponse>, AppError> {
       // Create or get session state
       let session_state = get_or_create_session(&req.tenant_id, &req.session_id);
       
       let result = apply_policy(
           &req.text,
           &state.policy,
           &state.rules,
           &session_state,
           &state.salt,
           &derive_fpe_key(&state.salt)
       )?;
       
       match result {
           ProcessResult::Masked(masked) => {
               Ok(Json(MaskResponse {
                   masked_text: masked.masked_text,
                   redactions: masked.redactions_count,
                   session_id: req.session_id,
               }))
           },
           _ => Err(AppError::InvalidMode)
       }
   }
   
   async fn reidentify_handler(
       headers: HeaderMap,
       State(state): State<Arc<AppState>>,
       Json(req): Json<ReidentifyRequest>
   ) -> Result<Json<ReidentifyResponse>, AppError> {
       // Validate JWT from Authorization header
       if !validate_jwt(&headers) {
           return Err(AppError::Unauthorized);
       }
       
       let session_state = get_session(&req.session_id)?;
       let original = session_state.lookup_reverse(&req.pseudonym)
           .ok_or(AppError::NotFound)?;
       
       Ok(Json(ReidentifyResponse { original }))
   }
   ```

3. Define request/response schemas:
   ```rust
   #[derive(Deserialize)]
   struct ScanRequest {
       text: String,
       tenant_id: String,
   }
   
   #[derive(Serialize)]
   struct ScanResponse {
       detections: Vec<Detection>,
   }
   
   #[derive(Deserialize)]
   struct MaskRequest {
       text: String,
       tenant_id: String,
       session_id: Option<String>,
       mode: Option<Mode>,
   }
   
   #[derive(Serialize)]
   struct MaskResponse {
       masked_text: String,
       redactions: HashMap<EntityType, usize>,
       session_id: String,
   }
   
   #[derive(Deserialize)]
   struct ReidentifyRequest {
       pseudonym: String,
       session_id: String,
   }
   
   #[derive(Serialize)]
   struct ReidentifyResponse {
       original: String,
   }
   ```

4. Add error handling:
   ```rust
   #[derive(Debug)]
   enum AppError {
       InvalidInput(String),
       Unauthorized,
       NotFound,
       InvalidMode,
       Internal(String),
   }
   
   impl IntoResponse for AppError {
       fn into_response(self) -> Response {
           let (status, message) = match self {
               AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
               AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
               AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".into()),
               AppError::InvalidMode => (StatusCode::BAD_REQUEST, "Invalid mode".into()),
               AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
           };
           (status, Json(json!({"error": message}))).into_response()
       }
   }
   ```

5. Add request logging (no PII):
   ```rust
   tracing::info!(
       tenant_id = %req.tenant_id,
       text_length = req.text.len(),
       "Received mask request"
   );
   ```

6. Write integration tests:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[tokio::test]
       async fn test_scan_endpoint() {
           let client = reqwest::Client::new();
           let res = client.post("http://localhost:8089/guard/scan")
               .json(&json!({
                   "text": "Contact John at john@example.com",
                   "tenant_id": "org1"
               }))
               .send()
               .await
               .unwrap();
           
           assert_eq!(res.status(), 200);
           let body: ScanResponse = res.json().await.unwrap();
           assert!(body.detections.iter().any(|d| d.entity_type == EntityType::EMAIL));
       }
       
       // ... more tests
   }
   ```

7. Run tests: `cargo test`

8. Commit:
   ```
   feat(guard): implement HTTP API with Axum
   
   - Add endpoints: /status, /guard/scan, /guard/mask, /guard/reidentify
   - Add request/response schemas
   - Add error handling (400, 401, 404, 500)
   - Add request logging (no PII, metadata only)
   - Add integration tests
   
   Refs: ADR-0021
   ```

**Acceptance:**
- All endpoints return 200 for valid requests
- Error responses correct
- JWT auth works on reidentify
- Integration tests pass
- Logs show metadata only (no PII)

**Output artifacts:**
- `src/privacy-guard/src/main.rs` (~400 lines)
- Integration tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A7`, `checklist.A7=done`

---

### Prompt A8 — Audit Logging

**Objective:**
Implement redaction event logging (counts only, no PII)

**Inputs and references:**
- Read: ADR-0008 (audit schema)
- Read: Phase-2-Execution-Plan.md (Task A8)

**Tasks:**
1. Define audit event schema in `audit.rs`:
   ```rust
   #[derive(Serialize)]
   pub struct RedactionEvent {
       pub timestamp: String,
       pub tenant_id: String,
       pub session_id: Option<String>,
       pub mode: String,
       pub entity_counts: HashMap<String, usize>,
       pub total_redactions: usize,
       pub performance_ms: u64,
       pub trace_id: Option<String>,
   }
   
   pub fn log_redaction_event(
       tenant_id: &str,
       session_id: Option<&str>,
       mode: &Mode,
       redactions: &HashMap<EntityType, usize>,
       duration_ms: u64
   ) {
       let entity_counts: HashMap<String, usize> = redactions.iter()
           .map(|(k, v)| (format!("{:?}", k), *v))
           .collect();
       
       let total = redactions.values().sum();
       
       let event = RedactionEvent {
           timestamp: chrono::Utc::now().to_rfc3339(),
           tenant_id: tenant_id.to_string(),
           session_id: session_id.map(|s| s.to_string()),
           mode: format!("{:?}", mode),
           entity_counts,
           total_redactions: total,
           performance_ms: duration_ms,
           trace_id: None, // TODO: extract from headers
       };
       
       tracing::info!(
           target: "audit",
           event = serde_json::to_string(&event).unwrap(),
           "Redaction event"
       );
   }
   ```

2. Integrate into mask_handler:
   ```rust
   async fn mask_handler(...) -> Result<...> {
       let start = std::time::Instant::now();
       
       // ... masking logic ...
       
       let duration_ms = start.elapsed().as_millis() as u64;
       log_redaction_event(
           &req.tenant_id,
           req.session_id.as_deref(),
           &state.policy.mode,
           &masked.redactions_count,
           duration_ms
       );
       
       Ok(Json(response))
   }
   ```

3. Verify logs never contain PII:
   - Audit code review
   - Add test that checks log output for known PII

4. Commit:
   ```
   feat(guard): implement audit logging for redaction events
   
   - Define RedactionEvent schema (counts only, no PII)
   - Emit structured JSON logs
   - Include performance metrics
   - Add trace ID propagation (placeholder)
   - Verify no PII in logs
   
   Refs: ADR-0008
   ```

**Acceptance:**
- Logs contain counts but no PII
- Structured JSON format
- Performance metrics included

**Output artifacts:**
- `src/privacy-guard/src/audit.rs` (~80 lines)

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A8`, `checklist.A8=done`, `current_workstream=B`

---

### Prompt B1 — Rules YAML

**Objective:**
Create baseline PII detection rules for 8 entity types

**Inputs and references:**
- Read: ADR-0022 (rules schema)
- Read: Phase-2-Execution-Plan.md (Task B1, rules structure)

**Tasks:**
1. Create `deploy/compose/guard-config/rules.yaml`:
   ```yaml
   version: "1.0"
   metadata:
     author: "Phase 2 Team"
     date: "2025-11-03"
     description: "Baseline PII detection rules for Privacy Guard"
   
   entity_types:
     SSN:
       display_name: "Social Security Number"
       category: "GOVERNMENT_ID"
       patterns:
         - regex: '\b\d{3}-\d{2}-\d{4}\b'
           confidence: HIGH
           description: "US SSN with hyphens (xxx-xx-xxxx)"
         - regex: '\b\d{9}\b'
           confidence: MEDIUM
           description: "US SSN no separators (context-dependent)"
           context_keywords: ["SSN", "social security", "SS#"]
         - regex: '\b\d{3}\s\d{2}\s\d{4}\b'
           confidence: MEDIUM
           description: "US SSN with spaces"
     
     CREDIT_CARD:
       display_name: "Credit Card Number"
       category: "FINANCIAL"
       patterns:
         - regex: '\b4\d{15}\b'
           confidence: HIGH
           description: "Visa (16 digits starting with 4)"
           luhn_check: true
         - regex: '\b5[1-5]\d{14}\b'
           confidence: HIGH
           description: "Mastercard (16 digits starting with 51-55)"
           luhn_check: true
         - regex: '\b3[47]\d{13}\b'
           confidence: HIGH
           description: "Amex (15 digits starting with 34 or 37)"
           luhn_check: true
         - regex: '\b6(?:011|5\d{2})\d{12}\b'
           confidence: HIGH
           description: "Discover (16 digits starting with 6011 or 65)"
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
           description: "US phone with parens ((xxx) xxx-xxxx)"
         - regex: '\b\d{3}\.\d{3}\.\d{4}\b'
           confidence: HIGH
           description: "US phone with dots (xxx.xxx.xxxx)"
         - regex: '\+1\s?\d{3}\s?\d{3}\s?\d{4}'
           confidence: HIGH
           description: "US phone with country code (+1 xxx xxx xxxx)"
         - regex: '\+\d{1,3}\s?\d{4,14}'
           confidence: MEDIUM
           description: "International E.164 format"
     
     PERSON:
       display_name: "Person Name"
       category: "IDENTITY"
       patterns:
         - regex: '\b(?:Mr\.|Mrs\.|Ms\.|Dr\.|Prof\.)\s+[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+\b'
           confidence: MEDIUM
           description: "Name with title (Mr./Mrs./Ms./Dr./Prof.)"
         - regex: '\b[A-Z][a-z]+\s+[A-Z][a-z]+\b'
           confidence: LOW
           description: "Two capitalized words (prone to false positives)"
           context_keywords: ["name", "person", "employee", "contact", "from", "to", "by"]
         - regex: '\b(?:Name|Full name|Contact):\s*([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)'
           confidence: HIGH
           description: "Explicit name field"
     
     IP_ADDRESS:
       display_name: "IP Address"
       category: "NETWORK"
       patterns:
         - regex: '\b(?:\d{1,3}\.){3}\d{1,3}\b'
           confidence: HIGH
           description: "IPv4 address"
         - regex: '\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b'
           confidence: HIGH
           description: "IPv6 address (full)"
     
     DATE_OF_BIRTH:
       display_name: "Date of Birth"
       category: "IDENTITY"
       patterns:
         - regex: '\b(?:DOB|Date of birth|Born):\s*(\d{1,2}/\d{1,2}/\d{2,4})\b'
           confidence: HIGH
           description: "DOB with label (MM/DD/YYYY or variants)"
         - regex: '\b\d{1,2}/\d{1,2}/\d{2,4}\b'
           confidence: LOW
           description: "Generic date (many false positives)"
           context_keywords: ["birth", "DOB", "born", "age"]
     
     ACCOUNT_NUMBER:
       display_name: "Account Number"
       category: "FINANCIAL"
       patterns:
         - regex: '\b(?:Account|Acct|Account #):\s*(\d{8,16})\b'
           confidence: HIGH
           description: "Account number with label"
         - regex: '\b\d{8,16}\b'
           confidence: LOW
           description: "Generic 8-16 digit number"
           context_keywords: ["account", "acct", "number", "ID"]
   ```

2. Validate YAML syntax:
   ```bash
   python3 -c "import yaml; yaml.safe_load(open('deploy/compose/guard-config/rules.yaml'))"
   ```

3. Update detection.rs to load from YAML (if not already done):
   ```rust
   impl Rules {
       pub fn load_from_yaml(path: &Path) -> Result<Self> {
           let contents = std::fs::read_to_string(path)?;
           let yaml: serde_yaml::Value = serde_yaml::from_str(&contents)?;
           // Parse entity_types, build regex patterns
           // ...
       }
   }
   ```

4. Test each pattern manually with samples

5. Commit:
   ```
   feat(guard): add baseline PII detection rules (8 entity types)
   
   - Create rules.yaml with SSN, CREDIT_CARD, EMAIL, PHONE, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER
   - 3-5 patterns per type
   - Confidence levels and context keywords
   - Luhn check annotations for credit cards
   
   Refs: ADR-0022
   ```

**Acceptance:**
- YAML is valid
- Guard loads rules without errors
- Each pattern tested with 5+ samples
- Log shows rule count on startup

**Output artifacts:**
- `deploy/compose/guard-config/rules.yaml` (~150 lines)

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=B1`, `checklist.B1=done`

---

### Prompt B2 — Policy YAML

**Objective:**
Define masking policy defaults (modes, strategies, FPE settings)

**Inputs and references:**
- Read: ADR-0022 (policy schema)
- Read: Phase-2-Execution-Plan.md (Task B2)

**Tasks:**
1. Create `deploy/compose/guard-config/policy.yaml`:
   ```yaml
   version: "1.0"
   
   global:
     mode: MASK  # OFF | DETECT | MASK | STRICT
     confidence_threshold: MEDIUM  # Ignore LOW confidence unless STRICT
   
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
       
       IP_ADDRESS:
         strategy: PSEUDONYM
       
       DATE_OF_BIRTH:
         strategy: PSEUDONYM
       
       ACCOUNT_NUMBER:
         strategy: PSEUDONYM
   
   audit:
     log_detections: true
     log_redactions: true
     log_mapping_count: true  # Count only, not actual mappings
     log_performance: true
   
   # Tenant overrides (future)
   # tenants:
   #   org1:
   #     mode: STRICT
   ```

2. Validate YAML syntax:
   ```bash
   python3 -c "import yaml; yaml.safe_load(open('deploy/compose/guard-config/policy.yaml'))"
   ```

3. Update policy.rs to load from YAML (if not already done):
   ```rust
   impl Policy {
       pub fn load_from_yaml(path: &Path) -> Result<Self> {
           let contents = std::fs::read_to_string(path)?;
           let yaml: serde_yaml::Value = serde_yaml::from_str(&contents)?;
           // Parse global, masking, audit sections
           // ...
       }
   }
   ```

4. Test policy loading in unit test

5. Commit:
   ```
   feat(guard): add policy configuration defaults
   
   - Create policy.yaml with modes and strategies
   - Define per-type masking strategies (FPE for phone/SSN, pseudonym for others)
   - Set default mode to MASK
   - Configure audit settings
   
   Refs: ADR-0022
   ```

**Acceptance:**
- YAML is valid
- Guard loads policy without errors
- Settings applied correctly in tests

**Output artifacts:**
- `deploy/compose/guard-config/policy.yaml` (~50 lines)

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=B2`, `checklist.B2=done`

---

### Prompt B3 — Test Data

**Objective:**
Create synthetic PII samples for testing (no real PII)

**Inputs and references:**
- Read: Phase-2-Execution-Plan.md (Task B3)

**Tasks:**
1. Create `tests/fixtures/pii_samples.txt`:
   ```
   Contact John Doe at 555-123-4567 or john.doe@example.com
   SSN: 123-45-6789, DOB: 01/15/1985
   Credit card: 4532015112830366 (Visa)
   Employee Jane Smith can be reached at (555) 987-6543
   Account #: 1234567890123456
   IP address: 192.168.1.100
   Dr. Alice Johnson's office: alice.johnson@hospital.example
   Please contact Mr. Robert Brown at +1 555 234 5678
   SSN 987654321 (no hyphens)
   Amex: 378282246310005
   [... 90+ more lines with various formats and combinations]
   ```

2. Create `tests/fixtures/clean_samples.txt`:
   ```
   The quick brown fox jumps over the lazy dog.
   Meeting scheduled for tomorrow at 3pm in conference room B.
   Please review the attached document and provide feedback.
   Budget allocation for Q4 is pending approval.
   [... 40+ more lines with no PII]
   ```

3. Create `tests/fixtures/expected_detections.json`:
   ```json
   {
     "samples": [
       {
         "text": "Contact John Doe at 555-123-4567 or john.doe@example.com",
         "expected": [
           {"type": "PERSON", "text": "John Doe"},
           {"type": "PHONE", "text": "555-123-4567"},
           {"type": "EMAIL", "text": "john.doe@example.com"}
         ]
       },
       // ... more samples with expectations
     ]
   }
   ```

4. Write integration test that validates against fixtures:
   ```rust
   #[test]
   fn test_detection_against_fixtures() {
       let samples = load_pii_samples();
       let expected = load_expected_detections();
       
       for (sample, expected) in samples.iter().zip(expected.iter()) {
           let detections = detect(&sample.text, &Rules::default());
           assert_eq!(detections.len(), expected.len());
           // ... validate each detection
       }
   }
   
   #[test]
   fn test_clean_samples_no_detections() {
       let clean = load_clean_samples();
       for sample in clean {
           let detections = detect(&sample, &Rules::default());
           assert_eq!(detections.len(), 0, "Found PII in clean sample: {}", sample);
       }
   }
   ```

5. Run tests: `cargo test`

6. Commit:
   ```
   test(guard): add synthetic PII test fixtures
   
   - Create pii_samples.txt (100+ lines with known PII)
   - Create clean_samples.txt (50+ lines with no PII)
   - Create expected_detections.json
   - Add integration tests against fixtures
   - Validate false positive rate < 5%
   
   Refs: ADR-0022
   ```

**Acceptance:**
- Guard detects all PII in pii_samples.txt
- Guard detects zero PII in clean_samples.txt
- False positive rate < 5%

**Output artifacts:**
- `tests/fixtures/pii_samples.txt` (~100 lines)
- `tests/fixtures/clean_samples.txt` (~50 lines)
- `tests/fixtures/expected_detections.json`
- Integration tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=B3`, `checklist.B3=done`, `current_workstream=C`

---

### Prompt C1 — Dockerfile

**Objective:**
Create multi-stage Docker build for privacy guard

**Inputs and references:**
- Read: ADR-0021 (deployment model)
- Read: Phase-2-Execution-Plan.md (Task C1, Dockerfile example)
- Reference: `src/controller/Dockerfile` (similar pattern)

**Tasks:**
1. Create `src/privacy-guard/Dockerfile`:
   ```dockerfile
   # Build stage
   FROM rust:1.83-bookworm AS builder
   
   WORKDIR /build
   
   # Cache dependencies
   COPY Cargo.toml Cargo.lock ./
   RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
   
   # Build application
   COPY src ./src
   RUN cargo build --release
   
   # Runtime stage
   FROM debian:bookworm-slim
   
   RUN apt-get update && \
       apt-get install -y ca-certificates curl && \
       rm -rf /var/lib/apt/lists/*
   
   COPY --from=builder /build/target/release/privacy-guard /usr/local/bin/privacy-guard
   
   # Config will be mounted at runtime
   RUN mkdir -p /etc/guard-config
   
   EXPOSE 8089
   
   # Run as non-root
   RUN useradd -r -s /bin/false guarduser
   USER guarduser
   
   HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
     CMD curl -f http://localhost:8089/status || exit 1
   
   CMD ["privacy-guard"]
   ```

2. Create `.dockerignore` in `src/privacy-guard/`:
   ```
   target/
   Cargo.lock
   .git/
   ```

3. Test build:
   ```bash
   cd src/privacy-guard
   docker build -t privacy-guard:dev .
   docker run --rm privacy-guard:dev privacy-guard --version
   ```

4. Check image size:
   ```bash
   docker images privacy-guard:dev --format "{{.Size}}"
   # Should be < 100MB
   ```

5. Commit:
   ```
   build(guard): add Dockerfile with multi-stage build
   
   - Use Rust 1.83 builder
   - Debian slim runtime
   - Non-root user
   - Healthcheck CMD
   - Optimized layer caching
   - Image size < 100MB
   
   Refs: ADR-0021
   ```

**Acceptance:**
- Docker build succeeds
- Image size < 100MB
- Container starts and responds to health check

**Output artifacts:**
- `src/privacy-guard/Dockerfile`
- `src/privacy-guard/.dockerignore`

**Logging:**
- Append to progress log with image size
- Update state JSON: `current_task_id=C1`, `checklist.C1=done`

---

### Prompt C2 — Compose Service

**Objective:**
Add privacy-guard service to Docker Compose

**Inputs and references:**
- Read: Phase-2-Execution-Plan.md (Task C2, compose example)
- Reference: `deploy/compose/ce.dev.yml` (existing services)

**Tasks:**
1. Update `deploy/compose/ce.dev.yml`:
   ```yaml
   services:
     # ... existing services ...
     
     privacy-guard:
       build:
         context: ../../src/privacy-guard
       environment:
         - PSEUDO_SALT=${PSEUDO_SALT}
         - GUARD_PORT=8089
         - GUARD_MODE=${GUARD_MODE:-MASK}
         - RUST_LOG=${GUARD_LOG_LEVEL:-info}
         - GUARD_CONFIG_PATH=/etc/guard-config
       ports:
         - "8089:8089"
       volumes:
         - ./guard-config:/etc/guard-config:ro
       healthcheck:
         test: ["CMD", "curl", "-f", "http://localhost:8089/status"]
         interval: 10s
         timeout: 3s
         retries: 3
         start_period: 5s
       depends_on:
         vault:
           condition: service_healthy
       networks:
         - goose-net
       profiles:
         - privacy-guard
   ```

2. Update `.env.ce.example` if needed (PSEUDO_SALT should already exist from Phase 1.2):
   ```bash
   # Privacy Guard (added in Phase 2)
   GUARD_MODE=MASK
   GUARD_LOG_LEVEL=info
   ```

3. Test compose startup:
   ```bash
   cd deploy/compose
   docker compose --profile privacy-guard up -d
   docker compose ps
   docker compose logs privacy-guard
   ```

4. Verify healthcheck:
   ```bash
   docker compose exec privacy-guard curl http://localhost:8089/status
   ```

5. Commit:
   ```
   build(guard): integrate privacy-guard into docker compose
   
   - Add privacy-guard service to ce.dev.yml
   - Configure environment variables
   - Mount config volume
   - Add healthcheck
   - Add profile: privacy-guard
   - Depends on vault
   
   Refs: ADR-0021
   ```

**Acceptance:**
- `docker compose --profile privacy-guard up` starts all services
- Healthcheck passes
- Guard reachable at http://localhost:8089/status

**Output artifacts:**
- `deploy/compose/ce.dev.yml` (updated)
- `deploy/compose/.env.ce.example` (updated if needed)

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=C2`, `checklist.C2=done`

---

### Prompt C3 — Healthcheck Script

**Objective:**
Create healthcheck script for guard service

**Inputs and references:**
- Read: Phase-2-Execution-Plan.md (Task C3)
- Reference: `deploy/compose/healthchecks/` (existing scripts)

**Tasks:**
1. Create `deploy/compose/healthchecks/guard_health.sh`:
   ```bash
   #!/bin/bash
   set -e
   
   GUARD_URL="${GUARD_URL:-http://localhost:8089}"
   
   # Check status endpoint
   response=$(curl -sf "${GUARD_URL}/status" || exit 1)
   
   # Verify response contains expected fields
   if echo "$response" | grep -q '"status"'; then
       echo "Privacy Guard is healthy"
       exit 0
   else
       echo "Privacy Guard returned unexpected response"
       exit 1
   fi
   ```

2. Make executable:
   ```bash
   chmod +x deploy/compose/healthchecks/guard_health.sh
   ```

3. Test script:
   ```bash
   # With guard running
   ./deploy/compose/healthchecks/guard_health.sh && echo "PASS" || echo "FAIL"
   
   # With guard down
   docker compose stop privacy-guard
   ./deploy/compose/healthchecks/guard_health.sh && echo "PASS" || echo "FAIL"
   docker compose start privacy-guard
   ```

4. Commit:
   ```
   build(guard): add healthcheck script
   
   - Check /status endpoint
   - Verify response structure
   - Exit 0 on success, 1 on failure
   
   Refs: ADR-0021
   ```

**Acceptance:**
- Script passes when guard is healthy
- Script fails when guard is down

**Output artifacts:**
- `deploy/compose/healthchecks/guard_health.sh`

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=C3`, `checklist.C3=done`

---

### Prompt C4 — Controller Integration

**Objective:**
Add optional guard call from controller to mask audit events

**Inputs and references:**
- Read: Phase-2-Execution-Plan.md (Task C4)
- Read: `src/controller/src/main.rs` (existing audit ingest handler)

**Tasks:**
1. Add environment variables to controller in `ce.dev.yml`:
   ```yaml
   controller:
     environment:
       # ... existing vars ...
       - GUARD_ENABLED=${GUARD_ENABLED:-false}
       - GUARD_URL=http://privacy-guard:8089
   ```

2. Create `src/controller/src/guard_client.rs`:
   ```rust
   use reqwest::Client;
   use serde::{Deserialize, Serialize};
   use std::collections::HashMap;
   
   pub struct GuardClient {
       client: Client,
       base_url: String,
       enabled: bool,
   }
   
   impl GuardClient {
       pub fn new(base_url: String, enabled: bool) -> Self {
           Self {
               client: Client::new(),
               base_url,
               enabled,
           }
       }
       
       pub async fn mask(&self, text: &str, tenant_id: &str) -> Result<MaskResponse> {
           if !self.enabled {
               return Ok(MaskResponse {
                   masked_text: text.to_string(),
                   redactions: HashMap::new(),
               });
           }
           
           let url = format!("{}/guard/mask", self.base_url);
           let req = MaskRequest {
               text: text.to_string(),
               tenant_id: tenant_id.to_string(),
               session_id: None,
           };
           
           let res = self.client.post(&url)
               .json(&req)
               .send()
               .await
               .map_err(|e| format!("Guard call failed: {}", e))?;
           
           if !res.status().is_success() {
               tracing::warn!("Guard returned error, failing open");
               return Ok(MaskResponse {
                   masked_text: text.to_string(),
                   redactions: HashMap::new(),
               });
           }
           
           res.json().await.map_err(|e| format!("Failed to parse guard response: {}", e).into())
       }
   }
   
   #[derive(Serialize)]
   struct MaskRequest {
       text: String,
       tenant_id: String,
       session_id: Option<String>,
   }
   
   #[derive(Deserialize)]
   pub struct MaskResponse {
       pub masked_text: String,
       pub redactions: HashMap<String, usize>,
   }
   ```

3. Update `src/controller/src/main.rs`:
   ```rust
   mod guard_client;
   use guard_client::GuardClient;
   
   #[tokio::main]
   async fn main() {
       // ... existing setup ...
       
       let guard_enabled = std::env::var("GUARD_ENABLED")
           .unwrap_or("false".into())
           .parse::<bool>()
           .unwrap_or(false);
       let guard_url = std::env::var("GUARD_URL").unwrap_or("http://localhost:8089".into());
       let guard_client = Arc::new(GuardClient::new(guard_url, guard_enabled));
       
       tracing::info!("Privacy Guard: {}", if guard_enabled { "ENABLED" } else { "DISABLED" });
       
       // ... router setup, pass guard_client to audit_ingest handler ...
   }
   
   async fn audit_ingest_handler(
       State(guard_client): State<Arc<GuardClient>>,
       Json(mut event): Json<AuditEvent>
   ) -> Result<...> {
       // Mask content if guard enabled
       if let Some(content) = &event.content {
           let masked_res = guard_client.mask(content, &event.tenant_id).await?;
           event.content = Some(masked_res.masked_text);
           event.redactions = Some(masked_res.redactions);
           
           if !masked_res.redactions.is_empty() {
               tracing::info!(
                   tenant_id = %event.tenant_id,
                   redactions = ?masked_res.redactions,
                   "Masked audit content"
               );
           }
       }
       
       // ... continue with existing logic (store metadata) ...
   }
   ```

4. Write integration test:
   ```rust
   #[tokio::test]
   async fn test_audit_with_guard_enabled() {
       std::env::set_var("GUARD_ENABLED", "true");
       std::env::set_var("GUARD_URL", "http://localhost:8089");
       
       // Start both controller and guard
       // POST audit event with PII
       // Verify content is masked
       // Verify redactions logged
   }
   
   #[tokio::test]
   async fn test_audit_with_guard_disabled() {
       std::env::set_var("GUARD_ENABLED", "false");
       
       // POST audit event
       // Verify content unchanged
   }
   ```

5. Run tests: `cargo test`

6. Commit:
   ```
   feat(controller): integrate privacy guard for audit masking
   
   - Add GuardClient for HTTP calls to guard service
   - Add GUARD_ENABLED and GUARD_URL env vars
   - Call guard in audit_ingest handler (if enabled)
   - Log redaction counts
   - Fail-open on guard unavailability
   - Add integration tests
   
   Refs: ADR-0021, ADR-0008
   ```

**Acceptance:**
- Controller calls guard when GUARD_ENABLED=true
- Audit events contain redaction counts
- Graceful degradation if guard unavailable (fail-open)
- Tests pass for both enabled/disabled

**Output artifacts:**
- `src/controller/src/guard_client.rs` (~100 lines)
- Updated `src/controller/src/main.rs`
- Integration tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=C4`, `checklist.C4=done`, `current_workstream=D`

---

### Prompt D1 — Configuration Guide

**Objective:**
Document how to configure privacy guard (rules, policy, modes)

**Inputs and references:**
- Read: ADR-0021, ADR-0022
- Read: Phase-2-Execution-Plan.md (Task D1)
- Read: rules.yaml and policy.yaml

**Tasks:**
1. Create `docs/guides/privacy-guard-config.md`:
   ````markdown
   # Privacy Guard Configuration Guide
   
   Privacy Guard is configured via two YAML files: `rules.yaml` (detection patterns) and `policy.yaml` (masking behavior).
   
   ## Configuration Files
   
   Location: `deploy/compose/guard-config/`
   
   - `rules.yaml` - PII detection patterns and entity types
   - `policy.yaml` - Masking modes, strategies, and audit settings
   
   ## rules.yaml Structure
   
   ```yaml
   entity_types:
     ENTITY_NAME:
       display_name: "Human-readable name"
       category: "CATEGORY"  # GOVERNMENT_ID, FINANCIAL, CONTACT, IDENTITY, NETWORK
       patterns:
         - regex: 'regex pattern'
           confidence: HIGH|MEDIUM|LOW
           description: "What this pattern matches"
           context_keywords: ["optional", "keywords"]  # For MEDIUM/LOW confidence
           luhn_check: true  # Optional, for credit cards
   ```
   
   ### Adding a New Entity Type
   
   Example: Add passport number detection
   
   ```yaml
   PASSPORT:
     display_name: "Passport Number"
     category: "GOVERNMENT_ID"
     patterns:
       - regex: '\b[A-Z]{1,2}\d{6,9}\b'
         confidence: MEDIUM
         description: "Generic passport format"
         context_keywords: ["passport", "PP#", "travel document"]
   ```
   
   Then add masking strategy to `policy.yaml`:
   
   ```yaml
   masking:
     per_type:
       PASSPORT:
         strategy: PSEUDONYM
   ```
   
   ### Regex Best Practices
   
   - Use `\b` word boundaries to avoid partial matches
   - Test patterns on diverse samples
   - Start conservative (HIGH confidence) and expand
   - Use context keywords for ambiguous patterns (LOW/MEDIUM confidence)
   
   ## policy.yaml Structure
   
   ```yaml
   global:
     mode: MASK  # OFF | DETECT | MASK | STRICT
     confidence_threshold: MEDIUM  # Minimum confidence to process
   
   masking:
     default_strategy: PSEUDONYM
     per_type:
       ENTITY_NAME:
         strategy: PSEUDONYM | FPE | REDACT
         # FPE options (if strategy: FPE)
         fpe_preserve_last: 4  # For SSN, account numbers
         fpe_preserve_area_code: true  # For phone
   
   audit:
     log_detections: true
     log_redactions: true
     log_mapping_count: true
     log_performance: true
   ```
   
   ### Modes
   
   - **OFF**: No processing (pass-through)
   - **DETECT**: Detect and log, but don't mask (dry-run)
   - **MASK**: Mask detected PII (default, recommended)
   - **STRICT**: Block requests if PII detected
   
   ### Strategies
   
   - **PSEUDONYM**: Deterministic hash (e.g., `PERSON_a3f7b2c8`)
     - Use for: names, emails, IPs, most PII
     - Preserves: nothing, opaque tokens
   
   - **FPE** (Format-Preserving Encryption): Preserves format
     - Use for: phone, SSN, account numbers
     - Preserves: format, optional partial values (last 4 digits, area code)
     - Example: `555-123-4567` → `555-847-9201`
   
   - **REDACT**: Replace with placeholder
     - Use for: credit cards (show last 4)
     - Example: `4532015112830366` → `CARD_****_****_****_0366`
   
   ### Tuning Confidence Threshold
   
   - **HIGH**: Only very confident matches (low false positives)
   - **MEDIUM** (default): Balance of precision and recall
   - **LOW**: Catch more PII but risk false positives
   
   Start with MEDIUM, run in DETECT mode, review logs, adjust.
   
   ## Environment Variables
   
   Set in `deploy/compose/.env.ce`:
   
   ```bash
   # Required (from Vault)
   PSEUDO_SALT=<from-vault-secret/pseudonymization:pseudo_salt>
   
   # Optional
   GUARD_MODE=MASK  # Override policy.yaml mode
   GUARD_LOG_LEVEL=info  # trace, debug, info, warn, error
   GUARD_CONFIG_PATH=/etc/guard-config  # Config directory
   ```
   
   ## Testing Your Configuration
   
   1. Edit `rules.yaml` or `policy.yaml`
   2. Restart guard: `docker compose restart privacy-guard`
   3. Run in DETECT mode to test patterns without masking:
      ```bash
      curl -X POST http://localhost:8089/guard/scan \
        -H 'Content-Type: application/json' \
        -d '{"text": "Test SSN: 123-45-6789", "tenant_id": "org1"}'
      ```
   4. Check detections in response
   5. Switch to MASK mode when confident
   
   ## References
   
   - ADR-0021: Privacy Guard Rust Implementation
   - ADR-0022: PII Detection Rules and FPE
   - Integration Guide: [privacy-guard-integration.md](./privacy-guard-integration.md)
   ````

2. Review and validate examples

3. Commit:
   ```
   docs(guard): add configuration guide
   
   - Document rules.yaml and policy.yaml structure
   - Show how to add new entity types
   - Explain modes and strategies
   - Provide tuning guidance
   - Link to ADRs
   
   Refs: ADR-0021, ADR-0022
   ```

**Acceptance:**
- Guide is complete with examples
- User can add new entity type following guide

**Output artifacts:**
- `docs/guides/privacy-guard-config.md`

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=D1`, `checklist.D1=done`

---

### Prompt D2 — Integration Guide

**Objective:**
Document how to use privacy guard API

**Inputs and references:**
- Read: ADR-0021 (API design)
- Read: Phase-2-Execution-Plan.md (Task D2, API examples)

**Tasks:**
1. Create `docs/guides/privacy-guard-integration.md`:
   ````markdown
   # Privacy Guard Integration Guide
   
   Privacy Guard provides HTTP endpoints for PII detection and masking. Use this guide to integrate guard into applications and agents.
   
   ## Endpoints
   
   Base URL: `http://privacy-guard:8089` (Docker Compose) or `http://localhost:8089` (local)
   
   ### GET /status
   
   Health check and configuration status.
   
   ```bash
   curl http://localhost:8089/status
   ```
   
   Response:
   ```json
   {
     "status": "healthy",
     "mode": "MASK",
     "rule_count": 25,
     "config_loaded": true
   }
   ```
   
   ### POST /guard/scan
   
   Detect PII without masking (useful for dry-run).
   
   ```bash
   curl -X POST http://localhost:8089/guard/scan \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Contact John Doe at 555-123-4567 or john.doe@example.com",
       "tenant_id": "org1"
     }'
   ```
   
   Response:
   ```json
   {
     "detections": [
       {"start": 8, "end": 16, "type": "PERSON", "confidence": "MEDIUM", "matched_text": "John Doe"},
       {"start": 20, "end": 32, "type": "PHONE", "confidence": "HIGH", "matched_text": "555-123-4567"},
       {"start": 36, "end": 56, "type": "EMAIL", "confidence": "HIGH", "matched_text": "john.doe@example.com"}
     ]
   }
   ```
   
   ### POST /guard/mask
   
   Detect and mask PII.
   
   ```bash
   curl -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Contact John Doe at 555-123-4567",
       "tenant_id": "org1"
     }'
   ```
   
   Response:
   ```json
   {
     "masked_text": "Contact PERSON_a3f7b2c8 at 555-847-9201",
     "redactions": {
       "PERSON": 1,
       "PHONE": 1
     },
     "session_id": "sess_abc123"
   }
   ```
   
   **Determinism:** Same input produces same pseudonyms per tenant.
   
   ```bash
   # Call again with same text and tenant_id
   curl -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Contact John Doe at 555-123-4567",
       "tenant_id": "org1"
     }'
   # Returns same pseudonyms: PERSON_a3f7b2c8, 555-847-9201
   ```
   
   ### POST /guard/reidentify
   
   Reverse pseudonym to original value (requires JWT authentication).
   
   ```bash
   curl -X POST http://localhost:8089/guard/reidentify \
     -H 'Content-Type: application/json' \
     -H 'Authorization: Bearer <JWT>' \
     -d '{
       "pseudonym": "PERSON_a3f7b2c8",
       "session_id": "sess_abc123"
     }'
   ```
   
   Response:
   ```json
   {
     "original": "John Doe"
   }
   ```
   
   **Security:** This endpoint requires valid JWT (from OIDC). Unauthorized calls return 401.
   
   ### POST /internal/flush-session
   
   Clear mapping state for a session.
   
   ```bash
   curl -X POST http://localhost:8089/internal/flush-session \
     -H 'Content-Type: application/json' \
     -d '{"session_id": "sess_abc123"}'
   ```
   
   ## Controller Integration (Phase 2)
   
   Controller can optionally mask audit events before storing.
   
   **Environment:**
   ```bash
   GUARD_ENABLED=true
   GUARD_URL=http://privacy-guard:8089
   ```
   
   **Flow:**
   1. Agent sends audit event to controller (`POST /audit/ingest`)
   2. Controller calls `POST /guard/mask` (if GUARD_ENABLED)
   3. Controller stores masked content + redaction counts
   4. Audit log shows metadata only (no raw PII)
   
   **Fail-Open:** If guard is unavailable, controller logs warning and stores unmasked content (configurable).
   
   ## Agent-Side Integration (Future: Phase 3)
   
   **Conceptual MCP Tool Wrapper:**
   
   ```python
   # Future: goose-guard-mcp tool
   from goose.toolkit import tool
   
   @tool
   def mask_prompt(prompt: str, tenant_id: str) -> dict:
       """Mask PII in prompt before sending to model."""
       response = requests.post(
           "http://privacy-guard:8089/guard/mask",
           json={"text": prompt, "tenant_id": tenant_id}
       )
       return response.json()
   
   # Usage in agent
   masked = mask_prompt("Send email to john.doe@example.com", "org1")
   model_response = llm.call(masked["masked_text"])
   ```
   
   **Planned:** Phase 3 will add MCP extension for agent-side guard integration.
   
   ## Error Handling
   
   **400 Bad Request:**
   ```json
   {"error": "Invalid input: missing tenant_id"}
   ```
   
   **401 Unauthorized:**
   ```json
   {"error": "Unauthorized"}
   ```
   
   **404 Not Found:**
   ```json
   {"error": "Not found"}
   ```
   
   **500 Internal Server Error:**
   ```json
   {"error": "Internal error: ..."}
   ```
   
   ## Performance Considerations
   
   - P50 latency: ~50-100ms for typical prompts (~1000 chars)
   - P95 latency: ~500ms
   - P99 latency: ~1000ms
   
   For very long texts (>10KB), consider chunking or client-side pre-filtering.
   
   ## References
   
   - ADR-0002: Privacy Guard Placement
   - ADR-0021: Privacy Guard Rust Implementation
   - Configuration Guide: [privacy-guard-config.md](./privacy-guard-config.md)
   - Smoke Tests: [smoke-phase2.md](../tests/smoke-phase2.md)
   ````

2. Review and validate examples

3. Commit:
   ```
   docs(guard): add integration guide
   
   - Document all API endpoints with curl examples
   - Show controller integration pattern
   - Describe future agent-side integration
   - Include error handling and performance notes
   - Link to ADRs and other guides
   
   Refs: ADR-0021, ADR-0002
   ```

**Acceptance:**
- Guide includes working curl examples
- Integration patterns clear

**Output artifacts:**
- `docs/guides/privacy-guard-integration.md`

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=D2`, `checklist.D2=done`

---

### Prompt D3 — Smoke Test Procedure

**Objective:**
E2E validation checklist with performance benchmarking

**Inputs and references:**
- Read: Phase-2-Execution-Plan.md (Task D3)
- Read: Integration guide (curl examples)

**Tasks:**
1. Create `docs/tests/smoke-phase2.md`:
   ````markdown
   # Phase 2 Smoke Tests — Privacy Guard
   
   Manual validation checklist for Privacy Guard functionality.
   
   ## Prerequisites
   
   - Docker and Docker Compose installed
   - Repository cloned: `/home/papadoc/Gooseprojects/goose-org-twin`
   - Vault running with `PSEUDO_SALT` set (from Phase 1.2)
   
   ## Setup
   
   1. Navigate to compose directory:
      ```bash
      cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
      ```
   
   2. Ensure `.env.ce` has PSEUDO_SALT:
      ```bash
      grep PSEUDO_SALT .env.ce
      # Should show: PSEUDO_SALT=<value>
      ```
   
   3. Start services with privacy-guard profile:
      ```bash
      docker compose --profile privacy-guard up -d
      ```
   
   4. Wait for healthchecks:
      ```bash
      docker compose ps
      # All services should show "healthy"
      ```
   
   ## Test 1: Healthcheck
   
   **Objective:** Verify guard service is running
   
   ```bash
   curl http://localhost:8089/status
   ```
   
   **Expected:**
   ```json
   {
     "status": "healthy",
     "mode": "MASK",
     "rule_count": 25,  // Or however many patterns in rules.yaml
     "config_loaded": true
   }
   ```
   
   **Pass Criteria:** Status 200, all fields present, config_loaded=true
   
   ---
   
   ## Test 2: PII Detection (Scan)
   
   **Objective:** Detect PII without masking
   
   ```bash
   curl -X POST http://localhost:8089/guard/scan \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789",
       "tenant_id": "test-org"
     }'
   ```
   
   **Expected:**
   ```json
   {
     "detections": [
       {"start": 8, "end": 16, "type": "PERSON", "confidence": "MEDIUM", ...},
       {"start": 20, "end": 32, "type": "PHONE", "confidence": "HIGH", ...},
       {"start": 36, "end": 56, "type": "EMAIL", "confidence": "HIGH", ...},
       {"start": 63, "end": 74, "type": "SSN", "confidence": "HIGH", ...}
     ]
   }
   ```
   
   **Pass Criteria:**
   - Status 200
   - 4 detections (PERSON, PHONE, EMAIL, SSN)
   - Confidence levels correct
   
   ---
   
   ## Test 3: Masking with Pseudonyms
   
   **Objective:** Mask PII with PSEUDONYM strategy
   
   ```bash
   curl -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Email sent to alice.smith@example.com from 192.168.1.100",
       "tenant_id": "test-org"
     }'
   ```
   
   **Expected:**
   ```json
   {
     "masked_text": "Email sent to EMAIL_<hash>@redacted.local from IP_<hash>",
     "redactions": {
       "EMAIL": 1,
       "IP_ADDRESS": 1
     },
     "session_id": "sess_<id>"
   }
   ```
   
   **Pass Criteria:**
   - Status 200
   - EMAIL and IP_ADDRESS masked with pseudonyms
   - Redactions count correct
   - Session ID returned
   
   ---
   
   ## Test 4: Format-Preserving Encryption (Phone)
   
   **Objective:** Verify FPE preserves phone format
   
   ```bash
   curl -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Call me at 555-123-4567",
       "tenant_id": "test-org"
     }'
   ```
   
   **Expected:**
   ```json
   {
     "masked_text": "Call me at 555-XXX-XXXX",  // Different digits, same format
     "redactions": {"PHONE": 1},
     "session_id": "sess_<id>"
   }
   ```
   
   **Pass Criteria:**
   - Phone format preserved (XXX-XXX-XXXX)
   - Area code may be preserved (555-)
   - Digits different from original
   
   ---
   
   ## Test 5: Format-Preserving Encryption (SSN)
   
   **Objective:** Verify FPE preserves SSN last 4 digits
   
   ```bash
   curl -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "SSN: 123-45-6789",
       "tenant_id": "test-org"
     }'
   ```
   
   **Expected:**
   ```json
   {
     "masked_text": "SSN: XXX-XX-6789",  // Last 4 preserved
     "redactions": {"SSN": 1},
     "session_id": "sess_<id>"
   }
   ```
   
   **Pass Criteria:**
   - Last 4 digits preserved (6789)
   - First 5 digits encrypted
   - Format preserved (XXX-XX-XXXX)
   
   ---
   
   ## Test 6: Determinism
   
   **Objective:** Same input produces same pseudonyms
   
   ```bash
   # Call 1
   response1=$(curl -s -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Email: test@example.com",
       "tenant_id": "test-org"
     }')
   
   # Call 2 (same input, same tenant)
   response2=$(curl -s -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Email: test@example.com",
       "tenant_id": "test-org"
     }')
   
   # Compare
   echo "$response1" | jq .masked_text
   echo "$response2" | jq .masked_text
   ```
   
   **Pass Criteria:**
   - Both responses have identical `masked_text`
   - Pseudonyms are deterministic
   
   ---
   
   ## Test 7: Reidentification
   
   **Objective:** Reverse pseudonym to original (with JWT)
   
   **Note:** This test requires a valid JWT from Keycloak. Follow Phase 1.2 smoke tests to obtain JWT.
   
   ```bash
   # First, mask some text and save session_id
   mask_response=$(curl -s -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Contact Jane Doe",
       "tenant_id": "test-org"
     }')
   
   session_id=$(echo "$mask_response" | jq -r .session_id)
   pseudonym=$(echo "$mask_response" | jq -r '.masked_text' | grep -oP 'PERSON_\w+')
   
   # Get JWT (from Keycloak - see Phase 1.2 smoke tests)
   JWT="<your-jwt-here>"
   
   # Reidentify
   curl -X POST http://localhost:8089/guard/reidentify \
     -H 'Content-Type: application/json' \
     -H "Authorization: Bearer $JWT" \
     -d "{
       \"pseudonym\": \"$pseudonym\",
       \"session_id\": \"$session_id\"
     }"
   ```
   
   **Expected:**
   ```json
   {
     "original": "Jane Doe"
   }
   ```
   
   **Pass Criteria:**
   - With JWT: Status 200, original value returned
   - Without JWT: Status 401 Unauthorized
   
   ---
   
   ## Test 8: Audit Logs (No PII)
   
   **Objective:** Verify logs contain only counts, not raw PII
   
   ```bash
   # Run a mask request
   curl -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Secret: My SSN is 987-65-4321",
       "tenant_id": "test-org"
     }'
   
   # Check logs
   docker compose logs privacy-guard | tail -20
   ```
   
   **Expected Log Entry (structured JSON):**
   ```json
   {
     "timestamp": "2025-11-03T...",
     "tenant_id": "test-org",
     "mode": "MASK",
     "entity_counts": {"SSN": 1},
     "total_redactions": 1,
     "performance_ms": 45
   }
   ```
   
   **Pass Criteria:**
   - Log contains counts and metadata
   - Log DOES NOT contain "987-65-4321" or any raw PII
   - Performance_ms field present
   
   ---
   
   ## Test 9: Performance Benchmarking
   
   **Objective:** Measure P50, P95, P99 latency
   
   Create test script `bench_guard.sh`:
   ```bash
   #!/bin/bash
   for i in {1..100}; do
     start=$(date +%s%3N)
     curl -s -X POST http://localhost:8089/guard/mask \
       -H 'Content-Type: application/json' \
       -d '{
         "text": "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789. Credit card: 4532015112830366.",
         "tenant_id": "test-org"
       }' > /dev/null
     end=$(date +%s%3N)
     echo $((end - start))
   done | sort -n | awk '
     {arr[NR]=$1}
     END {
       print "P50: " arr[int(NR*0.5)]
       print "P95: " arr[int(NR*0.95)]
       print "P99: " arr[int(NR*0.99)]
     }
   '
   ```
   
   Run:
   ```bash
   chmod +x "Technical Project Plan/PM Phases/Phase-2/bench_guard.sh"
   ./Technical Project Plan/PM Phases/Phase-2/bench_guard.sh
   ```
   
   **Expected:**
   ```
   P50: <50-100ms>
   P95: <500-1000ms>
   P99: <1000-2000ms>
   ```
   
   **Pass Criteria:**
   - P50 ≤ 500ms ✅ (target met)
   - P95 ≤ 1000ms ✅ (target met)
   - P99 ≤ 2000ms ✅ (target met)
   
   ---
   
   ## Test 10: Controller Integration (Optional)
   
   **Objective:** Verify controller calls guard when enabled
   
   **Prerequisites:** `GUARD_ENABLED=true` in controller env
   
   1. Enable guard in controller:
      ```bash
      echo "GUARD_ENABLED=true" >> .env.ce
      docker compose restart controller
      ```
   
   2. Send audit event with PII:
      ```bash
      JWT="<your-jwt>"  # From Phase 1.2
      curl -X POST http://localhost:8088/audit/ingest \
        -H 'Content-Type: application/json' \
        -H "Authorization: Bearer $JWT" \
        -d '{
          "tenant_id": "test-org",
          "event_type": "task_completed",
          "content": "User alice@example.com completed task"
        }'
      ```
   
   3. Check controller logs:
      ```bash
      docker compose logs controller | grep "Masked audit content"
      ```
   
   **Expected:**
   - Controller log shows redaction counts: `{"EMAIL": 1}`
   - Stored audit event has masked content
   
   **Pass Criteria:**
   - Controller calls guard
   - Redactions logged
   - No raw PII in controller logs
   
   ---
   
   ## Cleanup
   
   ```bash
   docker compose --profile privacy-guard down
   ```
   
   ---
   
   ## Summary
   
   **Total Tests:** 10  
   **Required Passes:** All 10 (Test 10 optional if controller integration not enabled)
   
   **Acceptance:**
   - All functional tests pass (1-8)
   - Performance targets met (Test 9)
   - No PII in logs (Test 8)
   
   **Sign-Off:**
   - [ ] All tests passed
   - [ ] Performance within SLA (P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s)
   - [ ] No PII in logs verified
   - [ ] Ready for Phase 2 completion
   
   **Date:** __________  
   **Tester:** __________
   ````

2. Review and validate procedure

3. Commit:
   ```
   test(guard): add Phase 2 smoke test procedure
   
   - 10 E2E validation tests
   - Health, detection, masking, FPE, determinism, reidentify
   - Audit log validation (no PII)
   - Performance benchmarking (P50/P95/P99)
   - Controller integration test
   - Include expected outputs and pass criteria
   
   Refs: ADR-0021, ADR-0022
   ```

**Acceptance:**
- All smoke test steps documented
- Expected outputs included
- Performance benchmarking commands provided

**Output artifacts:**
- `docs/tests/smoke-phase2.md`

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=D3`, `checklist.D3=done`

---

### Prompt D4 — Update Project Docs

**Objective:** Sync architecture and tracking docs with Phase 2 changes

**Inputs and references:**
- Read: `docs/architecture/mvp.md`
- Read: `VERSION_PINS.md`
- Read: `PROJECT_TODO.md`

**Tasks:**
1. Update `docs/architecture/mvp.md`:
   - Add Privacy Guard to component list
   - Add flow diagram showing guard placement
   - Update data flow section

2. Update `VERSION_PINS.md` (if new images):
   - No new external images in Phase 2 (guard built from source)
   - Document guard build from `src/privacy-guard/`

3. Update `PROJECT_TODO.md`:
   ```markdown
   ## Phase 2 — Privacy Guard (M) — ✅ COMPLETE
   **Delivered:** [Date]
   
   ### Delivered
   - ✅ Rust HTTP service with Axum (port 8089)
   - ✅ Regex-based PII detection (8 entity types)
   - ✅ HMAC-SHA256 deterministic pseudonymization
   - ✅ Format-preserving encryption for phone/SSN
   - ✅ HTTP API: /guard/scan, /guard/mask, /guard/reidentify
   - ✅ In-memory mapping state (session-scoped)
   - ✅ Configuration via rules.yaml and policy.yaml
   - ✅ Docker Compose integration (privacy-guard service)
   - ✅ Controller integration (optional GUARD_ENABLED flag)
   - ✅ Synthetic test data (100+ PII samples)
   - ✅ Performance: P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s (met)
   
   ### Key Implementation
   - ADR-0021: Rust implementation, HTTP service, in-memory state
   - ADR-0022: Regex detection, FPE, extensible rules
   - Default mode: MASK (mask-and-forward)
   - No raw PII in logs (counts only)
   
   **Completion:** See `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md`
   ```

4. Update `CHANGELOG.md`:
   ```markdown
   ## [Phase 2] - 2025-11-03
   
   ### Added
   - Privacy Guard HTTP service (Rust + Axum)
   - PII detection: SSN, EMAIL, PHONE, CREDIT_CARD, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER
   - HMAC-SHA256 deterministic pseudonymization
   - Format-preserving encryption for phone and SSN
   - REST API: /guard/scan, /guard/mask, /guard/reidentify, /status
   - Docker Compose privacy-guard service
   - Configuration guides and integration documentation
   - Smoke test procedure for Phase 2
   - ADR-0021: Privacy Guard Rust Implementation
   - ADR-0022: PII Detection Rules and FPE
   
   ### Changed
   - Controller: Optional guard integration via GUARD_ENABLED flag
   
   ### Security
   - All PII masked before logging
   - Audit events contain counts only, no raw data
   - JWT required for reidentification endpoint
   ```

5. Commit:
   ```
   docs(phase2): update project documentation for Phase 2 completion
   
   - Update mvp.md with guard component
   - Mark Phase 2 complete in PROJECT_TODO
   - Add Phase 2 changelog entries
   - Link to completion summary
   
   Refs: Phase 2
   ```

**Acceptance:**
- Architecture docs reflect Phase 2
- PROJECT_TODO accurate
- CHANGELOG updated

**Output artifacts:**
- Updated `docs/architecture/mvp.md`
- Updated `PROJECT_TODO.md`
- Updated `CHANGELOG.md`

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=D4`, `checklist.D4=done`, `current_workstream=DONE`

---

## Final Steps — Phase 2 Completion

When all workstreams (A, B, C, D) are complete and all checklist items are marked "done":

1. **Run full smoke test:**
   - Follow `docs/tests/smoke-phase2.md`
   - Document results in progress log

2. **Write completion summary:**
   - Create `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md`
   - Similar structure to Phase 1.2 summary
   - Include:
     - Objectives achieved
     - What was delivered (code, config, docs)
     - Validation results (smoke tests, performance benchmarks)
     - Issues encountered and resolutions
     - ADR alignment table
     - Next steps

3. **Update state JSON:**
   - Set `current_workstream=DONE`
   - Set `performance_results` with actual P50/P95/P99
   - Add final notes

4. **Update progress log:**
   - Final entry with completion timestamp
   - Link to completion summary
   - Performance results

5. **Prepare PRs:**
   - Merge feature branches or create PRs:
     - `feat/phase2-guard-core`
     - `feat/phase2-guard-config`
     - `feat/phase2-guard-deploy`
     - `docs/phase2-guides`
   - Provide PR titles and descriptions

6. **Suggest next steps:**
   - "Phase 2 complete. Ready to proceed with Phase 2.2 (Privacy Guard Enhancement with local model) or Phase 3 (Controller API + Agent Mesh)?"

---

## Out of Scope (Phase 2)

Explicitly NOT in this phase:
- Local LLM/NER integration (Phase 2.2)
- Provider middleware wrapper (Phase 3+)
- Image/file redaction (Post-MVP)
- Persistent mapping state
- Multi-tenant key isolation (single tenant MVP)
- Dashboard/UI
- Advanced NER beyond regex
- Context-aware masking

---

## Version History

- v1.0 (2025-11-03): Initial Phase 2 agent prompts based on approved execution plan
