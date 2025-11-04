# Progress Log Instructions for Phase 2.5 & 3

**IMPORTANT:** Both Phase 2.5 and Phase 3 orchestration prompts include progress logging as part of post-execution deliverables.

---

## Progress Log Location

**Standard Location:** `docs/tests/`

**Naming Convention:**
- Phase 2.5: `docs/tests/phase2.5-progress.md`
- Phase 3: `docs/tests/phase3-progress.md`

---

## When to Create Progress Logs

### Phase 2.5
- Create `docs/tests/phase2.5-progress.md` during **Post-Execution** (after all workstreams complete)
- Include in final commit before merge to main

### Phase 3
- Create `docs/tests/phase3-progress.md` during **Post-Execution** (after all workstreams complete)  
- Include in final commit before merge to main

---

## Progress Log Template

Based on existing pattern (phase0-progress.md, phase1-progress.md, phase2-progress.md, phase2.2-progress.md):

```markdown
# Phase [X.X] — [Phase Title] — Progress Log

**Phase:** Phase [X.X] - [Title]  
**Status:** Complete  
**Created:** [Date]  
**Completed:** [Date]

---

## Overview

This log tracks progress for Phase [X.X], which [brief description].

**Scope:**
- [Workstream A description]
- [Workstream B description]
- [Workstream C description]

**Effort:** [Small/Medium/Large] ([X] days)

**Baseline (Previous Phase):**
- [Key metrics from previous phase]

**Targets (This Phase):**
- [Success criteria]

---

## Timeline

| Date | Milestone | Status | Notes |
|------|-----------|--------|-------|
| [Date] | Phase start | ✅ | [Notes] |
| [Date] | Workstream A complete | ✅ | [Notes] |
| [Date] | Workstream B complete | ✅ | [Notes] |
| [Date] | Workstream C complete | ✅ | [Notes] |
| [Date] | Phase complete | ✅ | [Notes] |

---

## Workstream Progress

### Workstream A: [Name]
- [X] Task 1 description
- [X] Task 2 description
- ...

**Status:** ✅ Complete  
**Duration:** [X hours/days]

### Workstream B: [Name]
- [X] Task 1 description
- [X] Task 2 description
- ...

**Status:** ✅ Complete  
**Duration:** [X hours/days]

---

## Deliverables

- ✅ [Deliverable 1]
- ✅ [Deliverable 2]
- ✅ [ADR-XXXX: Title]
- ✅ Updated VERSION_PINS.md
- ✅ Updated CHANGELOG.md

---

## Issues Encountered

### Issue 1: [Title]
- **Description:** [What happened]
- **Resolution:** [How it was resolved]
- **Impact:** [Minimal/Moderate/High]

---

## Metrics

### Performance
- [Metric 1]: [Value] (baseline: [baseline value])
- [Metric 2]: [Value] (baseline: [baseline value])

### Testing
- Unit tests: [X/Y] pass
- Integration tests: [X/Y] pass
- Smoke tests: [X/Y] pass

---

## Lessons Learned

1. **[Lesson 1]:** [Description]
2. **[Lesson 2]:** [Description]

---

## Next Phase

**Phase [X+1]:** [Title]  
**Prerequisites:** All Phase [X.X] deliverables complete  
**Estimated Start:** [Date]
```

---

## Inclusion in Orchestration Prompts

### Phase 2.5 Orchestration Prompt

Under **"Completion Checklist"** section:
```markdown
- [ ] Create Phase-2.5-Completion-Summary.md
- [ ] **Create docs/tests/phase2.5-progress.md** (progress log)
- [ ] Commit changes (conventional commit format)
```

Under **"Git Workflow > Commit Messages"**:
```bash
git add docs/tests/phase2.5-progress.md
git commit -m "docs(phase-2.5): add progress log

Tracks execution timeline, workstream completion, deliverables, and metrics.

Part of Phase 2.5 completion."
```

### Phase 3 Orchestration Prompt

Under **"Completion Checklist"** section:
```markdown
- [ ] Create Phase-3-Completion-Summary.md
- [ ] **Create docs/tests/phase3-progress.md** (progress log)
- [ ] Commit changes (conventional commit format)
```

Under **"Git Workflow > Commit Messages"**:
```bash
git add docs/tests/phase3-progress.md
git commit -m "docs(phase-3): add progress log

Tracks 9-day execution, workstream progress, ADR creation, and integration test results.

Part of Phase 3 completion."
```

---

## Verification

After phase completion:

```bash
# Check progress log exists
ls -la docs/tests/phase2.5-progress.md
ls -la docs/tests/phase3-progress.md

# Verify format
head -30 docs/tests/phase2.5-progress.md
head -30 docs/tests/phase3-progress.md
```

---

## Existing Examples

**Reference existing progress logs:**
- `docs/tests/phase0-progress.md` (6.4 KB)
- `docs/tests/phase1-progress.md` (6.4 KB)
- `docs/tests/phase2-progress.md` (40.4 KB)
- `docs/tests/phase2.2-progress.md` (52.7 KB)

These follow a consistent structure:
1. Header with phase info
2. Overview + scope
3. Timeline table
4. Workstream progress
5. Deliverables checklist
6. Issues encountered
7. Metrics
8. Lessons learned
9. Next phase info

---

**Prepared by:** Goose AI Agent  
**Date:** 2025-11-04  
**Status:** Instruction document for orchestration prompts  
**Action:** Progress logs will be created during execution (post-workstream completion)
