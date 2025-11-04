# Phase 2.2 — Privacy Guard Enhancement (S) — Agent Prompts

**Purpose:** Enhance Privacy Guard with local NER model (Ollama-based) to improve detection accuracy while preserving all existing functionality and local-only posture.

**Builds on:** Phase 0 (infra), Phase 1 (controller), Phase 1.2 (JWT, Vault), Phase 2 (Privacy Guard baseline)

---

## How to Use This Prompt

### Starting a New Session (First Time)
Copy the entire "Master Orchestrator Prompt" section below and paste it into a new Goose session.

### Resuming Work (Returning Later)
Copy the "Resume Prompt" section below and paste it into Goose. It will read your state and continue where you left off.

---

## Resume Prompt — Copy this block when resuming Phase 2.2

```markdown
You are resuming Phase 2.2 orchestration for goose-org-twin.

**Context:**
- Phase: 2.2 — Privacy Guard Enhancement (Small)
- Repository: /home/papadoc/Gooseprojects/goose-org-twin

**Required Actions:**
1. Read state from: `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-State.json`
2. Read last progress entry from: `docs/tests/phase2.2-progress.md`
3. Must Re-read authoritative documents:
   - `Technical Project Plan/master-technical-project-plan.md`
   - `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-Prompts.md`
   - `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Checklist.md`
   - `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Execution-Plan.md`
   - Relevant ADRs: 0002, 0015, 0021, 0022
   - Phase 2 completion: `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md`

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
- Local-only model execution (Ollama container, no cloud)
- Preserve all existing Phase 2 functionality
- Update state JSON, checklist, and progress log after each milestone
```

---

## Master Orchestrator Prompt — Copy this block for a new session

**Role:** Phase 2.2 Orchestrator for goose-org-twin

You are an engineering orchestrator responsible for executing Phase 2.2: Privacy Guard Enhancement. You will enhance the existing Rust Privacy Guard service (from Phase 2) with local NER model support via Ollama HTTP API. You must preserve all existing functionality, maintain local-only posture, and keep the scope small (≤ 2 days effort). Be pause/resume capable and persist state.

### Project Context

**Project root:** `/home/papadoc/Gooseprojects/goose-org-twin`

**Always read these source documents by absolute path at start and after resume:**
- `Technical Project Plan/master-technical-project-plan.md`
- `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Execution-Plan.md`
- `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Checklist.md`
- Prior phase summaries:
  - `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md` (CRITICAL - baseline)
  - `Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Completion-Summary.md`
  - `Technical Project Plan/PM Phases/Phase-1/Phase-1-Completion-Summary.md`
- Relevant ADRs:
  - `docs/adr/0002-privacy-guard-placement.md` (local-first requirement)
  - `docs/adr/0015-guard-model-policy-and-selection.md` (model choice)
  - `docs/adr/0021-privacy-guard-rust-implementation.md` (Phase 2 baseline)
  - `docs/adr/0022-pii-detection-rules-and-fpe.md` (detection strategy)
- Existing guard code:
  - `src/privacy-guard/src/*.rs` (Phase 2 implementation)
- Guides:
  - `docs/guides/guard-model-selection.md`
  - `docs/guides/privacy-guard-config.md`
  - `docs/guides/privacy-guard-integration.md`
- Version pins: `VERSION_PINS.md`

### State Persistence (Mandatory)

**State file:**
- `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-State.json`

**Schema:**
```json
{
  "phase": "Phase-2.2",
  "phase_name": "Privacy Guard Enhancement",
  "status": "INIT|IN_PROGRESS|COMPLETE",
  "current_workstream": "INIT|A|B|C|DONE",
  "current_task_id": "A1|A2|A3|B1|B2|C1|C2",
  "last_step_completed": "free text",
  "branches": {
    "A": "feat/phase2.2-ollama-detection",
    "B": "docs/phase2.2-guides",
    "C": "test/phase2.2-validation"
  },
  "user_inputs": {
    "os": "linux",
    "docker_available": true,
    "ollama_model": "llama3.2:1b",
    "fallback_to_regex": true,
    "enable_model_by_default": true,
    "performance_target_increase": 200
  },
  "pending_questions": [],
  "checklist": {"A1": "todo|in-progress|done", ...},
  "artifacts": {
    "adrs": [],
    "docs": [],
    "code": [],
    "config": [],
    "tests": []
  },
  "performance_results": {
    "baseline_p50_ms": 16,
    "baseline_p95_ms": 22,
    "with_model_p50_ms": null,
    "with_model_p95_ms": null,
    "accuracy_improvement": null
  },
  "notes": []
}
```

**Log progress to:** `docs/tests/phase2.2-progress.md` (append entries with timestamps, branches, commits, acceptance checks)

### Pause/Resume Protocol

When you need user input:
1. Write/update the state file with pending question(s) and current position (workstream, task)
2. Append note to `docs/tests/phase2.2-progress.md` describing what you're waiting for
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
- Create feature branch (naming: `feat/phase2.2-*` or `docs/phase2.2-*`)
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
- **Local-first model execution** (Ollama container only, no cloud)
- **Preserve all Phase 2 functionality** (no breaking changes)
- **Backward compatible API** (same endpoints, same responses)

### Phase 2.2 Specific Guardrails

- Model execution via Ollama HTTP API only (http://ollama:11434)
- Graceful fallback to regex-only if model unavailable
- Model selection configurable via env var (default: llama3.2:1b per ADR-0015)
- Performance target: P50 ≤ 700ms with model (200ms increase acceptable)
- No changes to external API contracts (scan/mask/reidentify)
- Hybrid detection: regex + model consensus (not model-only)
- Test with synthetic data only (same fixtures as Phase 2)

### Before Starting Workstream A

**User inputs to confirm:**
- OS: linux (assumed from Phase 2)
- Docker: available (assumed from Phase 2)
- Ollama model: llama3.2:1b (default, can change)
- Fallback to regex: yes (default)
- Enable model by default: yes or no (config flag)
- Performance target increase: 200ms (P50 ≤ 700ms)

**If any are missing or user wants different defaults, ask before proceeding.**

### Execution Sequence

Execute workstreams in order. Update state JSON and progress log after each task.

**Workstream A: Model Integration**
- Branch: `feat/phase2.2-ollama-detection`
- Tasks: A1 (Ollama client), A2 (hybrid detection), A3 (config & fallback)
- Estimated: 4-6 hours

**Workstream B: Documentation**
- Branch: `docs/phase2.2-guides`
- Tasks: B1 (update config guide), B2 (update integration guide)
- Estimated: 1-2 hours

**Workstream C: Testing & Validation**
- Branch: `test/phase2.2-validation` (or same as A)
- Tasks: C1 (accuracy tests), C2 (smoke tests)
- Estimated: 2-3 hours

**After all workstreams:**
- Set `current_workstream=DONE`
- Write completion summary to `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md`
- Update progress log with final status
- Update `PROJECT_TODO.md` to mark Phase 2.2 complete
- Suggest next steps (Phase 3: Controller API + Agent Mesh)

---

## Sub-Prompts (Detailed) — Use within orchestrator flow

All sub-prompts: Always read relevant docs by path, write state, and log progress. Ask for missing inputs and pause if necessary.

---

### Prompt A1 — Ollama HTTP Client

**Objective:**
Add Ollama HTTP client to privacy-guard for NER calls

**Inputs and references:**
- Read: ADR-0015 (model selection guidance)
- Read: existing `src/privacy-guard/src/detection.rs` (Phase 2 baseline)
- Read: Phase-2.2-Execution-Plan.md (Task A1 section)
- User inputs: ollama_model (default: llama3.2:1b)

**Tasks:**
1. Add dependencies to `src/privacy-guard/Cargo.toml`:
   ```toml
   reqwest = { version = "0.12", features = ["json"] }  # Already present
   # No new deps needed, reuse reqwest
   ```

2. Create `src/privacy-guard/src/ollama_client.rs`:
   ```rust
   use reqwest::Client;
   use serde::{Deserialize, Serialize};
   
   pub struct OllamaClient {
       client: Client,
       base_url: String,
       model: String,
       enabled: bool,
   }
   
   impl OllamaClient {
       pub fn new(base_url: String, model: String, enabled: bool) -> Self {
           Self {
               client: Client::builder()
                   .timeout(std::time::Duration::from_secs(5))
                   .build()
                   .expect("Failed to build HTTP client"),
               base_url,
               model,
               enabled,
           }
       }
       
       pub fn from_env() -> Self {
           let enabled = std::env::var("GUARD_MODEL_ENABLED")
               .unwrap_or("false".into())
               .parse::<bool>()
               .unwrap_or(false);
           let base_url = std::env::var("OLLAMA_URL")
               .unwrap_or("http://ollama:11434".into());
           let model = std::env::var("OLLAMA_MODEL")
               .unwrap_or("llama3.2:1b".into());
           
           tracing::info!(
               "Ollama NER: {} (model: {}, url: {})",
               if enabled { "ENABLED" } else { "DISABLED" },
               model,
               base_url
           );
           
           Self::new(base_url, model, enabled)
       }
       
       /// Extract named entities using Ollama chat completion
       pub async fn extract_entities(&self, text: &str) -> Result<Vec<NerEntity>, String> {
           if !self.enabled {
               return Ok(Vec::new());
           }
           
           let prompt = format!(
               "Extract PII from the following text. Return only the entity type and text, one per line.\n\
                Entity types: PERSON, ORGANIZATION, LOCATION, EMAIL, PHONE, SSN, CREDIT_CARD, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER\n\
                Format: TYPE: text\n\n\
                Text: {}\n\n\
                Entities:",
               text
           );
           
           let req = OllamaRequest {
               model: self.model.clone(),
               prompt,
               stream: false,
           };
           
           let url = format!("{}/api/generate", self.base_url);
           let res = self.client.post(&url)
               .json(&req)
               .send()
               .await
               .map_err(|e| format!("Ollama request failed: {}", e))?;
           
           if !res.status().is_success() {
               tracing::warn!("Ollama returned error status: {}", res.status());
               return Ok(Vec::new());  // Fail gracefully
           }
           
           let ollama_res: OllamaResponse = res.json()
               .await
               .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
           
           // Parse response text to extract entities
           Ok(parse_ner_response(&ollama_res.response))
       }
   }
   
   #[derive(Serialize)]
   struct OllamaRequest {
       model: String,
       prompt: String,
       stream: bool,
   }
   
   #[derive(Deserialize)]
   struct OllamaResponse {
       response: String,
   }
   
   #[derive(Debug, Clone)]
   pub struct NerEntity {
       pub entity_type: String,
       pub text: String,
   }
   
   fn parse_ner_response(response: &str) -> Vec<NerEntity> {
       let mut entities = Vec::new();
       for line in response.lines() {
           if let Some((entity_type, text)) = line.split_once(':') {
               let entity_type = entity_type.trim().to_uppercase();
               let text = text.trim();
               if !entity_type.is_empty() && !text.is_empty() {
                   entities.push(NerEntity {
                       entity_type,
                       text: text.to_string(),
                   });
               }
           }
       }
       entities
   }
   
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_parse_ner_response() {
           let response = "PERSON: John Doe\nEMAIL: john@example.com\nPHONE: 555-1234";
           let entities = parse_ner_response(response);
           assert_eq!(entities.len(), 3);
           assert_eq!(entities[0].entity_type, "PERSON");
           assert_eq!(entities[0].text, "John Doe");
       }
       
       #[test]
       fn test_ollama_client_disabled() {
           let client = OllamaClient::new(
               "http://localhost:11434".into(),
               "llama3.2:1b".into(),
               false
           );
           assert!(!client.enabled);
       }
   }
   ```

3. Add module declaration to `src/privacy-guard/src/main.rs`:
   ```rust
   mod ollama_client;
   use ollama_client::OllamaClient;
   ```

4. Update AppState in `main.rs` to include OllamaClient:
   ```rust
   struct AppState {
       rules: Rules,
       policy: Policy,
       salt: String,
       ollama_client: Arc<OllamaClient>,  // NEW
   }
   
   // In main():
   let ollama_client = Arc::new(OllamaClient::from_env());
   ```

5. Write unit tests for client initialization and parsing

6. Run tests: `cargo test --package privacy-guard`

7. Commit on branch `feat/phase2.2-ollama-detection`:
   ```
   feat(guard): add Ollama HTTP client for NER
   
   - Create OllamaClient with configurable model and URL
   - Environment-based configuration (GUARD_MODEL_ENABLED, OLLAMA_URL, OLLAMA_MODEL)
   - 5-second timeout with graceful failure
   - NER entity parsing from Ollama response
   - Unit tests for parsing and client initialization
   - Fail gracefully if Ollama unavailable
   
   Refs: ADR-0015
   ```

**Acceptance:**
- `cargo test` passes
- OllamaClient can be instantiated with env vars
- Response parsing tested

**Output artifacts:**
- `src/privacy-guard/src/ollama_client.rs` (~150 lines)
- Updated `src/privacy-guard/src/main.rs`
- Unit tests

**Logging:**
- Append to `docs/tests/phase2.2-progress.md`:
  ```
  [2025-11-04 HH:MM] A1: Ollama client complete
  - Branch: feat/phase2.2-ollama-detection
  - Commit: <hash>
  - OllamaClient implemented with env-based config
  - Tests pass, ready for hybrid detection integration
  ```
- Update state JSON: `current_task_id=A1`, `checklist.A1=done`

---

### Prompt A2 — Hybrid Detection Logic

**Objective:**
Combine regex and NER model results for improved detection

**Inputs and references:**
- Read: existing `src/privacy-guard/src/detection.rs`
- Read: Phase-2.2-Execution-Plan.md (Task A2, hybrid logic)
- Depends on: A1 (OllamaClient)

**Tasks:**
1. Update `detection.rs` to add hybrid detection function:
   ```rust
   use crate::ollama_client::{OllamaClient, NerEntity};
   
   /// Hybrid detection: combine regex + NER model
   pub async fn detect_hybrid(
       text: &str,
       rules: &Rules,
       ollama: &OllamaClient
   ) -> Vec<Detection> {
       // Step 1: Regex-based detection (fast, high precision)
       let regex_detections = detect(text, rules);
       
       // Step 2: Model-based NER (if enabled)
       let model_entities = match ollama.extract_entities(text).await {
           Ok(entities) => entities,
           Err(e) => {
               tracing::warn!("Model extraction failed, using regex only: {}", e);
               return regex_detections;  // Fallback
           }
       };
       
       // Step 3: Merge results (prioritize consensus, add model-only HIGH confidence)
       merge_detections(text, regex_detections, model_entities)
   }
   
   fn merge_detections(
       text: &str,
       regex_detections: Vec<Detection>,
       model_entities: Vec<NerEntity>
   ) -> Vec<Detection> {
       let mut merged = regex_detections.clone();
       
       // For each model entity
       for model_entity in model_entities {
           // Find in text
           if let Some(start) = text.find(&model_entity.text) {
               let end = start + model_entity.text.len();
               
               // Check if already detected by regex
               let already_detected = regex_detections.iter()
                   .any(|d| overlaps(d.start, d.end, start, end));
               
               if already_detected {
                   // Consensus: increase confidence to HIGH
                   if let Some(detection) = merged.iter_mut()
                       .find(|d| overlaps(d.start, d.end, start, end))
                   {
                       detection.confidence = Confidence::HIGH;
                   }
               } else {
                   // Model-only detection: add as HIGH confidence
                   if let Ok(entity_type) = map_ner_type(&model_entity.entity_type) {
                       merged.push(Detection {
                           start,
                           end,
                           entity_type,
                           confidence: Confidence::HIGH,
                           matched_text: model_entity.text.clone(),
                       });
                   }
               }
           }
       }
       
       // Sort by start position
       merged.sort_by_key(|d| d.start);
       merged
   }
   
   fn overlaps(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
       !(end1 <= start2 || end2 <= start1)
   }
   
   fn map_ner_type(ner_type: &str) -> Result<EntityType, String> {
       match ner_type.to_uppercase().as_str() {
           "PERSON" => Ok(EntityType::PERSON),
           "ORGANIZATION" => Ok(EntityType::PERSON),  // Map to PERSON for now
           "LOCATION" => Err("LOCATION not supported".into()),
           "EMAIL" => Ok(EntityType::EMAIL),
           "PHONE" => Ok(EntityType::PHONE),
           "SSN" => Ok(EntityType::SSN),
           "CREDIT_CARD" => Ok(EntityType::CREDIT_CARD),
           "IP_ADDRESS" => Ok(EntityType::IP_ADDRESS),
           "DATE_OF_BIRTH" => Ok(EntityType::DATE_OF_BIRTH),
           "ACCOUNT_NUMBER" => Ok(EntityType::ACCOUNT_NUMBER),
           _ => Err(format!("Unknown NER type: {}", ner_type)),
       }
   }
   ```

2. Update `scan_handler` and `mask_handler` in `main.rs` to use hybrid detection:
   ```rust
   async fn scan_handler(
       State(state): State<Arc<AppState>>,
       Json(req): Json<ScanRequest>
   ) -> Result<Json<ScanResponse>, AppError> {
       let detections = detect_hybrid(
           &req.text,
           &state.rules,
           &state.ollama_client
       ).await;
       
       Ok(Json(ScanResponse { detections }))
   }
   
   async fn mask_handler(
       State(state): State<Arc<AppState>>,
       Json(req): Json<MaskRequest>
   ) -> Result<Json<MaskResponse>, AppError> {
       let session_state = get_or_create_session(&req.tenant_id, &req.session_id);
       
       // Use hybrid detection
       let detections = detect_hybrid(
           &req.text,
           &state.rules,
           &state.ollama_client
       ).await;
       
       // Rest of masking logic unchanged
       // ...
   }
   ```

3. Write integration tests:
   ```rust
   #[tokio::test]
   async fn test_hybrid_detection_consensus() {
       // Text with both regex and model detection
       let text = "Contact John Doe at john@example.com";
       
       let ollama = OllamaClient::new("http://localhost:11434".into(), "llama3.2:1b".into(), true);
       let detections = detect_hybrid(text, &Rules::default(), &ollama).await.unwrap();
       
       // Should detect PERSON and EMAIL
       assert!(detections.iter().any(|d| d.entity_type == EntityType::PERSON));
       assert!(detections.iter().any(|d| d.entity_type == EntityType::EMAIL));
   }
   
   #[tokio::test]
   async fn test_hybrid_detection_model_disabled() {
       let text = "Contact John Doe";
       
       let ollama = OllamaClient::new("http://localhost:11434".into(), "llama3.2:1b".into(), false);
       let detections = detect_hybrid(text, &Rules::default(), &ollama).await.unwrap();
       
       // Should fall back to regex only
       assert!(detections.len() >= 0);  // May or may not detect depending on regex rules
   }
   ```

4. Run tests: `cargo test --package privacy-guard`

5. Commit:
   ```
   feat(guard): implement hybrid detection (regex + NER model)
   
   - Add detect_hybrid() function combining regex and model results
   - Consensus detection increases confidence to HIGH
   - Model-only detections added as HIGH confidence
   - Graceful fallback to regex-only if model fails
   - Update scan/mask handlers to use hybrid detection
   - Add integration tests for hybrid logic
   
   Refs: ADR-0015
   ```

**Acceptance:**
- Hybrid detection works with model enabled
- Falls back to regex-only if model disabled or fails
- Consensus detections have HIGH confidence
- Tests pass

**Output artifacts:**
- Updated `src/privacy-guard/src/detection.rs` (+100 lines)
- Updated `src/privacy-guard/src/main.rs`
- Integration tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A2`, `checklist.A2=done`

---

### Prompt A3 — Configuration & Fallback Logic

**Objective:**
Add environment-based configuration and robust fallback behavior

**Inputs and references:**
- Read: Phase-2.2-Execution-Plan.md (Task A3)
- Read: ADR-0015 (graceful degradation)

**Tasks:**
1. Update `deploy/compose/.env.ce.example`:
   ```bash
   # Privacy Guard Model (Phase 2.2)
   GUARD_MODEL_ENABLED=true  # Enable Ollama NER model
   OLLAMA_URL=http://ollama:11434
   OLLAMA_MODEL=llama3.2:1b  # Options: llama3.2:1b, llama3.2:3b, tinyllama:1.1b
   ```

2. Update `deploy/compose/ce.dev.yml` privacy-guard service:
   ```yaml
   privacy-guard:
     # ... existing config ...
     environment:
       # ... existing vars ...
       - GUARD_MODEL_ENABLED=${GUARD_MODEL_ENABLED:-true}
       - OLLAMA_URL=${OLLAMA_URL:-http://ollama:11434}
       - OLLAMA_MODEL=${OLLAMA_MODEL:-llama3.2:1b}
     depends_on:
       vault:
         condition: service_healthy
       ollama:  # NEW dependency
         condition: service_started
   ```

3. Update `/status` endpoint to report model status:
   ```rust
   async fn status_handler(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
       Json(StatusResponse {
           status: "healthy",
           mode: state.policy.mode.clone(),
           rule_count: state.rules.count(),
           config_loaded: true,
           model_enabled: state.ollama_client.enabled,  // NEW
           model_name: if state.ollama_client.enabled {
               Some(state.ollama_client.model.clone())
           } else {
               None
           },  // NEW
       })
   }
   
   #[derive(Serialize)]
   struct StatusResponse {
       status: &'static str,
       mode: GuardMode,
       rule_count: usize,
       config_loaded: bool,
       model_enabled: bool,  // NEW
       model_name: Option<String>,  // NEW
   }
   ```

4. Add health check for Ollama availability:
   ```rust
   impl OllamaClient {
       pub async fn health_check(&self) -> bool {
           if !self.enabled {
               return true;  // Not an error if disabled
           }
           
           let url = format!("{}/api/tags", self.base_url);
           match self.client.get(&url).send().await {
               Ok(res) if res.status().is_success() => true,
               Ok(_) | Err(_) => {
                   tracing::warn!("Ollama health check failed, model detection will be disabled");
                   false
               }
           }
       }
   }
   ```

5. Update startup to check Ollama health:
   ```rust
   #[tokio::main]
   async fn main() {
       // ... existing setup ...
       
       let ollama_client = Arc::new(OllamaClient::from_env());
       
       // Check Ollama health (non-blocking)
       let ollama_healthy = ollama_client.health_check().await;
       if ollama_client.enabled && !ollama_healthy {
           tracing::warn!("Ollama health check failed, model detection will fall back to regex-only");
       }
       
       // ... rest of setup ...
   }
   ```

6. Write unit tests for fallback scenarios:
   ```rust
   #[tokio::test]
   async fn test_fallback_on_model_unavailable() {
       // Simulate Ollama unavailable
       let ollama = OllamaClient::new("http://invalid:11434".into(), "llama3.2:1b".into(), true);
       let text = "Contact John Doe";
       
       // Should fall back to regex without panicking
       let detections = detect_hybrid(text, &Rules::default(), &ollama).await;
       assert!(detections.is_ok());
   }
   ```

7. Run tests: `cargo test --package privacy-guard`

8. Commit:
   ```
   feat(guard): add model configuration and fallback logic
   
   - Add env vars: GUARD_MODEL_ENABLED, OLLAMA_URL, OLLAMA_MODEL
   - Update status endpoint to report model status
   - Add Ollama health check on startup (non-blocking)
   - Graceful fallback to regex-only if model unavailable
   - Update compose config with ollama dependency
   - Add fallback tests
   
   Refs: ADR-0015
   ```

**Acceptance:**
- Configuration via env vars works
- Status endpoint shows model status
- Graceful fallback if Ollama unavailable
- Tests pass

**Output artifacts:**
- Updated `deploy/compose/.env.ce.example`
- Updated `deploy/compose/ce.dev.yml`
- Updated `src/privacy-guard/src/main.rs`
- Updated `src/privacy-guard/src/ollama_client.rs`
- Fallback tests

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=A3`, `checklist.A3=done`, `current_workstream=B`

---

### Prompt B1 — Update Configuration Guide

**Objective:**
Document model configuration options in guard config guide

**Inputs and references:**
- Read: existing `docs/guides/privacy-guard-config.md`
- Read: ADR-0015 (model selection)

**Tasks:**
1. Add new section to `docs/guides/privacy-guard-config.md`:
   ````markdown
   ## Model-Enhanced Detection (Phase 2.2)
   
   Privacy Guard can optionally use a local NER model (via Ollama) to improve detection accuracy.
   
   ### Configuration
   
   Set in `deploy/compose/.env.ce`:
   
   ```bash
   # Enable model-enhanced detection
   GUARD_MODEL_ENABLED=true  # false = regex-only (Phase 2 baseline)
   
   # Ollama configuration
   OLLAMA_URL=http://ollama:11434  # Ollama service URL
   OLLAMA_MODEL=llama3.2:1b         # Model to use for NER
   ```
   
   ### Supported Models (ADR-0015)
   
   - **llama3.2:1b** (default) - Recommended, CPU-friendly, ~1GB
   - **llama3.2:3b** - Better accuracy, more resource usage, ~3GB
   - **tinyllama:1.1b** - Smallest, lowest accuracy, ~637MB
   
   To change model:
   ```bash
   # Update .env.ce
   OLLAMA_MODEL=llama3.2:3b
   
   # Restart services
   docker compose restart privacy-guard
   ```
   
   ### How It Works
   
   **Hybrid Detection:**
   1. Regex patterns run first (fast, high precision)
   2. NER model extracts entities (slower, better recall)
   3. Results merged:
      - Consensus (both detect) → HIGH confidence
      - Model-only → HIGH confidence
      - Regex-only → Original confidence
   
   **Fallback Behavior:**
   - If `GUARD_MODEL_ENABLED=false`: regex-only (Phase 2 baseline)
   - If Ollama unavailable: automatic fallback to regex-only
   - No API changes, transparent to clients
   
   ### Performance Impact
   
   With model enabled (llama3.2:1b):
   - P50: ~150-300ms (vs 16ms regex-only)
   - P95: ~500-700ms (vs 22ms regex-only)
   - Accuracy: +10-20% recall for PERSON, ORGANIZATION
   
   Trade-off: Latency increases ~10-20x for improved accuracy.
   
   ### When to Use Model-Enhanced Detection
   
   **Use model (GUARD_MODEL_ENABLED=true):**
   - Need higher recall (catch more PII)
   - Text has ambiguous person names (no titles/context)
   - Organization names are critical
   - Latency < 1s is acceptable
   
   **Use regex-only (GUARD_MODEL_ENABLED=false):**
   - Need lowest latency (<50ms)
   - High-volume ingestion
   - PII patterns are well-structured (phone, SSN, email)
   - CPU/memory constrained
   
   ### Troubleshooting
   
   **Model not detecting:**
   - Check Ollama logs: `docker compose logs ollama`
   - Verify model pulled: `docker exec ollama ollama list`
   - Pull model manually: `docker exec ollama ollama pull llama3.2:1b`
   
   **High latency:**
   - Use smaller model: `OLLAMA_MODEL=tinyllama:1.1b`
   - Disable model: `GUARD_MODEL_ENABLED=false`
   - Check Ollama CPU usage: `docker stats ollama`
   
   **Fallback to regex:**
   - Check guard logs for "model extraction failed" warnings
   - Verify Ollama health: `curl http://localhost:11434/api/tags`
   - Restart Ollama: `docker compose restart ollama`
   ````

2. Update "Environment Variables" section:
   ```markdown
   ## Environment Variables
   
   ### Phase 2 (Baseline)
   ```bash
   PSEUDO_SALT=<from-vault>
   GUARD_MODE=MASK
   GUARD_LOG_LEVEL=info
   ```
   
   ### Phase 2.2 (Model Enhancement)
   ```bash
   GUARD_MODEL_ENABLED=true  # Enable NER model
   OLLAMA_URL=http://ollama:11434
   OLLAMA_MODEL=llama3.2:1b
   ```
   ```

3. Review and validate examples

4. Commit on branch `docs/phase2.2-guides`:
   ```
   docs(guard): update config guide for model-enhanced detection
   
   - Add Phase 2.2 model configuration section
   - Document hybrid detection logic
   - List supported models (ADR-0015)
   - Explain performance trade-offs
   - Add troubleshooting guide for model issues
   - Update environment variables section
   
   Refs: ADR-0015
   ```

**Acceptance:**
- Guide includes model configuration
- Performance trade-offs explained
- Troubleshooting steps clear

**Output artifacts:**
- Updated `docs/guides/privacy-guard-config.md` (+80 lines)

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=B1`, `checklist.B1=done`

---

### Prompt B2 — Update Integration Guide

**Objective:**
Document API behavior with model-enhanced detection

**Inputs and references:**
- Read: existing `docs/guides/privacy-guard-integration.md`

**Tasks:**
1. Add note to `/status` endpoint section:
   ```markdown
   ### GET /status
   
   Response (Phase 2.2+):
   ```json
   {
     "status": "healthy",
     "mode": "MASK",
     "rule_count": 25,
     "config_loaded": true,
     "model_enabled": true,           // NEW: Phase 2.2
     "model_name": "llama3.2:1b"      // NEW: Phase 2.2
   }
   ```
   
   **New fields (Phase 2.2):**
   - `model_enabled`: Whether NER model is active
   - `model_name`: Ollama model name (if enabled)
   ```

2. Add performance section update:
   ```markdown
   ## Performance Considerations (Updated Phase 2.2)
   
   ### Regex-Only Mode (Phase 2 baseline)
   - P50 latency: ~16ms
   - P95 latency: ~22ms
   - P99 latency: ~23ms
   - Use when: Latency critical, structured PII patterns
   
   ### Model-Enhanced Mode (Phase 2.2)
   - P50 latency: ~150-300ms (model: llama3.2:1b)
   - P95 latency: ~500-700ms
   - P99 latency: ~800-1000ms
   - Use when: Accuracy critical, unstructured text
   
   **Configuration:**
   ```bash
   # Regex-only (fastest)
   GUARD_MODEL_ENABLED=false
   
   # Model-enhanced (best accuracy)
   GUARD_MODEL_ENABLED=true
   OLLAMA_MODEL=llama3.2:1b
   ```
   
   **Client Impact:**
   - API contract unchanged (same endpoints, same responses)
   - Clients automatically benefit from improved detection
   - Fallback to regex-only transparent to clients
   ```

3. Update "Controller Integration" section:
   ```markdown
   ## Controller Integration (Phase 2)
   
   Controller integration unchanged in Phase 2.2:
   - Same GUARD_ENABLED flag
   - Same HTTP API calls
   - No code changes needed
   - Automatically uses hybrid detection if model enabled
   ```

4. Commit:
   ```
   docs(guard): update integration guide for Phase 2.2
   
   - Document new status fields (model_enabled, model_name)
   - Update performance characteristics with model
   - Note API contract unchanged (backward compatible)
   - Clarify controller integration still works
   
   Refs: Phase 2.2
   ```

**Acceptance:**
- API changes documented
- Performance impact clear
- Backward compatibility noted

**Output artifacts:**
- Updated `docs/guides/privacy-guard-integration.md` (+40 lines)

**Logging:**
- Append to progress log
- Update state JSON: `current_task_id=B2`, `checklist.B2=done`, `current_workstream=C`

---

### Prompt C1 — Accuracy Validation Tests

**Objective:**
Measure detection accuracy improvement with model vs regex-only

**Inputs and references:**
- Read: `tests/fixtures/pii_samples.txt` (Phase 2 test data)
- Read: Phase-2.2-Execution-Plan.md (Task C1)

**Tasks:**
1. Create accuracy test script `tests/accuracy/compare_detection.sh`:
   ```bash
   #!/bin/bash
   set -e
   
   echo "=== Privacy Guard Detection Accuracy Comparison ==="
   echo ""
   
   FIXTURES_DIR="tests/fixtures"
   
   # Test 1: Regex-only
   echo "1. Testing regex-only detection..."
   export GUARD_MODEL_ENABLED=false
   docker compose restart privacy-guard
   sleep 5
   
   regex_results=$(mktemp)
   while IFS= read -r line; do
     curl -s -X POST http://localhost:8089/guard/scan \
       -H 'Content-Type: application/json' \
       -d "{\"text\": \"$line\", \"tenant_id\": \"test\"}" \
       | jq -r '.detections | length'
   done < "$FIXTURES_DIR/pii_samples.txt" > "$regex_results"
   
   regex_total=$(awk '{sum+=$1} END {print sum}' "$regex_results")
   echo "   Regex-only detections: $regex_total"
   
   # Test 2: Model-enhanced
   echo "2. Testing model-enhanced detection..."
   export GUARD_MODEL_ENABLED=true
   docker compose restart privacy-guard
   sleep 10  # Model startup takes longer
   
   model_results=$(mktemp)
   while IFS= read -r line; do
     curl -s -X POST http://localhost:8089/guard/scan \
       -H 'Content-Type: application/json' \
       -d "{\"text\": \"$line\", \"tenant_id\": \"test\"}" \
       | jq -r '.detections | length'
   done < "$FIXTURES_DIR/pii_samples.txt" > "$model_results"
   
   model_total=$(awk '{sum+=$1} END {print sum}' "$model_results")
   echo "   Model-enhanced detections: $model_total"
   
   # Calculate improvement
   if [ "$regex_total" -gt 0 ]; then
     improvement=$(echo "scale=2; (($model_total - $regex_total) / $regex_total) * 100" | bc)
     echo ""
     echo "=== Results ==="
     echo "Regex-only:      $regex_total entities"
     echo "Model-enhanced:  $model_total entities"
     echo "Improvement:     ${improvement}%"
     
     # Expected: +10-20% improvement
     if (( $(echo "$improvement >= 10" | bc -l) )); then
       echo "✅ PASS: Accuracy improvement >= 10%"
       exit 0
     else
       echo "❌ FAIL: Accuracy improvement < 10% (got ${improvement}%)"
       exit 1
     fi
   else
     echo "❌ ERROR: No regex detections to compare"
     exit 1
   fi
   ```

2. Make executable:
   ```bash
   chmod +x tests/accuracy/compare_detection.sh
   ```

3. Create false positive test `tests/accuracy/test_false_positives.sh`:
   ```bash
   #!/bin/bash
   # Test that clean samples have no false positives
   
   echo "Testing false positive rate..."
   
   false_positives=0
   total=0
   
   while IFS= read -r line; do
     total=$((total + 1))
     detections=$(curl -s -X POST http://localhost:8089/guard/scan \
       -H 'Content-Type: application/json' \
       -d "{\"text\": \"$line\", \"tenant_id\": \"test\"}" \
       | jq -r '.detections | length')
     
     if [ "$detections" -gt 0 ]; then
       false_positives=$((false_positives + 1))
       echo "False positive: $line"
     fi
   done < "tests/fixtures/clean_samples.txt"
   
   fp_rate=$(echo "scale=2; ($false_positives / $total) * 100" | bc)
   echo "False positive rate: ${fp_rate}%"
   
   if (( $(echo "$fp_rate < 5" | bc -l) )); then
     echo "✅ PASS: FP rate < 5%"
     exit 0
   else
     echo "❌ FAIL: FP rate >= 5%"
     exit 1
   fi
   ```

4. Run accuracy tests:
   ```bash
   cd /home/papadoc/Gooseprojects/goose-org-twin
   ./tests/accuracy/compare_detection.sh
   ./tests/accuracy/test_false_positives.sh
   ```

5. Document results in progress log

6. Commit on branch `test/phase2.2-validation`:
   ```
   test(guard): add accuracy validation tests for Phase 2.2
   
   - Create comparison script (regex vs model)
   - Measure detection improvement (+10-20% expected)
   - Test false positive rate (< 5% target)
   - Use Phase 2 test fixtures
   
   Refs: Phase 2.2
   ```

**Acceptance:**
- Accuracy improvement measured
- FP rate within target
- Results documented

**Output artifacts:**
- `tests/accuracy/compare_detection.sh`
- `tests/accuracy/test_false_positives.sh`
- Results in progress log

**Logging:**
- Append to progress log with actual improvement %
- Update state JSON: `current_task_id=C1`, `checklist.C1=done`, `accuracy_improvement` field

---

### Prompt C2 — Smoke Tests

**Objective:**
Validate Phase 2.2 functionality end-to-end

**Inputs and references:**
- Read: `docs/tests/smoke-phase2.md` (Phase 2 baseline)
- Read: Phase-2.2-Execution-Plan.md (Task C2)

**Tasks:**
1. Create `docs/tests/smoke-phase2.2.md`:
   ````markdown
   # Phase 2.2 Smoke Tests — Model-Enhanced Detection
   
   Validation of Privacy Guard with local NER model (Ollama).
   
   ## Prerequisites
   
   Same as Phase 2 smoke tests, plus:
   - Ollama service running with llama3.2:1b model
   
   ## Setup
   
   1. Pull Ollama model (if not already):
      ```bash
      docker compose exec ollama ollama pull llama3.2:1b
      ```
   
   2. Enable model in `.env.ce`:
      ```bash
      GUARD_MODEL_ENABLED=true
      OLLAMA_MODEL=llama3.2:1b
      ```
   
   3. Restart privacy-guard:
      ```bash
      docker compose restart privacy-guard
      ```
   
   ## Test 1: Model Status Check
   
   **Objective:** Verify model is enabled and reported
   
   ```bash
   curl http://localhost:8089/status | jq
   ```
   
   **Expected:**
   ```json
   {
     "status": "healthy",
     "model_enabled": true,
     "model_name": "llama3.2:1b",
     ...
   }
   ```
   
   **Pass Criteria:** model_enabled=true, model_name correct
   
   ---
   
   ## Test 2: Model-Enhanced Detection
   
   **Objective:** Detect entities that regex might miss (e.g., person without title)
   
   ```bash
   curl -X POST http://localhost:8089/guard/scan \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Alice Cooper and Bob Dylan discussed the project.",
       "tenant_id": "test-org"
     }' | jq '.detections[] | {type, text}'
   ```
   
   **Expected:**
   ```json
   {"type": "PERSON", "text": "Alice Cooper"}
   {"type": "PERSON", "text": "Bob Dylan"}
   ```
   
   **Pass Criteria:** Both person names detected (regex-only might miss these)
   
   ---
   
   ## Test 3: Fallback to Regex (Model Disabled)
   
   **Objective:** Verify graceful fallback when model disabled
   
   ```bash
   # Disable model
   export GUARD_MODEL_ENABLED=false
   docker compose restart privacy-guard
   sleep 5
   
   # Check status
   curl http://localhost:8089/status | jq '.model_enabled'
   # Expected: false
   
   # Test still works (regex-only)
   curl -X POST http://localhost:8089/guard/scan \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Email: test@example.com, SSN: 123-45-6789",
       "tenant_id": "test-org"
     }' | jq '.detections | length'
   # Expected: >= 2 (EMAIL, SSN)
   
   # Re-enable model
   export GUARD_MODEL_ENABLED=true
   docker compose restart privacy-guard
   ```
   
   **Pass Criteria:** 
   - Status shows model_enabled=false
   - Detection still works (regex-only)
   - No errors
   
   ---
   
   ## Test 4: Performance with Model
   
   **Objective:** Measure P50/P95 latency with model enabled
   
   ```bash
   # Create benchmark script (or reuse Phase 2 bench_guard.sh)
   for i in {1..50}; do
     start=$(date +%s%3N)
     curl -s -X POST http://localhost:8089/guard/mask \
       -H 'Content-Type: application/json' \
       -d '{
         "text": "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789.",
         "tenant_id": "test-org"
       }' > /dev/null
     end=$(date +%s%3N)
     echo $((end - start))
   done | sort -n | awk '
     {arr[NR]=$1}
     END {
       print "P50: " arr[int(NR*0.5)] "ms"
       print "P95: " arr[int(NR*0.95)] "ms"
     }
   '
   ```
   
   **Expected:**
   ```
   P50: 150-300ms
   P95: 500-700ms
   ```
   
   **Pass Criteria:**
   - P50 ≤ 700ms (200ms increase from Phase 2 acceptable)
   - P95 ≤ 1000ms
   
   ---
   
   ## Test 5: Backward Compatibility
   
   **Objective:** Verify Phase 2 clients still work without changes
   
   ```bash
   # Same API calls as Phase 2
   curl -X POST http://localhost:8089/guard/mask \
     -H 'Content-Type: application/json' \
     -d '{
       "text": "Secret SSN: 987-65-4321",
       "tenant_id": "test-org"
     }' | jq
   ```
   
   **Expected:** Same response structure as Phase 2 (masked_text, redactions, session_id)
   
   **Pass Criteria:**
   - Response structure identical to Phase 2
   - SSN masked correctly
   - No breaking changes
   
   ---
   
   ## Summary
   
   **Total Tests:** 5  
   **Required Passes:** All 5
   
   **Acceptance:**
   - Model integration works
   - Detection accuracy improved
   - Fallback to regex works
   - Performance within target
   - Backward compatible
   
   **Sign-Off:**
   - [ ] All tests passed
   - [ ] Performance acceptable (P50 ≤ 700ms)
   - [ ] Backward compatibility verified
   - [ ] Ready for Phase 2.2 completion
   
   **Date:** __________  
   **Tester:** __________
   ````

2. Run all smoke tests

3. Document results in progress log

4. Commit:
   ```
   test(guard): add Phase 2.2 smoke tests
   
   - 5 E2E tests for model-enhanced detection
   - Model status, detection, fallback, performance, compatibility
   - Based on Phase 2 smoke test template
   - Expected results documented
   
   Refs: Phase 2.2
   ```

**Acceptance:**
- All smoke tests pass
- Results documented
- Performance within target

**Output artifacts:**
- `docs/tests/smoke-phase2.2.md`
- Test results in progress log

**Logging:**
- Append to progress log with test results
- Update state JSON: `current_task_id=C2`, `checklist.C2=done`, `current_workstream=DONE`

---

## Final Steps — Phase 2.2 Completion

When all workstreams (A, B, C) are complete and all checklist items are marked "done":

1. **Run full smoke test:**
   - Follow `docs/tests/smoke-phase2.2.md`
   - Document results in progress log

2. **Measure accuracy improvement:**
   - Run `tests/accuracy/compare_detection.sh`
   - Record improvement % in state JSON

3. **Write completion summary:**
   - Create `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md`
   - Similar structure to Phase 2 summary
   - Include:
     - Objectives achieved
     - What was delivered (code, docs, tests)
     - Accuracy improvement results
     - Performance impact
     - Validation results
     - Issues encountered and resolutions
     - Next steps

4. **Update state JSON:**
   - Set `current_workstream=DONE`
   - Set `status=COMPLETE`
   - Set `accuracy_improvement` with actual %
   - Set `with_model_p50_ms` and `with_model_p95_ms` with actual measurements
   - Add final notes

5. **Update progress log:**
   - Final entry with completion timestamp
   - Link to completion summary
   - Accuracy and performance results

6. **Update project docs:**
   - Update `PROJECT_TODO.md` to mark Phase 2.2 complete
   - Update `CHANGELOG.md` with Phase 2.2 entry
   - Update `VERSION_PINS.md` if needed (Ollama model version)

7. **Prepare PRs:**
   - Merge or create PRs for:
     - `feat/phase2.2-ollama-detection`
     - `docs/phase2.2-guides`
     - `test/phase2.2-validation`
   - Provide PR titles and descriptions

8. **Suggest next steps:**
   - "Phase 2.2 complete. Ready to proceed with Phase 3 (Controller API + Agent Mesh)?"

---

## Out of Scope (Phase 2.2)

Explicitly NOT in this phase:
- Custom NER training (use pre-trained models only)
- Cloud-based models (local-only via Ollama)
- Image/file content analysis
- Real-time model fine-tuning
- Multiple model ensemble
- GPU optimization (CPU-friendly models only)
- Provider middleware integration (deferred to Phase 3+)
- UI for model management

---

## Version History

- v1.0 (2025-11-04): Initial Phase 2.2 agent prompts based on master plan and Phase 2 completion
