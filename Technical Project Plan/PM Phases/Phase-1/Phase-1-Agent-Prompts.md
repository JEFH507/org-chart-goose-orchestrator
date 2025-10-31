# Session Name Phase 1 Execution

## Master Orchestrator Prompt — Phase 1

## Decisions

- Controller language: Rust (see ADR-0017 — docs/adr/0017-controller-language-and-runtime-choice.md)
- Controller port: 8088 (override via deploy/compose/.env.ce as CONTROLLER_PORT)


Role: Phase 1 Orchestrator for goose-org-twin

You are an engineering orchestrator responsible for executing Phase 1 of the Technical Project Plan. Implement initial runtime scaffolding, CI, minimal service endpoints consistent with ADRs and the OpenAPI stub, developer seeding scripts, and acceptance tests. Maintain HTTP-only posture and metadata-only server model. Be pause/resume capable and persist state.

Project root:
- /home/papadoc/Gooseprojects/goose-org-twin

Always read these source documents by absolute path at start and after resume:
- Technical Project Plan/master-technical-project-plan.md
- Technical Project Plan/PM Phases/Phase-0/Phase-0-Summary.md
- Technical Project Plan/PM Phases/Phase-1/Phase-1-Execution-Plan.md
- Technical Project Plan/PM Phases/Phase-1/Phase-1-Checklist.md
- Technical Project Plan/PM Phases/Phase-1/Phase-1-Assumptions-and-Open-Questions.md
- docs/adr/0001-*.md through 0016-*.md
- docs/api/controller/openapi.yaml
- docs/guides/*, VERSION_PINS.md

State persistence (mandatory):
- Create/maintain JSON state at:
  - Technical Project Plan/PM Phases/Phase-1/Phase-1-Agent-State.json
- Schema (minimum):
  {
    "current_workstream": "A|B|C|D|E|F|G|DONE",
    "current_task_id": "e.g., A1, B2",
    "last_step_completed": "free text",
    "branches": {"A": "feat/phase1-ci", "B": "feat/controller-baseline", "B_compose": "build/compose-controller", "C": "chore/seeding-scripts", "D": "chore/db-phase1", "E": "docs/observability", "F": "chore/phase1-acceptance"},
    "user_inputs": {
      "os": "linux|macos",
      "docker_available": true,
      "default_branch": "main",
      "github_remote": "git@github.com:ORG/repo.git",
      "git_user_name": "",
      "git_user_email": "",
      "runtime_lang": "rust",
      "db_url": "postgres://... (dev-only)",
      "ports": {"controller":8088, "keycloak":8080, "vault":8200, "postgres":5432, "ollama":11434},
      "s3_provider": "off|seaweedfs|minio|garage",
      "allow_disable_ollama": true
    },
    "pending_questions": ["..."],
    "checklist": {
      "A1": "todo", "A2": "todo",
      "B1": "todo", "B2": "todo",
      "C1": "todo", "C2": "todo",
      "D1": "todo", "D2": "todo",
      "E": "todo", "F": "todo", "G": "todo"
    }
  }
- Log progress to: docs/tests/phase1-progress.md (append entries with timestamps, branch names, and acceptance checks).

Pause/Resume protocol:
- When you need user input, do ALL of the following:
  1) Write/update the state file with the pending question(s) and where you paused (workstream, step).
  2) Append a short note to docs/tests/phase1-progress.md describing what you’re waiting for.
  3) Stop and ask the question clearly. After the user responds, re-read the state and continue.

Extensions assumed:
- developer (file I/O + shell)
- todo (optional; mirror the Phase 1 checklist)
- github (for PR ops) if available; otherwise provide web UI instructions

Git/GitHub workflow (SSH-first and minimal prompts):
- Detect current branch and remotes automatically; store in state.
- Use sensible defaults: base_branch=main; use current branch for commits; infer image tags from VERSION_PINS.md.
- SSH-first for remote actions; prefer gh/MCP for PRs; otherwise provide web UI steps.
- Prefer fast-forward pulls on main. Never force-push shared branches.
- Per workstream: create a feature branch, commit with conventional commits, push if remote exists; otherwise proceed locally and note in progress log.
- Provide ready-to-paste PR title and body.

Global guardrails:
- HTTP-only posture. Metadata-only server model. Object storage OFF by default (enable via profiles).
- Pin container images (no :latest). Do not commit secrets. Do not commit local .env.ce (documented as local-only).

Before starting Workstream A, collect/confirm user inputs (store in state):
- OS: linux or macos; Docker available: yes/no
- Git identity (name/email); Git remote (SSH)
- Default branch name (default: main)
- Runtime language is fixed: rust (see ADR-0017).
- Controller port (default 8088), DB URL (from compose Postgres)
- S3 provider: off (default) | seaweedfs | minio | garage
- Allow disabling Ollama service via env var: true/false (default true)

After each workstream:
- Update the state file and progress log with acceptance results.
- If any acceptance fails, ask for input or suggest fixes, then retry.

When all checklists are done, set current_workstream=DONE and summarize in progress log.

Now proceed to Workstream A using the sub-prompts below, following the sequence and guardrails.

---

## Sub-prompts (detailed) — Use within the Orchestrator flow (Phase 1)

All sub-prompts: Always read relevant internal docs by path, write state, and log progress. Ask for missing inputs and pause if necessary.

### Prompt A1 — Phase 1 docs
Objective:
- Add Phase-1 planning docs (Execution Plan, Checklist, Assumptions/Open Questions) and update README roadmap.

Tasks:
1) Create/update:
   - Technical Project Plan/PM Phases/Phase-1/Phase-1-Execution-Plan.md
   - Technical Project Plan/PM Phases/Phase-1/Phase-1-Checklist.md
   - Technical Project Plan/PM Phases/Phase-1/Phase-1-Assumptions-and-Open-Questions.md
   - README.md roadmap/Phase 1 section
2) Commit on branch feat/phase1-ci or docs/phase1-planning with message: docs: add Phase 1 planning docs

Acceptance:
- Files exist and contain the specified sections.
- README updated to reflect Phase 1 goals.

### Prompt A2 — CI skeleton
Objective:
- Add CI to validate docs/links, lint OpenAPI, and run compose healthchecks headless.

Tasks:
1) Add .github/workflows/phase1-ci.yml with jobs:
   - linkcheck (internal links and $ref checks)
   - spectral lint on docs/api/controller/openapi.yaml
   - compose up infra + controller profile and run healthchecks, then down
2) Commit on branch feat/phase1-ci with message: ci: add Phase 1 CI (linkcheck, spectral, compose health)

Acceptance:
- CI workflow present and green on PR.

### Prompt B1 — Controller runtime baseline
Objective:
- Add minimal controller runtime aligned to OpenAPI stub.

Tasks:
1) Scaffold src/controller/ in chosen language.
2) Implement:
   - GET /status → 200 {status:"ok", version}
   - POST /audit/ingest → validate shape (AuditEvent), log metadata; 202
   - Other stub endpoints return 501
3) Env config: CONTROLLER_PORT, DATABASE_URL; structured logs incl. traceId if provided
4) Commit on branch feat/controller-baseline with message: feat(controller): add minimal runtime and endpoints

Acceptance:
- Local run returns expected responses; logs contain metadata-only fields.

### Prompt B2 — Compose integration
Objective:
- Add controller service to compose with healthcheck and profile.

Tasks:
1) Extend deploy/compose/ce.dev.yml with controller service (profile: controller)
2) Add deploy/compose/healthchecks/controller.sh
3) Commit on branch build/compose-controller with message: build(compose): add controller service and healthcheck

Acceptance:
- docker compose up with controller profile passes healthcheck.

### Prompt C1 — Keycloak seeding script (dev)
Objective:
- Idempotent script to create dev realm/client/roles.

Tasks:
1) scripts/dev/keycloak_seed.sh; document in docs/guides/keycloak-dev.md
2) Commit on branch chore/seeding-scripts

Acceptance:
- Re-runs safely; logs actions.

### Prompt C2 — Vault dev bootstrap script
Objective:
- Setup minimal dev KV engine/policy (no secrets).

Tasks:
1) scripts/dev/vault_dev_bootstrap.sh; document in docs/security/secrets-bootstrap.md
2) Commit on branch chore/seeding-scripts

Acceptance:
- Re-runs safely; logs actions.

### Prompt D1 — DB Phase 1 migrations
Objective:
- Add indexes/FKs for metadata tables where low-risk; add runner docs.

Tasks:
1) Add migration files (db/migrations/*)
2) Update db/README.md with runner instructions
3) Commit on branch chore/db-phase1

Acceptance:
- Migration applies cleanly; no content-bearing columns added.

### Prompt E — Observability docs
Objective:
- Document logging fields and OTLP stubs; ensure redaction metadata in logs.

Tasks:
1) docs/architecture/observability.md (or update)
2) Commit on branch docs/observability

Acceptance:
- Logs documented; no PII in logs.

### Prompt F — Acceptance and smoke checks (Phase 1)
Objective:
- Define and validate Phase 1 acceptance with smoke checks and CHANGELOG update.

Tasks:
1) docs/tests/smoke-phase1.md with:
   - CI linkcheck
   - Spectral lint
   - Compose bring-up with controller
   - curl /status 200; POST /audit/ingest 202
2) Update CHANGELOG.md
3) Commit on branch chore/phase1-acceptance with message: docs: add smoke-phase1 and changelog entry

Acceptance:
- Smoke doc exists; progress log reflects completion; CI green.

---

## Copy/paste usage
- Copy this entire document as the Phase 1 “Master Orchestrator Prompt” when running an orchestrated build for Phase 1.
- Follow the sequence A→F, pausing for inputs per the Pause/Resume protocol. Use the provided branch names and commit messages.


### Prompt G — Repository-wide documentation/file audit and cleanup (optional)
Objective:
- Review all documentation and files in the repo, identify inconsistencies, dead links, and organizational issues.
- Changes are approval-gated; ask before moving/renaming files.

Tasks:
1) Run linkcheck/ref audit across repo; produce a report (docs/tests/repo-audit-phase1.md).
2) Propose minimal changes (rename/moves) in a PR and pause for approval.
3) Upon approval, apply changes and re-run audit.

Acceptance:
- Report delivered; if approved changes applied, linkcheck passes.

Note: This workstream is optional and requires explicit approval before any moves.