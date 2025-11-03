# Phase X — [Phase Name] ([Size: S/M/L]) — Prompt Template

**Purpose:** [One sentence describing what this phase delivers and why it matters for MVP]

**Builds on:** Phase X-1 artifacts and decisions

---

## How to Use This Prompt

### Starting a New Session (First Time)
Copy the entire "Master Orchestrator Prompt" section below and paste it into a new Goose session.

### Resuming Work (Returning Later)
Copy the "Resume Prompt" section below and paste it into Goose. It will read your state and continue where you left off.

---

## Resume Prompt — Copy this block when resuming Phase X

```markdown
You are resuming Phase X orchestration for goose-org-twin.

**Context:**
- Phase: X — [Phase Name] ([Size])
- Repository: /home/papadoc/Gooseprojects/goose-org-twin

**Required Actions:**
1. Read state from: `Technical Project Plan/PM Phases/Phase-X/Phase-X-Agent-State.json`
2. Read last progress entry from: `docs/tests/phase[X]-progress.md`
3. Re-read authoritative documents:
   - `Technical Project Plan/master-technical-project-plan.md`
   - `Technical Project Plan/PM Phases/Phase-X/Phase-X-Agent-Prompts.md` (this file)
   - `Technical Project Plan/PM Phases/Phase-X/Phase-X-Checklist.md`
   - `Technical Project Plan/PM Phases/Phase-X/Phase-X-Execution-Plan.md` (if present)
   - Relevant ADRs: [list ADR numbers/names]

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
- Keep CI stable; run acceptance tests locally/in CI as appropriate
- Update state JSON and progress log after each milestone
```

---

## Master Orchestrator Prompt — Copy this block for a new session

**Role:** Phase X Orchestrator for goose-org-twin

You are an engineering orchestrator responsible for executing Phase X of the Technical Project Plan. [Brief description of what you'll implement/deliver]. Maintain HTTP-only posture and metadata-only server model. Be pause/resume capable and persist state.

### Project Context

**Project root:** `/home/papadoc/Gooseprojects/goose-org-twin`

**Always read these source documents by absolute path at start and after resume:**
- `Technical Project Plan/master-technical-project-plan.md`
- `Technical Project Plan/PM Phases/Phase-X/Phase-X-Execution-Plan.md`
- `Technical Project Plan/PM Phases/Phase-X/Phase-X-Checklist.md`
- `Technical Project Plan/PM Phases/Phase-X/Phase-X-Assumptions-and-Open-Questions.md` (if present)
- Prior phase summaries:
  - `Technical Project Plan/PM Phases/Phase-0/Phase-0-Summary.md`
  - `Technical Project Plan/PM Phases/Phase-1/Phase-1-Completion-Summary.md`
  - [Add other completed phases]
- Relevant ADRs: `docs/adr/00XX-*.md` [list specific numbers]
- `docs/guides/*`, `VERSION_PINS.md`, `docs/api/controller/openapi.yaml`

### State Persistence (Mandatory)

**Create/maintain JSON state at:**
- `Technical Project Plan/PM Phases/Phase-X/Phase-X-Agent-State.json`

**Schema (minimum):**
```json
{
  "current_workstream": "A|B|C|...|DONE",
  "current_task_id": "e.g., A1, B2",
  "last_step_completed": "free text",
  "branches": {
    "A": "feat/phaseX-workstream-a",
    "B": "feat/phaseX-workstream-b"
  },
  "user_inputs": {
    "os": "linux|macos",
    "docker_available": true,
    "default_branch": "main",
    "github_remote": "git@github.com:JEFH507/org-chart-goose-orchestrator.git",
    "git_user_name": "Javier",
    "git_user_email": "132608441+JEFH507@users.noreply.github.com",
    "runtime_lang": "rust",
    "controller_port": 8088,
    "db_url": "postgresql://postgres:postgres@localhost:5432/postgres",
    "[phase_specific_inputs]": "..."
  },
  "pending_questions": ["..."],
  "checklist": {
    "A1": "todo|in-progress|done",
    "A2": "todo|in-progress|done",
    "B1": "todo|in-progress|done"
  }
}
```

**Log progress to:** `docs/tests/phase[X]-progress.md` (append entries with timestamps, branch names, and acceptance checks)

### Pause/Resume Protocol

When you need user input, do ALL of the following:
1. Write/update the state file with the pending question(s) and where you paused (workstream, step)
2. Append a short note to `docs/tests/phase[X]-progress.md` describing what you're waiting for
3. Stop and ask the question clearly. After the user responds, re-read the state and continue

### Extensions & Tools

**Assumed available:**
- `developer` (file I/O + shell)
- `todo` (optional; mirror the phase checklist)
- `github` (for PR ops) if available; otherwise provide web UI instructions

### Git/GitHub Workflow (SSH-first, minimal prompts)

**Policy:**
- Detect current branch and remotes automatically; store in state
- Use sensible defaults:
  - base_branch = main
  - Use current branch for commits
  - Infer image tags from VERSION_PINS.md
- SSH-first for remote actions; prefer GNOME askpass (see .goosehints for snippet)
- Prefer fast-forward pulls on main. **Never force-push shared branches**
- If `gh` (GitHub CLI) not available, provide web UI steps

**Per workstream:**
- Create a feature branch (naming: `feat/phaseX-workstream-name` or `docs/phaseX-topic`)
- Commit with conventional commits (feat/fix/docs/chore/build/ci)
- Push if remote exists; otherwise proceed locally and note in progress log
- Provide ready-to-paste PR title and body

### Global Guardrails

**DO NOT VIOLATE:**
- HTTP-only posture (no message bus in MVP)
- Metadata-only server model (no PII/content persistence)
- Object storage OFF by default (enable via profiles if needed)
- Pin container images (no `:latest`)
- Do not commit secrets
- Do not commit local `.env.ce` (documented as local-only)

### Before Starting Workstream A

**Collect/confirm user inputs (store in state):**
- OS: linux or macos
- Docker available: yes/no
- Git identity (name/email)
- Git remote (SSH URL) or "skip push"
- Default branch name (default: main)
- [Phase-specific inputs, e.g., controller port, DB URL, etc.]

### Execution Sequence

**Workstreams and sub-prompts to run:**
- **Workstream A:** [Name]
  - Run Prompt A1, then Prompt A2
- **Workstream B:** [Name]
  - Run Prompt B1, then Prompt B2
- **Workstream C:** [Name]
  - [tasks]
- [Add more as needed]

**After each workstream:**
- Update the state file and progress log with acceptance results
- If any acceptance fails, ask for input or suggest fixes, then retry

**When all checklists are done:**
- Set `current_workstream=DONE`
- Write completion summary to `Technical Project Plan/PM Phases/Phase-X/Phase-X-Completion-Summary.md`
- Update progress log with final status
- Suggest next steps (e.g., "Ready to start Phase X+1")

---

## Sub-Prompts (Detailed) — Use within the Orchestrator flow

All sub-prompts: Always read relevant internal docs by path, write state, and log progress. Ask for missing inputs and pause if necessary.

### Prompt A1 — [Workstream A Task 1 Name]

**Objective:**
[One sentence describing what this task accomplishes]

**Inputs and references:**
- Read: [list files/ADRs to read]
- User inputs (ask if missing): [list]

**Tasks:**
1. [Task step 1]
2. [Task step 2]
3. Commit on branch `[branch-name]` with message: `[conventional-commit-message]`

**Acceptance:**
- [Acceptance criterion 1]
- [Acceptance criterion 2]

**Output artifacts:**
- [file path 1]
- [file path 2]

**Logging:**
- Append summary of files created/updated and commit hash to `docs/tests/phase[X]-progress.md`

---

### Prompt A2 — [Workstream A Task 2 Name]

[Same structure as A1]

---

### Prompt B1 — [Workstream B Task 1 Name]

[Same structure]

---

[Continue for all workstreams and tasks]

---

## Notes for Agents About Internal Documentation

- When referencing decisions, always link or cite the exact ADR path (`docs/adr/00XX-*.md`)
- For [topic], cite ADR-00XX and `docs/guides/[topic].md`
- For [another topic], cite ADR-00YY and `docs/[area]/[file].md`

## Out-of-Scope Reminders (Phase X)

- [What NOT to implement in this phase]
- [What's deferred to later phases]

## What to Expect After Running These Prompts

- Feature branches for each workstream, each with commits adding [deliverables]
- [Specific artifact 1 that can be verified]
- [Specific artifact 2 that can be demonstrated]
- A persistent state file to safely pause/resume work at any point
- Optional: [other outcomes]

---

## Template Usage Instructions (For Phase Authors)

1. **Copy this template** to `Technical Project Plan/PM Phases/Phase-X/Phase-X-Agent-Prompts.md`
2. **Replace all placeholders:**
   - `[Phase Name]` → e.g., "Privacy Guard", "Controller API + Agent Mesh"
   - `[Size: S/M/L]` → S (≤2d), M (3–5d), L (1–2w), XL (>2w)
   - `X` → actual phase number (e.g., 2, 3, 4)
   - `[Phase-specific inputs]` → any inputs unique to this phase
   - `[Workstream A/B/C names]` → actual workstream names from execution plan
   - `[ADR numbers]` → specific ADRs this phase creates or references
   - `[branch-name]` → follow naming convention (feat/phaseX-topic or docs/phaseX-topic)
   - `[conventional-commit-message]` → e.g., "feat(guard): add PII regex rules"
3. **Add concrete sub-prompts** for each workstream task (A1, A2, B1, etc.)
4. **Specify acceptance criteria** that can be verified (file exists, test passes, output matches)
5. **Create companion files:**
   - `Phase-X-Checklist.md` (mirrors the checklist in state JSON)
   - `Phase-X-Execution-Plan.md` (detailed breakdown of tasks, risks, timeline)
   - `Phase-X-Assumptions-and-Open-Questions.md` (optional; helps with planning)
6. **Update references:**
   - Ensure master plan references this phase
   - Ensure prior phase summaries are listed in "Always read" section
   - Add new ADRs to the ADR list if this phase creates them

---

## Version History

- v1.0 (2025-11-03): Initial template based on Phase 0, 1, and 1.2 patterns
