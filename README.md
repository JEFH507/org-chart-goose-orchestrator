# goose-org-twin
Org-Chart Orchestrated AI Framework — project workspace.

## What is this?
An org-chart–aware, privacy-first AI orchestration framework. It coordinates role-based “digital twin” agents across departments via HTTP-only flows, with policy, auditability, and observability. MVP aligns to Goose v1.12.

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
