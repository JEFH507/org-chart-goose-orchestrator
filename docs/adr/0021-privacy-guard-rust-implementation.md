# ADR 0021: Privacy Guard Rust Implementation and Architecture

- Status: Accepted (Phase 2)
- Date: 2025-11-03
- Authors: @owner
- Supersedes: None
- Related: ADR-0002 (Guard Placement), ADR-0009 (Deterministic Keys), ADR-0015 (Model Selection)

## Context

Phase 2 implements the Privacy Guard as defined in ADR-0002. We need to decide:
1. Implementation language (Rust vs Python vs Go)
2. Deployment model (HTTP service vs embedded library)
3. State management strategy (in-memory vs persistent)
4. Graceful degradation behavior when prerequisites missing

## Decision

### Language: Rust

**Rationale:**
- Consistency with controller (Phase 1) - shared tooling, build process, dependencies
- Performance requirements (P50 ≤ 500ms) favor compiled language
- Memory safety critical for cryptographic operations (HMAC, FPE)
- Strong regex crate ecosystem (`regex` crate is highly optimized)
- Easy integration with Axum (HTTP framework already in use)
- Single binary deployment (no runtime dependencies)

**Alternatives considered:**
- Python: Easier NER libraries, but slower; would require separate runtime
- Go: Good performance, but introduces second language; less cryptographic rigor

### Deployment: HTTP Service (Phase 2)

**Rationale:**
- Decoupled from controller and agents (can upgrade independently)
- Language-agnostic interface (future: Python agents, JS tools)
- Horizontally scalable (though single instance sufficient for MVP)
- Consistent with HTTP-only architecture (ADR-0001)
- Easier to add optional provider middleware wrapper later (ADR-0002)

**Service Characteristics:**
- Standalone process
- RESTful HTTP API
- Stateless endpoints (session state in-memory, not persisted)
- Docker Compose profile: `privacy-guard`
- Port: 8089 (controller: 8088, Keycloak: 8080)

**Future (Post-MVP):**
- Optionally compile as library crate for tight agent integration
- MCP tool wrapper for agent-side calls

### State Management: In-Memory Only

**Rationale:**
- Aligns with metadata-only storage policy (ADR-0005)
- Mapping state needed only for session duration (re-identification)
- No PII persistence requirement
- Simple operational model (restart = clean state)
- Lower attack surface (no db, no files with PII)

**Implementation:**
- Session-scoped HashMap: `original → pseudonym`
- Thread-safe via RwLock or DashMap
- Optional TTL-based expiry (10-minute default)
- Manual flush endpoint for session cleanup

**Persistence explicitly NOT included:**
- No database writes
- No file-based mapping logs
- Audit events log only counts and entity types (no mappings)

### Graceful Degradation Strategy

**Missing PSEUDO_SALT:**
- Default to `OFF` mode with warning log
- Return 503 Service Unavailable on `/guard/mask` requests
- Allow `/guard/scan` (detection only, no masking)
- Healthcheck returns degraded status

**Invalid Configuration:**
- Log parsing errors for rules.yaml / policy.yaml
- Fall back to hardcoded baseline rules
- Continue with reduced functionality

**Performance Degradation:**
- If P95 > 2s, log warning
- Optional: circuit breaker to OFF mode after N slow requests

## Consequences

### Benefits
- **Performance:** Compiled binary, optimized regex, no GC pauses
- **Security:** Memory-safe crypto, no persistence attack surface
- **Simplicity:** Single binary, no external dependencies at runtime
- **Consistency:** Same language as controller (shared patterns, tooling)
- **Scalability:** Stateless HTTP design allows horizontal scaling

### Trade-offs
- **Development velocity:** Rust learning curve vs Python
  - Mitigation: Controller team already familiar with Rust
- **Library ecosystem:** Fewer ML/NER libraries than Python
  - Mitigation: Phase 2 is regex-based; Phase 2.2 uses Ollama (HTTP)
- **Restart = state loss:** In-memory mappings cleared on restart
  - Mitigation: Acceptable for MVP; sessions are short-lived

### Risks
- **Complexity in FPE:** Format-preserving encryption in Rust less mature than Python
  - Mitigation: Use `fpe` crate (AES-FFX); limit to phone/SSN initially
- **Regex performance on very long texts:** Catastrophic backtracking risk
  - Mitigation: Input size limits (10KB default); timeout on regex execution

## Implementation Notes

### Crate Structure
```
src/privacy-guard/
├── Cargo.toml
├── src/
│   ├── main.rs           # HTTP server (Axum)
│   ├── detection.rs      # Regex engine + rule loading
│   ├── pseudonym.rs      # HMAC-SHA256 mapping
│   ├── redaction.rs      # Text masking + FPE
│   ├── policy.rs         # Config + mode logic
│   ├── state.rs          # In-memory mapping store
│   └── audit.rs          # Redaction event logging
└── Dockerfile
```

### Key Dependencies
- `axum` 0.7 - HTTP framework
- `tokio` 1.x - Async runtime
- `regex` 1.x - Pattern matching
- `hmac` 0.12, `sha2` 0.10 - Cryptographic hashing
- `fpe` 0.6 - Format-preserving encryption (AES-FFX)
- `serde`, `serde_json`, `serde_yaml` - Serialization
- `tracing` 0.1 - Structured logging

### API Surface (Phase 2)
```
POST /guard/scan
  → Detect entities, return spans + types (no masking)

POST /guard/mask
  → Mask + return pseudonymized text + redaction summary

POST /guard/reidentify
  → Reverse mapping (requires JWT auth header)

GET /status
  → Healthcheck + configuration status

POST /internal/flush-session
  → Clear mapping state for session ID
```

### Environment Variables
```bash
PSEUDO_SALT=<from-vault>           # Required for masking
GUARD_PORT=8089                     # HTTP listen port
GUARD_MODE=MASK                     # OFF|DETECT|MASK|STRICT
GUARD_CONFIG_PATH=/etc/guard-config # rules.yaml, policy.yaml
RUST_LOG=info                       # Logging level
```

## Alignment with Master Plan

- ✅ HTTP-only architecture (ADR-0001)
- ✅ Metadata-only storage (ADR-0005)
- ✅ Agent-side guard placement (ADR-0002)
- ✅ Deterministic pseudonymization (ADR-0009)
- ✅ Local-first processing (no cloud exposure)
- ✅ Performance target: P50 ≤ 500ms

## Decision Lifecycle

**Revisit after:**
- Phase 2.2 completion (evaluate Ollama integration complexity)
- Performance benchmarks (if P50 > 500ms, consider optimizations)
- Phase 3 Agent Mesh (evaluate library vs service trade-offs)

**Potential future changes:**
- Add library build target for tight agent integration
- Introduce caching layer if HMAC becomes bottleneck
- Add persistent mapping option for long-lived sessions (post-MVP)

## References

- Master Plan: `Technical Project Plan/master-technical-project-plan.md`
- Phase 2 Plan: `Technical Project Plan/PM Phases/Phase-2/Phase-2-Execution-Plan.md`
- ADR-0002: Privacy Guard Placement
- ADR-0005: Data Retention and Storage
- ADR-0009: Deterministic Pseudonymization Keys
- ADR-0015: Guard Model Policy and Selection
- Component docs: `Technical Project Plan/components/privacy-guard/`
