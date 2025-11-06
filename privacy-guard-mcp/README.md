# Privacy Guard MCP Extension

**Version:** 0.1.0  
**License:** Apache-2.0

Local PII protection for Goose agents via MCP (Model Context Protocol) stdio extension.

---

## Overview

Privacy Guard MCP intercepts prompts before they reach LLM providers, applying:

1. **Redaction:** Regex + NER-based PII detection
2. **Tokenization:** Replace PII with deterministic tokens
3. **Storage:** Encrypted token mapping (local ~/.goose/pii-tokens/)
4. **Detokenization:** Restore PII in responses
5. **Audit:** Log redactions to Controller API (metadata only)

---

## Features

### Privacy Modes

- **Rules:** Regex-only (fastest, P50 < 50ms)
- **NER:** Ollama-based named entity recognition (accurate, slower)
- **Hybrid:** Rules first, then NER (balanced)
- **Off:** Passthrough (no protection)

### Strictness Levels

- **Strict:** Maximum protection (deny on uncertainty)
- **Moderate:** Balanced (redact likely PII)
- **Permissive:** Minimal (high-confidence only)

### PII Categories

- SSN (Social Security Number)
- Email addresses
- Phone numbers
- Credit card numbers
- Employee IDs
- IP addresses
- Person names (NER)
- Organizations (NER)

### Local-Only Mode (Legal Role)

For attorney-client privilege compliance:
- Routes ALL prompts to local Ollama (http://localhost:11434)
- Zero cloud provider requests
- Enforced via profile policy

---

## Installation

### 1. Build from Source

```bash
cd privacy-guard-mcp
cargo build --release

# Binary location
./target/release/privacy-guard-mcp
```

### 2. Install to PATH

```bash
cargo install --path .
# Installs to ~/.cargo/bin/privacy-guard-mcp
```

---

## Configuration

### Environment Variables

```bash
# Privacy mode (rules, ner, hybrid, off)
export PRIVACY_GUARD_MODE=hybrid

# Strictness level (strict, moderate, permissive)
export PRIVACY_GUARD_STRICTNESS=moderate

# PII categories to detect (comma-separated)
export PRIVACY_GUARD_CATEGORIES=SSN,EMAIL,PHONE,CREDIT_CARD,EMPLOYEE_ID

# Controller API URL (for audit logs)
export CONTROLLER_URL=http://localhost:8088

# Ollama URL (for NER mode)
export OLLAMA_URL=http://localhost:11434

# Token storage directory
export PRIVACY_GUARD_TOKEN_DIR=~/.goose/pii-tokens

# Local-only mode (for Legal role - no cloud providers)
export PRIVACY_GUARD_LOCAL_ONLY=false

# Encryption key (base64-encoded 32 bytes for AES-256)
# If not set, ephemeral key is generated (tokens lost on restart)
export PRIVACY_GUARD_ENCRYPTION_KEY=$(openssl rand -base64 32)
```

### Goose Configuration

Add to `~/.config/goose/config.yaml`:

```yaml
provider: openrouter
model: anthropic/claude-3.5-sonnet

# Privacy Guard MCP Extension
extensions:
  - name: privacy-guard
    type: stdio
    command: ["privacy-guard-mcp"]
    env:
      PRIVACY_GUARD_MODE: "hybrid"
      PRIVACY_GUARD_STRICTNESS: "moderate"
      CONTROLLER_URL: "http://localhost:8088"
```

---

## Usage

### As MCP Stdio Extension

Privacy Guard runs automatically when configured in Goose:

```
Goose Client
  ↓ (sends prompt with PII)
Privacy Guard MCP
  ↓ (redacts: "John SSN 123-45-6789" → "[PERSON_A] SSN [SSN_XXX]")
  ↓ (stores tokens locally)
OpenRouter/Anthropic API
  ↓ (receives ONLY redacted text)
  ↓ (responds with tokens: "[PERSON_A] approved")
Privacy Guard MCP
  ↓ (detokenizes: "[PERSON_A]" → "John")
  ↓ (sends audit log to Controller)
Goose Client
  ↓ (user sees: "John approved")
```

### User Overrides

Users can customize privacy settings in Goose UI:

```yaml
# ~/.config/goose/privacy-overrides.yaml
mode: "rules"  # Downgrade from hybrid
strictness: "strict"  # Upgrade from moderate
disabled_categories: ["EMAIL"]  # Allow emails unredacted
```

---

## Development Status (Phase 5 Workstream E)

### Completed (E1-E3) ✅
- [x] **E1:** Crate scaffold created
  - [x] MCP stdio protocol handler
  - [x] Configuration system (env vars)
  - [x] Module structure (config, interceptor, redaction, tokenizer)
  - [x] Basic tests

- [x] **E2:** Tokenization and NER integration
  - [x] Enhanced tokenization logic with deterministic tokens
  - [x] Ollama NER client (health check + entity extraction)
  - [x] Enhanced redaction with graceful degradation
  - [x] Integration tests (5 tests)
  - [x] Build: 0 errors, 20/20 tests passing

- [x] **E3:** Response interceptor implementation
  - [x] Complete audit log submission to Controller
  - [x] Category extraction from token map
  - [x] HTTP POST with timeout + error handling
  - [x] Integration tests with mock server (2 tests)
  - [x] Audit logging enable/disable support
  - [x] Build: 0 errors, 22/22 tests passing

### Pending (E4-E9)
- [ ] **E4:** Token storage encryption
  - [ ] AES-GCM encryption (256-bit key)
  - [ ] Secure key management

- [ ] **E5:** Controller audit endpoint (POST /privacy/audit)
  - [ ] Create endpoint in Controller
  - [ ] POST /privacy/audit route
  - [ ] Metadata storage (no content)

- [ ] **E6:** User override UI mockup
  - [ ] Goose client integration proposal

- [ ] **E7:** Integration test - Finance PII redaction
- [ ] **E8:** Integration test - Legal local-only enforcement
- [ ] **E9:** Performance test - P50 < 500ms

---

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
# Requires: Controller, Ollama running
./tests/integration/test_privacy_mcp_redaction.sh
./tests/integration/test_privacy_mcp_local_only.sh
```

### Performance Tests

```bash
./tests/perf/privacy_latency_test.sh
# Target: P50 < 500ms (regex-only), P99 < 2s (with NER)
```

---

## Architecture

### Components

- **config.rs:** Environment-based configuration
- **interceptor.rs:** Request/response interception
- **redaction.rs:** PII detection (regex + NER)
- **tokenizer.rs:** Token generation + storage
- **main.rs:** MCP stdio server

### Data Flow

```
1. Goose sends prompt → Privacy Guard MCP (stdin)
2. Privacy Guard applies redaction → stores tokens
3. Privacy Guard forwards redacted prompt → LLM provider
4. LLM responds with tokens → Privacy Guard (stdin)
5. Privacy Guard detokenizes → sends audit log
6. Privacy Guard returns restored response → Goose (stdout)
```

---

## Security Considerations

### Token Storage

- **Current (E1):** Plain JSON (INSECURE - development only)
- **Target (E4):** AES-256-GCM encrypted JSON
- **Key Management:** Environment variable or OS keychain

### Audit Logs

- **Metadata Only:** No prompt/response content sent to Controller
- **Logged:** session_id, redaction_count, categories, mode, timestamp
- **NOT Logged:** Actual PII values, prompt text, response text

### Local-Only Mode

- **Legal Role:** MUST use local Ollama (no cloud providers)
- **Enforcement:** Profile policy + Privacy Guard verification
- **Verification:** Integration test checks zero cloud requests

---

## Performance Targets

- **Regex-only (Rules mode):** P50 < 50ms, P99 < 200ms
- **Hybrid mode:** P50 < 500ms, P99 < 2s
- **NER-only:** P50 < 1s, P99 < 5s (acceptable for compliance)

---

## License

Apache-2.0 (same as Goose core)

---

## Contributing

See main project CONTRIBUTING.md

---

## Phase 5 Integration

**Workstream E:** Privacy Guard MCP Extension  
**Estimated:** 2 days  
**Actual:** 50 minutes (E1-E3)  
**Status:** E1-E3 complete ✅, E4-E9 pending

**Next:** E4 - Token storage encryption (AES-256-GCM)
