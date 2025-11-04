# Next Session Resume Prompt

**Date:** 2025-11-04  
**Phase Completed:** Phase 2.2 (Privacy Guard Model Enhancement)  
**Status:** ✅ MERGED to main (PR #31)

---

## Quick Start

```
You are resuming work on the goose-org-twin project after completing Phase 2.2.

CONTEXT:
- Phase 2.2 COMPLETE: Privacy Guard with Ollama model enhancement (qwen3:0.6b)
- PR #31 merged to main (19 commits squashed)
- All branches cleaned up (feat/phase2.2-ollama-detection deleted)
- Completion summary: Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md

USER'S STRATEGIC DECISIONS:
- UI timing: Build UI **AFTER** Controller API + Agent Mesh + Directory/Policies + Profile
- Reason: Want comprehensive demo of full stack capabilities
- This means: UI comes after Phase 3-5 (Controller, Directory, Audit per master plan)
- Model default: GUARD_MODEL_ENABLED=true (core product feature, not optional)
- Performance: 22.8s P50 accepted for improved PII detection (accuracy > speed for MVP)
- Phase 2.3/2.4: DEFER to post-MVP (not on critical path to demo)

OPTIONAL NEXT PHASES (User to decide):
1. Phase 2.3: Performance Optimization (~1-2 days) - OPTIONAL SUB-PHASE
   - Smart model triggering (80-90% fast path → P50 ~100ms from 22.8s)
   - Model warm-up on startup (eliminate cold start)
   - Improved merge strategy (model-only → MEDIUM confidence)
   - Expected result: 240x performance improvement for typical workloads
   
2. Phase 2.4: Model Fine-Tuning (Post-MVP, ~2-3 days) - OPTIONAL SUB-PHASE (DEFERRED)
   - Fine-tune qwen3:0.6b on corporate PII data
   - Use Phase 2 fixtures (150+ PII samples) as training data
   - LoRA (Low-Rank Adaptation) for efficient training
   - Expected: +10-20% accuracy improvement

3. Minimal Privacy Guard UI (~2-3 days) - OPTIONAL ADDITION (DEFERRED)
   - Configuration panel (model toggle, modes, entity types)
   - Live PII tester (text input, detect/mask, highlighted results)
   - Status dashboard (health, model status, stats)
   - **USER PREFERENCE:** Build this AFTER Phase 3-5 (Controller + Directory + Profile)

4. Phase 3: Controller API + Agent Mesh (L - per master plan) - RECOMMENDED NEXT
   - OpenAPI v1 published (tasks, approvals, sessions, profiles proxy, audit ingest)
   - Controller routes with JWT auth middleware (from Phase 1.2)
   - Agent Mesh MCP tools (send_task, request_approval, notify, fetch_status)
   - Idempotency + retry w/ jitter + request size limits
   - Integration test: cross-agent approval demo
   - NOTE: Add quarterly dependency version audit task (Rust, Ollama, Docker images)

5. Phase 4: Directory/Policy + Profiles (M - per master plan)
   - Profile bundle schema (YAML) + signature (Ed25519)
   - GET /profiles/{role} and POST /policy/evaluate
   - Enforce extension allowlists per role
   - Policy default-deny with explainable deny reasons

6. Phase 5: Audit & Observability (S - per master plan)
   - OTLP export config; audit event schema; ndjson export

7. Phase 6: Model Orchestration (M - per master plan)
   - Lead/worker selection, cost-aware downshift; policy constraints

8. Phase 7: Storage/Metadata (S - per master plan)
   - Postgres schema for sessions/tasks/approvals/audit index; retention baseline

9. Phase 8: Packaging/Deployment + Docs (M - per master plan)
   - Desktop packaging guidance, docker compose for services, runbooks, demo script

YOUR TASK:
1. ASK USER: "Which phase would you like to proceed with?"
   - Options: Phase 2.3 (perf optimization), Phase 3 (Controller API), or other?
   - Remind: UI deferred until after Phase 3-5 per user preference

2. IF Phase 2.3 (Performance Optimization):
   - Create Phase-2.3-Execution-Plan.md (workstreams A/B/C)
   - Create Phase-2.3-Checklist.md (tasks with time estimates)
   - Create Phase-2.3-Agent-State.json (tracking)
   - Start with workstream A (Smart Triggering)

3. IF Phase 3+ (Controller API or later phases):
   - Follow master technical project plan structure
   - Reference: Technical Project Plan/master-technical-project-plan.md
   - Create Phase-X folders with standard artifacts

4. MAINTAIN pause/resume protocol:
   - Update state JSON after each task
   - Append progress log entries
   - Commit frequently with conventional commits
   - Update checklist completion percentages

TRACKING FILES PATTERN:
- State: Technical Project Plan/PM Phases/Phase-X/Phase-X-Agent-State.json
- Progress: docs/tests/phaseX-progress.md
- Checklist: Technical Project Plan/PM Phases/Phase-X/Phase-X-Checklist.md
- Completion: Technical Project Plan/PM Phases/Phase-X/Phase-X-Completion-Summary.md

IMPORTANT FILES TO READ:
- Master plan: Technical Project Plan/master-technical-project-plan.md
- Phase 2.2 summary: Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md
- PROJECT_TODO.md: Future work documented (lines 134-276)
- CHANGELOG.md: Recent changes (Phase 2.2 entry added)

GIT CONFIGURATION:
- GitHub user: JEFH507 (Javier)
- Git identity: Javier / 132608441+JEFH507@users.noreply.github.com
- Remote (SSH): git@github.com:JEFH507/org-chart-goose-orchestrator.git
- Default branch: main
- Branch naming: feature/*, fix/*, chore/*, docs/*, build/*
- Merge policy: Conventional commits, squash merges for PRs

SSH KEY HANDLING (for Git push):
Always use GUI askpass instead of waiting for input on terminal:
```bash
export DISPLAY=${DISPLAY:-:0}
export SSH_ASKPASS_REQUIRE=force
SSH_ASKPASS="$(command -v ssh-askpass-gnome || command -v ssh-askpass || true)"
if [ -n "$SSH_ASKPASS" ]; then setsid -w ssh-add ~/.ssh/id_ed25519 < /dev/null; fi
```

CURRENT REPOSITORY STATE:
- Branch: main (all Phase 2.2 changes merged)
- Last commit: 6530c4b (Phase 2.2 squashed merge)
- Clean working directory: Yes
- All feature branches deleted: Yes
- Ready for new work: Yes

Phase 2.2 PERFORMANCE BASELINE:
- Regex-only (model disabled): P50=16ms, P95=22ms, P99=23ms
- Model-enhanced (CPU-only): P50=22.8s, P95=47s, P99=47s
- Performance acceptable for compliance/audit use cases
- Smart triggering can reduce P50 to ~100ms (Phase 2.3)

RECOMMENDED NEXT STEP: Phase 3 (Controller API + Agent Mesh)

REASONING:
1. Critical path to demo (user wants to demo after Phase 3-5)
2. Foundation for Phases 4-5 (Directory/Policy and Audit depend on Controller API)
3. Highest value (enables multi-agent orchestration - core product vision)
4. Phase 2.3 optimization NOT critical (22.8s acceptable, can defer to post-demo)
5. Model already enabled by default (GUARD_MODEL_ENABLED=true - core feature)

Phase 2.3/2.4 can be optimized post-demo based on real user feedback.
```

---

## Phase 2.3 Details (If Selected)

### Goal
Reduce P50 latency from 22.8s to ~100ms for CPU-only inference through intelligent optimization.

### Workstreams

**Workstream A: Smart Model Triggering (~3 hours)**
- Task A1: Add `ConfidenceEvaluator` trait
- Task A2: Implement fast-path logic (skip model if regex HIGH confidence)
- Task A3: Update hybrid detector to use smart triggering
- Task A4: Add configuration flag `GUARD_SMART_TRIGGER_ENABLED`
- Expected: 80-90% requests use fast path → P50 ~100ms

**Workstream B: Model Warm-Up & Merge Strategy (~3 hours)**
- Task B1: Add model warm-up on service startup
- Task B2: Implement `MergeStrategy` enum (HighPrecision, HighRecall, Balanced)
- Task B3: Change model-only confidence to MEDIUM (reduce false positives)
- Task B4: Add configuration `GUARD_MERGE_STRATEGY`
- Expected: Eliminate cold start, better accuracy/performance trade-off

**Workstream C: Testing & Validation (~2 hours)**
- Task C1: Performance benchmark (validate P50 ~100ms target)
- Task C2: Accuracy validation (ensure no regression)
- Task C3: Smoke tests (5 tests like Phase 2.2)
- Task C4: Update documentation

**Total Effort:** ~8 hours (1 day)

---

## Phase 3 Details (If Selected)

**See:** Technical Project Plan/master-technical-project-plan.md (Phase 3: Controller API + Agent Mesh)

**Key Components:**
- OpenAPI v1 specification
- Controller HTTP routes (tasks, approvals, sessions, profiles proxy)
- Agent Mesh MCP tools
- JWT auth middleware (already implemented in Phase 1.2)
- Integration tests (cross-agent approval demo)

**Prerequisites:**
- Phase 1.2 complete ✅ (JWT verification)
- Phase 2.2 complete ✅ (Privacy Guard with model)
- Keycloak operational ✅
- Vault operational ✅

---

## References

- **Master Plan:** `Technical Project Plan/master-technical-project-plan.md`
- **Phase 2.2 Summary:** `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md`
- **PROJECT_TODO:** Future work (lines 134-276)
- **CHANGELOG:** Recent changes
- **GitHub PR:** https://github.com/JEFH507/org-chart-goose-orchestrator/pull/31

---

**Prepared:** 2025-11-04  
**Next Session:** Ask user for phase selection, then proceed with execution
