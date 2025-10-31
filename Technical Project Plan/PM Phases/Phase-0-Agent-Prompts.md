# Phase 0 Agent Prompts (for later coding; no code now)

Each prompt specifies objective, assumed inputs, constraints, acceptance criteria, and expected output artifacts (filenames/paths). Use these with Goose agents via “vibe coding” to generate code later. Do not implement now.

## A. Repo Scaffolding

### Prompt A1 — Repository hygiene and conventions
- Objective: Create repository hygiene artifacts and conventions aligned with ADRs and Goose v1.12 practices.
- Inputs to assume:
  - Root repo path: ~/Gooseprojects/goose-org-twin
  - Conventional commits policy; ADRs 0001–0013 present
- Constraints:
  - Planning phase only; do not add service code
  - Keep changes non-destructive; no overwriting existing docs without review
- Acceptance criteria:
  - PR template and issue templates present
  - CONTRIBUTING.md references ADR directory and commit conventions
  - Branch protection guidance documented
- Expected output artifacts:
  - .github/PULL_REQUEST_TEMPLATE.md
  - .github/ISSUE_TEMPLATE/bug.md
  - .github/ISSUE_TEMPLATE/feature.md
  - CONTRIBUTING.md
  - docs/conventions/commit-style.md
  - CODEOWNERS (optional stub)

### Prompt A2 — Scripts skeleton
- Objective: Add placeholders for dev scripts without implementation details.
- Inputs to assume:
  - Bash environment on Linux/macOS
- Constraints:
  - Only placeholders with usage comments
- Acceptance criteria:
  - Scripts exist and are executable stubs with TODOs
- Expected output artifacts:
  - scripts/dev/bootstrap.sh
  - scripts/dev/checks.sh
  - scripts/dev/health.sh

## B. Environment Bootstrap

### Prompt B1 — Developer setup guide
- Objective: Author dev-setup guide for Linux/macOS including prerequisites and ports map.
- Inputs to assume:
  - Docker Desktop or Docker Engine available
-  - Ports defaults: Keycloak 8080, Vault 8200, Postgres 5432, Ollama 11434, SeaweedFS 8333/9333/8081 (S3/master UI/filer), MinIO 9000/9001
- Constraints:
  - Document .env overrides; no platform-specific code
- Acceptance criteria:
  - doc explains prerequisites, environment variables, and how to run smoke checks
- Expected output artifacts:
  - docs/guides/dev-setup.md
  - deploy/compose/.env.ce (example values)

### Prompt B2 — Version pinning and ports registry
- Objective: Establish a single source of truth for service versions and default ports.
- Inputs to assume:
-  - Current stable images: Keycloak, Vault, Postgres, Ollama, and one S3 option (SeaweedFS default; MinIO/Garage optional)
- Constraints:
  - Use tags not “latest”; document rationale
- Acceptance criteria:
  - All versions and ports listed and cross-referenced by compose docs
- Expected output artifacts:
  - VERSION_PINS.md
  - docs/guides/ports.md

## C. CE Defaults docker-compose (Keycloak, Vault, Postgres, Ollama; optional S3-compatible object storage)

### Prompt C1 — Compose file baseline with healthchecks
- Objective: Create ce.dev.yml with services and basic health checks; allow optional S3-compatible storage (SeaweedFS default option; MinIO/Garage optional).
- Inputs to assume:
  - CPU-only; minimal resource requests
  - .env.ce for port overrides
- Constraints:
  - No app services; only infrastructure
  - Health checks: curl or nc; keep timeouts conservative
- Acceptance criteria:
  - docker compose up brings all services healthy
- Disabling Ollama or enabling/disabling S3 storage via env is documented
- Expected output artifacts:
  - deploy/compose/ce.dev.yml
  - deploy/compose/.env.ce
  - deploy/compose/healthchecks/keycloak.sh
  - deploy/compose/healthchecks/vault.sh
  - deploy/compose/healthchecks/postgres.sh
  - deploy/compose/healthchecks/ollama.sh
  - deploy/compose/healthchecks/minio.sh
  - docs/guides/compose-ce.md

### Prompt C2 — Vault and Keycloak dev-mode bootstrap notes
- Objective: Provide minimal notes to bring Vault (dev) and Keycloak up for future phases.
- Inputs to assume:
  - Vault dev mode with in-memory storage; Keycloak in dev profile
- Constraints:
  - No seeding scripts; only documented manual steps/placeholders
- Acceptance criteria:
  - Steps documented to confirm service is reachable and auth UI loads
- Expected output artifacts:
  - docs/security/secrets-bootstrap.md
  - docs/guides/keycloak-dev.md

## D. Controller/API OpenAPI skeleton generation (placeholders only)

### Prompt D1 — Controller OpenAPI stub
- Objective: Draft OpenAPI 3.1 stub covering ADR-0010 endpoints with schemas as TODOs.
- Inputs to assume:
  - Endpoints: /tasks/route, /sessions, /approvals, /status, /profiles (proxy), /audit/ingest
- Constraints:
  - No implementation; schemas can be referenced as $ref placeholders
  - Include idempotency-key header, size limits notes
- Acceptance criteria:
  - openapi.yaml passes lint (spectral) with warnings allowed for TODOs
- Expected output artifacts:
  - docs/api/controller/openapi.yaml
  - docs/api/schemas/README.md (how to evolve schemas)

### Prompt D2 — AuditEvent and Profile bundle schemas (stubs)
- Objective: Provide JSON/YAML schemas reflecting ADR-0008 and ADR-0011.
- Inputs to assume:
  - AuditEvent fields: id, ts, tenantId, actor, action, target, result, redactions[], cost, traceId, hashPrev
  - Profile bundles signed with Ed25519
- Constraints:
  - Keep fields minimal; include comments for future constraints
- Acceptance criteria:
  - Schemas exist and are referenced by OpenAPI stub
- Expected output artifacts:
  - docs/audit/audit-event.schema.json
  - docs/policy/profile-bundle.schema.yaml
  - config/profiles/sample/marketing.yaml.sig (placeholder, non-functional)

### Prompt D3 — Metadata-only DB migration stubs
- Objective: Create initial SQL migration stubs for metadata tables only.
- Inputs to assume:
  - Tables: sessions_meta, tasks_meta, approvals_meta, audit_index
- Constraints:
  - No content columns; only IDs, timestamps, hashes, counts
- Acceptance criteria:
  - Files exist; annotated with TODOs for Phase 7
- Expected output artifacts:
  - db/migrations/metadata-only/0001_init.sql
  - db/README.md
