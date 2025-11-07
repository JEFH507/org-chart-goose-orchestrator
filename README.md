# goose-org-twin
Org-Chart Orchestrated AI Framework — project workspace.

## What is this?
An org-chart–aware, privacy-first AI orchestration framework. It coordinates role-based “digital twin” agents across departments via HTTP-only flows, with policy, auditability, and observability. MVP aligns to Goose v1.12.

## Architecture Overview

### Service vs. Module Distinction

The `/src` directory contains 6 components organized by architectural pattern:

```
┌─────────────────────────────────────────────────────────────────┐
│                    GOOSE DESKTOP (User's Machine)              │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────┐          ┌────────────────────────────┐ │
│  │  Goose Client UI  │─────────▶│  Agent Mesh MCP Extension  │ │
│  └───────────────────┘          │  (stdio mode, local)       │ │
│           │                      └────────┬───────────────────┘ │
│           │                               │                     │
│           │ (chat prompts)                │ (send_task, etc.)   │
│           │                               │                     │
└───────────┼───────────────────────────────┼─────────────────────┘
            │                               │
            │                               ▼
            │                     ┌─────────────────────────┐
            │                     │  CONTROLLER (port 8088) │
            │                     │  ┌───────────────────┐  │
            │                     │  │  Axum HTTP Server │  │
            │                     │  └─────────┬─────────┘  │
            │                     │            │            │
            │                     │  ┌─────────▼─────────┐  │
            │                     │  │ Routes (15 API    │  │
            │                     │  │ endpoints)        │  │
            │                     │  └─────────┬─────────┘  │
            │                     │            │            │
            │                     │  IMPORTS (lib.rs):     │
            │                     │  ┌─────────▼─────────┐  │
            │                     │  │ lifecycle module  │  │
            │                     │  │ - SessionLifecycle│  │
            │                     │  │ - State machine   │  │
            │                     │  └───────────────────┘  │
            │                     │  ┌───────────────────┐  │
            │                     │  │ profile module    │  │
            │                     │  │ - Profile schema  │  │
            │                     │  │ - Validator       │  │
            │                     │  │ - Signer (Vault)  │  │
            │                     │  └───────┬───────────┘  │
            │                     │          │              │
            │                     │  ┌───────▼───────────┐  │
            │                     │  │ vault module      │  │
            │                     │  │ - VaultClient     │  │
            │                     │  │ - TransitOps      │  │
            │                     │  │ - KV operations   │  │
            │                     │  └───────────────────┘  │
            │                     └─────────┬───────────────┘
            │                               │
            ▼                               ▼
┌───────────────────────┐       ┌─────────────────────────┐
│ PRIVACY GUARD (8089)  │       │  EXTERNAL SERVICES      │
│ - Regex detection     │       │  - Postgres (sessions,  │
│ - NER (Ollama)        │       │    profiles, audit)     │
│ - Deterministic       │       │  - Redis (idempotency)  │
│   pseudonymization    │       │  - Vault (signing keys) │
│ - HTTP API            │       │  - Keycloak (OIDC/JWT)  │
└───────────────────────┘       └─────────────────────────┘
```

**Key Patterns:**

1. **Services** (Standalone Docker containers):
   - `controller/` — Main API server (port 8088, Rust/Axum)
   - `privacy-guard/` — PII detection/masking (port 8089, Rust)

2. **Modules** (Rust libraries imported by controller):
   - `lifecycle/` — Session state machine
   - `profile/` — Profile schema/validation/signing
   - `vault/` — HashiCorp Vault client

3. **MCP Extensions** (Launched by Goose Desktop):
   - `agent-mesh/` — Multi-agent coordination (Python MCP stdio)

**Import Pattern:**
```rust
// src/controller/src/lib.rs
#[path = "../../lifecycle/mod.rs"]
pub mod lifecycle;  // Session state management

#[path = "../../vault/mod.rs"]
pub mod vault;      // Vault client library

#[path = "../../profile/mod.rs"]
pub mod profile;    // Profile system
```

See **[docs/architecture/SRC-ARCHITECTURE-AUDIT.md](docs/architecture/SRC-ARCHITECTURE-AUDIT.md)** for detailed analysis.

---

## Quick links
- Product description: ./docs/product/productdescription.md
- Technical Project Plan (master): ./Technical Project Plan/master-technical-project-plan.md
- Component Plans: ./Technical Project Plan/components/
- Project TODO: ./PROJECT_TODO.md
- Docs (GitHub-native): ./docs/README.md
- Architecture: ./docs/architecture/
- ADRs: ./docs/adr/
- Product pages: ./docs/product/
- Guides: ./docs/guides/
- Compliance: ./docs/compliance/
- API docs: ./docs/api/ (TBD)

## Repository Info
- Local path: /home/papadoc/Gooseprojects/goose-org-twin
- GitHub: https://github.com/JEFH507/org-chart-goose-orchestrator

## Structure
- Technical Project Plan/: Master plan + component plans
- docs/
  - product/: Product one-pagers and posture docs
  - architecture/: Diagrams and architecture notes (MVP, roadmap, one-pager)
  - adr/: Architecture Decision Records (0001+)
  - api/: API documentation (to be added)
  - guides/: User and admin guides
  - compliance/: Enterprise/compliance notes
  - pitch/: Pitch and SOW templates
  - security/: Security and key/signing docs
  - tests/: Test and smoke guides
  - assets/, site/: Assets and site scaffolds (if used)
- goose-versions-references/: Upstream Goose references + analysis docs
- scripts/: Automation scripts (setup, deploy, backup)
- config/: Configuration files and templates
- tests/: Test suites (to be added)
- src/: Source code (to be added in implementation phases)
- .env.example
- .gitignore
- .goosehints
- .gooseignore
- CHANGELOG.md
- PROJECT_TODO.md
- README.md

## How we work
- Branching: feature branches, conventional commits, PR reviews
- Privacy-by-design: agent-side masking, metadata-only server, deterministic pseudonymization
- MVP constraints: HTTP-only orchestration, single-tenant, CE defaults (Keycloak, Vault OSS, Postgres, Ollama; optional S3-compatible object storage — SeaweedFS default option; MinIO/Garage optional)
- Observability/audit: OTLP traces, structured logs, ndjson export; see component plans

## Grant alignment
- License: Apache-2.0 (core)
- Goose Grant Program alignment: docs/grants/GRANT_PROPOSAL_DRAFT.md
- Community Edition (CE) focus: self-hostable, OSS defaults
- Pilot goal: validate MVP with 1–2 design partners

## Repository Info (Phase 0)
- Local path: /home/papadoc/Gooseprojects/goose-org-twin
- GitHub: https://github.com/JEFH507/org-chart-goose-orchestrator

## Progress Status

**Current Phase:** Phase 2.2 ✅ COMPLETE (2025-11-04)

### Completed Phases
- ✅ **Phase 0:** Project Setup (repo hygiene, docker compose baseline, OpenAPI stubs)
- ✅ **Phase 1:** Initial Runtime (minimal controller, CI skeleton, Keycloak/Vault seeding)
- ✅ **Phase 1.2:** Identity & Security (JWT verification, OIDC integration, Vault wiring)
- ✅ **Phase 2:** Privacy Guard (regex-based PII detection, pseudonymization, FPE, masking)
- ✅ **Phase 2.2:** Privacy Guard Enhancement (Ollama + qwen3:0.6b model, hybrid detection)

### Next Phases (Per Master Plan)
- 📋 **Phase 2.3 (Optional):** Performance Optimization (~1-2 days)
- 📋 **Phase 3:** Controller API + Agent Mesh (Large - 1-2 weeks)
- 📋 **Phase 4:** Directory/Policy + Profiles (Medium - 3-5 days)
- 📋 **Phase 5:** Audit & Observability (Small - ≤2 days)
- 📋 **Phase 6:** Model Orchestration (Medium - 3-5 days)
- 📋 **Phase 7:** Storage/Metadata (Small - ≤2 days)
- 📋 **Phase 8:** Packaging/Deployment + Docs (Medium - 3-5 days)

**See:** `PROJECT_TODO.md` for detailed tracking and `Technical Project Plan/master-technical-project-plan.md` for full roadmap.
