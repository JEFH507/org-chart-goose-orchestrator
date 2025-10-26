# Project TODO

Use this list for project-scoped tasks. Keep the global TODO for cross-project/system tasks.

- [ ] Initial project folder/structure setup
- [ ] Initial productdescription.md file (it defines the product, persona, value proposition, and general concepts at a client level perspective)
- [ ] Define scope and acceptance criteria through the Product Owner and the requirements.md file it will generate
- [ ] Initial project plan via the planner agent, and the result plan.mmd
- [ ] Define first milestone

## Possible future tasks (to be planned)
- [ ] Add additional architecture diagram variants (e.g., sequenceDiagram) to docs/architecture/architecture.html
- [ ] Create a static index page to navigate multiple diagrams in docs/architecture
- [ ] Draft ADR template and first ADRs (open-core split, privacy guard approach)
- [ ] Add roadmap.md and mvp.md under docs/architecture
- [ ] Flesh out config structure (profiles/, policies/, providers/, extensions/, recipes/)
- [ ] Add Makefile/justfile for common tasks (build docs, lint, test)
- [ ] Populate docs/guides with onboarding and admin guides
- [ ] Prepare example profiles and a sample cross-agent workflow under examples/
- [ ] Review and fill GOOSEREF.md for gooseV1.12.00 (commit/tag/date, PRs/issues, notes)
- [ ] Create technical-requirements.md via recipe and review against ADRs
- [ ] Use ADR Creator to record initial decisions (0001–0005 created)
- [ ] Add RUNNING_RECIPES.md runbook to repo (created) and keep updated
- [ ] After plan.mmd, run Project Manager to generate project-board.mmd and append tasks
- [ ] Before implementation, run Architect to scaffold feature branches

- [ ] Move current Goose session into goose-org-twin to consolidate context
- [ ] Validate .gooseignore patterns for this project and tighten if needed
- [ ] Draft a minimal controller HTTP API outline for handoff/approval stubs (endpoints, payloads, auth)

## OSS-first alignment (new)
- [ ] Community Edition (CE) docker-compose stack under ./deploy/ce-compose/
  - [ ] Keycloak (OIDC) with "known good" client config
  - [ ] Vault OSS (unseal/dev profile) + Transit (optionally wire to cloud KMS later)
  - [ ] Postgres (metadata) and MinIO (S3-compatible object store)
  - [ ] Ollama (local models for guard)
  - [ ] Optional: Prometheus, Loki, Grafana (disabled by default)
- [ ] Provider interfaces and reference adapters
  - [ ] Define interfaces: AuthProvider, SecretsProvider, StorageProvider, ModelProvider, BusProvider
  - [ ] Conformance tests for adapters; sample Keycloak, Vault, Postgres, MinIO, Ollama implementations
- [ ] Portability tooling
  - [ ] Export/import CLI for sessions/policies/recipes/audit (JSON/JSONL/TAR)
  - [ ] Migration guides CE ↔ SaaS
- [ ] Controller HTTP API definition
  - [ ] ADR: controller-http-api-mvp (endpoints, payloads, auth)
  - [ ] OpenAPI v1 stub for: /tasks/route, /sessions, /approvals, /status/{id}, /profiles/{role}, /audit/ingest
  - [ ] Minimal server stubs (handlers + idempotency + JWT)
- [ ] Docs/diagrams
  - [ ] CE vs SaaS mapping (table/diagram)
  - [ ] Update orchestrator_min diagram to show agent guard pre/post and OIDC flow
  - [ ] Architecture index: ensure links to MVP and Roadmap are present
- [ ] Recipes alignment
  - [ ] Ensure Technical Requirements, Architect, and Project Manager recipes reference OIDC SSO + Vault/KMS decisions
  - [ ] Add Makefile/justfile tasks for common runs (e.g., make trs, make adr, make plan)

- [ ] Decide novel interaction emphasis for Q2: Whiteboard-to-Workflow vs Voice/Meeting Approvals
- [ ] Identify 1–2 design partners for first paid pilot by end of Q1; draft SOW templates
