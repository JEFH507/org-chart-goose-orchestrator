# Phase 0 Agent Prompts — Orchestrated, Resume-Capable (planning + scaffolding only)

This document contains execution-ready prompts for Goose agents to complete Phase 0. It includes:
- Operator Guide (how to run in Goose Desktop, new vs reuse session, Git/GitHub basics).
- A Master Orchestrator Prompt that coordinates all Phase 0 workstreams with pause/resume and state persistence.
- Detailed sub-prompts for each workstream (A–F, optional G) with file paths, acceptance criteria, and exact references to internal documentation so prompts do not rely on chat context alone.

Scope guardrails for agents: No application/service runtime code; only repo scaffolding, docs, configuration, and placeholders per Phase 0. HTTP-only constraints; metadata-only posture; optional S3 storage OFF by default.

---

## Operator Guide — How to run these prompts in Goose

Prerequisites you handle once:
- You have this repository checked out locally at:
  - /home/papadoc/Gooseprojects/goose-org-twin (Linux path used throughout)
- Docker is installed and can run (Docker Desktop or Docker Engine).
- Git and a GitHub account available. If you plan to push to GitHub, have a Personal Access Token (classic, repo scope) ready.

Recommended extensions for the agent session:
- developer (to read/write files and run shell commands)
- todo (for simple task tracking; optional but recommended)

New session vs reuse:
- New session: Recommended for Phase 0. Start a new Goose session and paste the “Master Orchestrator Prompt” in Copy Block A. The prompt instructs the agent to read all required files by path.
- Reuse session: If you return later, paste the “Resume Prompt” in Copy Block B (see below in this Operator Guide). It reads a state file from disk and continues.

### Copy Block B — Resume Prompt (paste when returning in a later session)

<!-- BEGIN COPY BLOCK B: Resume Prompt (copy this block when resuming in a later session) -->

You are resuming Phase 0 orchestration for goose-org-twin.
- Read state from: Technical Project Plan/PM Phases/Phase-0/Phase-0-Agent-State.json
- Read and re-ingest the same project documents listed in the Master Orchestrator Prompt.
- Summarize: current_workstream, current_task_id, last_step_completed, and pending_questions.
- If pending_questions exist, ask them and wait for answers; then continue.
- Otherwise, continue with the next step in the defined sequence (A → B → C → D → E → F → optional G).
- Keep using the same guardrails, state persistence, and progress logging.

<!-- END COPY BLOCK B -->


Git/GitHub basics for this repo:
- If not initialized (unlikely):
  - git init
  - git branch -M main
- Configure your identity (once per machine):
  - git config user.name "Your Name"
  - git config user.email "you@example.com"
- Check/add remote:
  - git remote -v
  - If missing: git remote add origin git@github.com:YOURORG/goose-org-twin.git
- Typical Phase 0 flow (the agent will guide you):
  - Create feature branches per workstream (e.g., feat/phase0-compose).
  - Make commits with conventional commit messages.
  - Push branch: git push -u origin feat/phase0-compose
  - Open PR on GitHub; the agent can draft the body text and provide a link.

Git Remote Policy — SSH-first with GNOME askpass:
- Ask only for values that are missing and required for the requested action.
- Propose sensible defaults (e.g., base_branch=main; use current branch; infer tags from VERSION_PINS.md).
- Prefer SSH for remotes. If the SSH key isn’t loaded, prefer GNOME askpass to avoid interactive loops:
  ```bash
  export DISPLAY=${DISPLAY:-:0}
  export SSH_ASKPASS_REQUIRE=force
  SSH_ASKPASS="$(command -v ssh-askpass-gnome || command -v ssh-askpass || true)"
  if [ -n "$SSH_ASKPASS" ]; then setsid -w ssh-add ~/.ssh/id_ed25519 < /dev/null; fi
  ```
- Prefer fast-forward pulls on main. Never force-push shared branches.
- If gh (GitHub CLI) isn’t installed for PR/release actions, explain install or offer pure git/website alternatives.

Where agent state and progress will be stored:
- JSON state: Technical Project Plan/PM Phases/Phase-0/Phase-0-Agent-State.json
- Human-readable log: docs/tests/phase0-progress.md
- Optional: todo list via the todo extension (mirrors the Phase 0 checklist).

Safety and scope:
- Phase 0 does not include service runtime code. Agents must not implement server logic or agents; only scaffolding, docs, configs, and placeholders.
- No secrets are committed. No production TLS setup in Phase 0.
- Object storage is OFF by default. SeaweedFS is the CE default option (opt-in), MinIO/Garage optional.

---

<!-- BEGIN COPY BLOCK A: Master Orchestrator + Sub-prompts A–F (copy this block into a new Goose session) -->

## Master Orchestrator Prompt (paste into a new Goose session)

Role: Phase 0 Orchestrator for goose-org-twin

You are an engineering orchestrator responsible for executing Phase 0 of the Technical Project Plan. You will create scaffolding, documentation, configuration, and placeholders only—no runtime service code—aligned with ADRs and Goose v1.12. You must be pause/resume capable and persist state.

Project root:
- /home/papadoc/Gooseprojects/goose-org-twin

Always read these source documents by absolute path at start and after resume:
- Technical Project Plan/master-technical-project-plan.md
- Technical Project Plan/PM Phases/Phase-0/Phase-0-Execution-Plan.md
- Technical Project Plan/PM Phases/Phase-0/Phase-0-Checklist.md
- Technical Project Plan/PM Phases/Phase-0/Phase-0-Assumptions-and-Open-Questions.md
- Technical Project Plan/PM Phases/Phase-0/Phase-0-Repo-Structure-Evaluation.md
- docs/adr/0001-*.md through 0016-*.md (all ADRs 0001–0016)
- docs/guides/* (if present), docs/api/*, VERSION_PINS.md (if present)

State persistence (mandatory):
- Create/maintain JSON state at:
  - Technical Project Plan/PM Phases/Phase-0/Phase-0-Agent-State.json
- Schema (minimum):
  {
    "current_workstream": "A|B|C|D|E|F|G|DONE",
    "current_task_id": "e.g., A1, B2",
    "last_step_completed": "free text",
    "branches": {"A": "feat/phase0-repo-hygiene", ...},
    "user_inputs": {
      "os": "linux|macos",
      "docker_available": true,
      "default_branch": "main",
      "github_remote": "git@github.com:ORG/goose-org-twin.git",
      "git_user_name": "",
      "git_user_email": "",
      "s3_provider": "off|seaweedfs|minio|garage",
      "ports": {"keycloak":8080,"vault":8200,"postgres":5432,"ollama":11434,
                 "seaweed_s3":8333,"seaweed_master":9333,"seaweed_filer":8081,
                 "minio_api":9000,"minio_console":9001},
      "allow_disable_ollama": true
    },
    "pending_questions": ["..."],
    "checklist": {
      "A1": "todo|done", "A2": "todo|done", "B1": "todo|done", "B2": "todo|done",
      "C1": "todo|done", "C2": "todo|done", "D1": "todo|done", "D2": "todo|done",
      "D3": "todo|done", "E": "todo|done", "F": "todo|done", "G": "todo|done"
    }
  }
- Log progress to: docs/tests/phase0-progress.md (append entries with timestamps, branch names, and acceptance checks).

Pause/Resume protocol:
- When you need user input, do ALL of the following:
  1) Write/update the state file with the pending question(s) and where you paused (workstream, step).
  2) Append a short note to docs/tests/phase0-progress.md describing what you’re waiting for.
  3) Stop and ask the question clearly. After the user responds, re-read the state and continue.

Extensions assumed:
- developer (file I/O + shell)
- todo (optional; mirror the Phase 0 checklist)

Git/GitHub workflow (SSH-first and minimal prompts):
- Detect current branch and remotes automatically; store in state. Ask only for missing and required values.
- Use sensible defaults: base_branch=main; use current branch for commits; infer image tags from VERSION_PINS.md.
- SSH-first for remote actions; if SSH key is not loaded, prefer GNOME askpass (see Operator Guide snippet). Avoid loops.
- Prefer fast-forward pulls on main. Never force-push shared branches.
- If gh (GitHub CLI) is not installed, explain install or provide alternatives (pure git and web UI steps) when opening PRs.
- Per workstream: create a feature branch, commit with conventional commits, push if remote exists; otherwise proceed locally and note in progress log.
- Provide ready-to-paste PR title and body.

Global guardrails:
- No service runtime code (Controller, Directory/Policy, Identity Gateway, Mesh). Docs/configs/placeholders only.
- HTTP-only posture. Metadata-only server model. Object storage OFF by default.
- Pin container images (no :latest). Do not commit secrets. Do not commit local .env.ce (documented as local-only).

Execution sequence and sub-prompts to run:
- Workstream A: Repo hygiene and conventions
  - Run Prompt A1, then Prompt A2.
- Workstream B: Developer environment bootstrap
  - Run Prompt B1, then Prompt B2.
- Workstream C: CE docker-compose baseline (infra only)
  - Run Prompt C1, then Prompt C2.
- Workstream D: Placeholder APIs/schemas/migrations
  - Run Prompt D1, then D2, then D3.
- Workstream E: Secrets bootstrap docs
- Workstream F: Acceptance criteria and smoke checks
- Optional Workstream G: Repository organization and cleanup (requires user approval)

Before starting Workstream A, collect/confirm user inputs (store in state):
- OS: linux or macos
- Docker available: yes/no
- Git identity (name/email)
- Git remote (SSH or HTTPS URL) or “skip push”
- Default branch name (default: main)
- S3 provider: off (default) | seaweedfs | minio | garage
- Port overrides, if any (otherwise use defaults listed in state schema)
- Allow disabling Ollama service via env var: true/false (default true)

After each workstream:
- Update the state file and progress log with acceptance results.
- If any acceptance fails, ask for input or suggest fixes, then retry.

When all checklists are done, set current_workstream=DONE and summarize in progress log.

Now proceed to Workstream A using the sub-prompts below, following the sequence and guardrails.

---

## Sub-prompts (detailed) — Use within the Orchestrator flow

All sub-prompts: Always read relevant internal docs by path, write state, and log progress. Ask for missing inputs and pause if necessary.

### Prompt A1 — Repository hygiene and conventions
Objective:
- Create repository hygiene artifacts and conventions aligned with ADRs and Goose v1.12 practices.

Inputs and references:
- Read:
  - Technical Project Plan/master-technical-project-plan.md
  - docs/adr/0001-*.md through 0016-*.md
  - README.md for current structure and links
- Use these directories (create if missing):
  - .github/ISSUE_TEMPLATE/
  - docs/conventions/

User inputs (ask if missing):
- Git user.name and user.email (configure if not set)
- Default branch name (default: main)

Tasks:
1) Create/update:
   - .github/PULL_REQUEST_TEMPLATE.md (sections: Summary, Changes, ADR references, Testing, Checklist, Screenshots/Links)
   - .github/ISSUE_TEMPLATE/bug.md (front-matter for GitHub issue templates; fields: Summary, Steps to Reproduce, Expected, Actual, Logs, Environment)
   - .github/ISSUE_TEMPLATE/feature.md (fields: Problem, Proposal, Acceptance Criteria, Out of Scope, ADR links)
   - CONTRIBUTING.md (link ADRs, conventional commits, branching strategy, PR process, DCO or CLA if relevant)
   - docs/conventions/commit-style.md (conventional commit types with examples: feat, fix, docs, chore, build, ci; scope guidance)
   - CODEOWNERS (optional stub pointing to core maintainers)
2) Ensure .gooseignore contains deploy/compose/.env.ce and similar local files. If absent, append.
3) Update README.md: add a “Repository Info” section (append-only) containing:
   - Local path: /home/papadoc/Gooseprojects/goose-org-twin
   - GitHub URL (from state.user_inputs.github_remote, converted to https if SSH): https://github.com/ORG/goose-org-twin
4) Commit on branch feat/phase0-repo-hygiene with message: docs: add repo hygiene templates and conventions

Acceptance:
- Files exist and contain the specified sections.
- CONTRIBUTING.md references ADR directory and commit conventions doc.
- .gooseignore includes local-only files and does not remove existing entries.
- README.md contains “Repository Info” with local path and GitHub URL.

Output artifacts:
- .github/PULL_REQUEST_TEMPLATE.md
- .github/ISSUE_TEMPLATE/bug.md
- .github/ISSUE_TEMPLATE/feature.md
- CONTRIBUTING.md
- docs/conventions/commit-style.md
- CODEOWNERS (optional)

Logging:
- Append summary of files created/updated and commit hash to docs/tests/phase0-progress.md

---

### Prompt A2 — Scripts skeleton (placeholders)
Objective:
- Add placeholders for dev scripts without implementation details (or only minimal usage comments), keeping them executable.

Inputs and references:
- Read scripts/ directory to avoid overwrites of existing files.

Tasks:
1) Create placeholders with shebang and TODO notes:
   - scripts/dev/bootstrap.sh
   - scripts/dev/checks.sh
   - scripts/dev/health.sh
2) Make executable (chmod +x).
3) Each script header should include: purpose, Phase 0 scope note (placeholder), and links to relevant docs (e.g., docs/guides/dev-setup.md).
4) Commit on branch feat/phase0-repo-hygiene or a new branch feat/phase0-scripts-skeleton with message: chore: add dev script placeholders

Acceptance:
- Scripts exist, are executable, and contain clear TODOs and documentation links.

---

### Prompt B1 — Developer setup guide
Objective:
- Author dev-setup guide for Linux/macOS including prerequisites, ports map, port override mechanism, and smoke steps.

Inputs and references:
- Read:
  - Technical Project Plan/PM Phases/Phase-0/Phase-0-Execution-Plan.md
  - docs/guides/ports.md (create in B2 if missing)
  - scripts/dev/preflight_ports.sh (if present) or note future addition
- Default ports to include: Keycloak 8080, Vault 8200, Postgres 5432, Ollama 11434, SeaweedFS 8333/9333/8081, MinIO 9000/9001

Tasks:
1) Create docs/guides/dev-setup.md covering:
   - Prerequisites (Docker, Git), OS support (Linux/macOS), CPU-only expectations
   - Repo layout overview (services/, deploy/compose/, docs/, config/, db/, scripts/)
   - Ports table and how to override via deploy/compose/.env.ce
   - How to run Phase 0 smoke checks
   - Known issues and troubleshooting
2) Create deploy/compose/.env.ce.example with default values and comments; instruct users to copy to .env.ce (not committed).
3) Commit on branch chore/docs-dev-setup with message: docs: developer setup guide and env example

Acceptance:
- docs/guides/dev-setup.md exists, includes ports, overrides, and smoke instructions.
- .env.ce.example exists with defaults and comments, and .gooseignore excludes .env.ce

---

### Prompt B2 — Version pinning and ports registry
Objective:
- Establish VERSION_PINS.md and docs/guides/ports.md as source of truth for versions and ports.

Tasks:
1) Create/Update VERSION_PINS.md with pinned versions (no :latest) for: Keycloak, Vault, Postgres, Ollama, SeaweedFS, MinIO, Garage (optional), plus notes on guard model tags (do not bundle weights).
2) Create docs/guides/ports.md listing default ports and override strategy, referencing deploy/compose/.env.ce.
3) Cross-link these docs from dev-setup.md.
4) Commit on branch chore/version-pins with message: docs: add VERSION_PINS and ports registry

Acceptance:
- VERSION_PINS.md present with explicit tags and rationale.
- docs/guides/ports.md present and referenced by dev-setup.md.

---

### Prompt C1 — CE docker-compose baseline with healthchecks (infra only)
Objective:
- Create deploy/compose/ce.dev.yml for Keycloak, Vault, Postgres, Ollama; optional S3-compatible storage OFF by default; add simple healthchecks and env overrides.

Inputs and references:
- Read VERSION_PINS.md for image tags.
- Object storage policy: ADR-0014 (SeaweedFS default option; MinIO/Garage optional; OFF by default in Phase 0).
- Ensure .gooseignore excludes deploy/compose/.env.ce from commits.

User inputs (ask if missing):
- S3 provider: off|seaweedfs|minio|garage (default: off)
- Port overrides (if any)
- Allow disabling Ollama service (default: true)

Tasks:
1) Create deploy/compose/ce.dev.yml with services:
   - keycloak (dev profile), vault (dev mode), postgres, ollama
   - Optional profiles or env flags to enable SeaweedFS/MinIO/Garage
   - Use ${...} to source ports from .env.ce
   - Add healthcheck commands via curl/nc
2) Create healthcheck scripts in deploy/compose/healthchecks/: keycloak.sh, vault.sh, postgres.sh, ollama.sh, minio.sh (for optional S3)
3) Update docs/guides/compose-ce.md with instructions to:
   - Copy .env.ce.example to .env.ce and adjust ports
   - Enable/disable S3 and/or Ollama via env
   - Run: docker compose -f deploy/compose/ce.dev.yml up -d
   - Verify healthchecks
4) Commit on branch feat/phase0-compose with message: build: add CE compose baseline and healthchecks

Acceptance:
- docker compose up runs infra only; health scripts succeed
- S3 is OFF by default; enabling is documented
- Ports are driven by .env.ce

---

### Prompt C2 — Vault and Keycloak dev-mode bootstrap notes
Objective:
- Provide minimal notes to confirm Vault (dev) and Keycloak (dev) run and are reachable.

Tasks:
1) Create docs/security/secrets-bootstrap.md covering Vault dev mode, initial root token output behavior, and reminding not to store secrets.
2) Create docs/guides/keycloak-dev.md with steps to access Keycloak admin UI, dev realm setup pointers (placeholders), and links to future seeding scripts (Phase 1).
3) Commit on branch docs/phase0-secrets-bootstrap with message: docs: add Vault and Keycloak dev-mode bootstrap notes

Acceptance:
- Both docs exist and clearly state Phase 0 scope and dev-only posture.

---

### Prompt D1 — Controller OpenAPI stub
Objective:
- Draft OpenAPI 3.1 stub covering ADR-0010 endpoints; schemas as TODOs or $ref placeholders. Lint with Spectral (warn-only).

Inputs and references:
- Read ADR-0010 and ADR-0012 (minimal server surface, metadata-only).
- Ensure .spectral.yaml exists (Phase 0 warn-only). If missing, create minimal ruleset.

Tasks:
1) Create docs/api/controller/openapi.yaml with:
   - Info: title, version 0.0.1, contact
   - Servers: http://localhost:${PORT}
   - Paths: /tasks/route, /sessions, /approvals, /status, /profiles (proxy), /audit/ingest
   - Components: securitySchemes (bearer, optional X-Secret-Key), headers (Idempotency-Key), common error schema
   - TODO schemas via $ref to docs/api/schemas/* to be filled later
   - Notes on payload size limits and idempotency
2) Create docs/api/schemas/README.md explaining schema evolution and references.
3) Add scripts/dev/openapi_lint.sh to run spectral (warn-only) and document usage in docs/api/linting.md (if not present).
4) Commit on branch feat/openapi-stub with message: docs(api): add controller OpenAPI stub and schema placeholders

Acceptance:
- openapi.yaml exists and lints with Spectral (warnings allowed for TODOs)
- schemas README present and linked

---

### Prompt D2 — AuditEvent and Profile bundle schemas (stubs)
Objective:
- Provide JSON/YAML schemas reflecting ADR-0008 (audit/redaction) and ADR-0011 (signed profiles), referenced by OpenAPI (as applicable).

Tasks:
1) Create docs/audit/audit-event.schema.json with fields:
   - id (uuid), ts (RFC3339), tenantId, actor{id,type,role}, action, target{id,type}, result{status,reason}, redactions[{ruleId,field,hash}], cost{inputTokens,outputTokens,currency}, traceId, hashPrev
2) Create docs/policy/profile-bundle.schema.yaml capturing:
   - bundle metadata (id, version), roles[], policies[], signatures[], publicKeys[] (Ed25519), createdAt, expiresAt; include comments for constraints
3) Create placeholder: config/profiles/sample/marketing.yaml.sig (non-functional demo filename to signal signing concept; content may be TODO)
4) Commit on branch feat/schemas-stubs with message: docs: add audit event and profile bundle schema stubs

Acceptance:
- Both schemas exist and are referenced (at least documented) by OpenAPI stub/README
- Placeholder signed profile filename exists

---

### Prompt D3 — Metadata-only DB migration stubs
Objective:
- Create initial SQL migration stubs for metadata tables only, no content storage.

Tasks:
1) Create db/migrations/metadata-only/0001_init.sql with CREATE TABLE statements for:
   - sessions_meta (id, tenant_id, created_at, updated_at, actor_id, trace_id)
   - tasks_meta (id, session_id, type, status, created_at, updated_at, cost_json, hash_prev)
   - approvals_meta (id, task_id, approver_role, status, decided_at)
   - audit_index (id, ts, tenant_id, actor_id, action, target_id, redactions_count, trace_id)
   Add TODO comments for indexes and FKs to be finalized in Phase 7.
2) Create db/README.md describing metadata-only posture, no transcripts, and future evolution.
3) Commit on branch feat/db-metadata-stubs with message: chore(db): add metadata-only migration stubs and README

Acceptance:
- SQL file and README exist; no content-bearing columns present

---

### Prompt E — Secrets bootstrap docs (consolidation)
Objective:
- Ensure secrets-related docs align and cross-reference: Vault dev-mode, OS keychain local notes, profile signing keys (ADR-0016).

Tasks:
1) Cross-link docs/security/secrets-bootstrap.md with docs/security/profile-bundle-signing.md (if present) and ADR-0016.
2) Update docs/guides/dev-setup.md to reference secrets bootstrap doc.
3) Commit on branch docs/phase0-secrets-xref with message: docs: cross-link secrets bootstrap and signing key policy

Acceptance:
- Cross-references exist; documents align with ADR-0016.

---

### Prompt F — Acceptance and smoke checks
Objective:
- Define and validate Phase 0 acceptance criteria with smoke checks and CHANGELOG update.

Tasks:
1) Create or update docs/tests/smoke-phase0.md to include:
   - Preflight ports check
   - Compose bring-up and health verification commands
   - OpenAPI lint command and expected outcome
   - Presence checks for schemas and migrations
2) Run the checks (documented). If checks are manual in Phase 0, write the exact commands without executing long-running services unless explicitly approved by user.
3) Add CHANGELOG.md entry under Unreleased or 0.0.0 noting Phase 0 planning and scaffolding completed, link to PRs.
4) Commit on branch chore/phase0-acceptance with message: docs: add smoke-phase0 and changelog entry

Acceptance:
- Smoke doc exists and is complete; CHANGELOG updated; progress log reflects M0.1–M0.4 status

---

<!-- END COPY BLOCK A -->

### Prompt G (optional) — Repository organization and cleanup (non-destructive, approval-gated)
Objective:
- Produce and, if approved, apply a minimal, non-destructive reorganization to align root files with the proposed repo layout, without breaking references. All moves require explicit user approval.

Inputs and references:
- Read:
  - Technical Project Plan/PM Phases/Phase-0/Phase-0-Repo-Structure-Evaluation.md
  - README.md
  - Current file tree (list with developer extension)

Tasks:
1) Inventory: Generate a file tree of the repo (top 2–3 levels). Identify candidates at repo root that should live under docs/product/, Technical Project Plan/, docs/architecture/, docs/guides/, or deploy/.
2) Proposal: Write a proposal to Technical Project Plan/PM Phases/Phase-0-Reorg-Proposal.md including:
   - Current vs target path table
   - Rationale per move
   - Link update plan (search-and-replace targets)
   - Risk assessment (links, external references)
3) Pause and request approval. Record pending question in Phase-0-Agent-State.json and phase0-progress.md.
4) If approved, apply moves with git mv only for low-risk items (e.g., productdescription.md → docs/product/productdescription.md) and immediately update all internal links that reference moved files.
5) Re-run link checks (grep/rg for old paths) and update remaining references.
6) Commit on branch chore/phase0-reorg with message: chore(docs): non-destructive repo organization (approved)

Acceptance:
- Reorg proposal exists and is approved before moves.
- Only approved moves are performed. Links updated. No broken references detected in quick scans.

---

## Notes for agents about internal documentation
- When referencing decisions, always link or cite the exact ADR path (docs/adr/00XX-*.md).
- For object storage policy, cite ADR-0014 and docs/guides/object-storage.md.
- For guard model policy, cite ADR-0015 and docs/guides/guard-model-selection.md.
- For profile signing keys, cite ADR-0016 and docs/security/profile-bundle-signing.md.
- For OpenAPI linting process, cite docs/api/linting.md and .spectral.yaml.

## Out-of-scope reminders (Phase 0)
- No implementation of services (controller, directory-policy, identity-gateway, mesh, guard logic).
- No message bus or gRPC; HTTP-only only.
- No server-side content storage; metadata-only model.
- No production TLS automation.

## What to expect after running these prompts
- Feature branches for each workstream, each with commits adding scaffolding/docs/configs.
- A CE compose file (infra only) that can be brought up and verified by healthchecks.
- OpenAPI stub and schema placeholders that lint with Spectral (warn-only).
- A reproducible developer setup and acceptance smoke guide.
- A persistent state file to safely pause/resume work at any point.
- Optional: approved, minimal repo reorganization with updated links.
