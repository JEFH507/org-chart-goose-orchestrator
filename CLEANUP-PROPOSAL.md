# Repository Documentation Cleanup Proposal

**Date**: 2025-11-06  
**Status**: AWAITING APPROVAL  
**Impact**: Documentation only - NO CODE CHANGES

## Executive Summary

The repository has accumulated **361 documentation files** over time. This proposal reorganizes documentation to:
- **Improve discoverability** - Clear structure for active vs historical docs
- **Reduce confusion** - Eliminate duplicates and consolidate related content
- **Preserve history** - Archive (not delete) session artifacts
- **Zero code impact** - All changes are documentation-only

## Inventory Summary

| Category | Count | Status |
|----------|-------|--------|
| Root-level markdown | 10 | Review for duplicates |
| docs/ markdown | 15 | Keep active, archive summaries |
| Technical Project Plan | 194 | Keep structure, archive completed phases |
| tests/ markdown | 4 | Keep all |
| **Total** | **361** | Reorganize systematically |

## Code Safety Verification

✅ **No code references found** to documentation paths:
- Checked `src/` - Only reference is `/api-docs/` endpoint (API route, not file path)
- Checked `.goosehints` - No hardcoded doc paths
- DOCS_INDEX.md is a reference guide only (not loaded by code)

✅ **All integration tests verified working** before proposal:
- H4: 12/12 passing ✅
- H3 Finance: 8/8 passing ✅  
- H3 Legal: 10/10 passing ✅
- **Total: 30/30 tests passing**

## Proposed Actions

### Category 1: ROOT LEVEL - Consolidate & Archive

#### 🔄 CONSOLIDATE (Duplicates/Similar Purpose)

**1.1 Resume Prompts**
```
Current:
- RESUME_PROMPT.md (older)
- RESUME_PROMPT_FINAL.md (newer)

Action: KEEP RESUME_PROMPT_FINAL.md → Archive RESUME_PROMPT.md
Rationale: "FINAL" suffix indicates this is the current version
```

**1.2 H4 Completion Summaries**
```
Current:
- H4-COMPLETION-SUMMARY.md (root)
- docs/SESSION-SUMMARY-H4-COMPLETION.md (more detailed)

Action: KEEP docs/SESSION-SUMMARY-H4-COMPLETION.md → Archive H4-COMPLETION-SUMMARY.md
Rationale: Session summary in docs/ is more comprehensive and properly located
```

#### 📦 ARCHIVE (Historical/Superseded)

**1.3 Status Tracking**
```
File: WORKSTREAM-D-STATUS.md
Action: Archive to docs/archive/phase-artifacts/
Rationale: Superseded by Phase-5-Agent-State.json
Verification: Check if referenced in DOCS_INDEX.md
```

#### ✅ KEEP (Active)

```
- README.md (main project readme)
- CHANGELOG.md (release history)
- CONTRIBUTING.md (contribution guidelines)
- DOCS_INDEX.md (documentation index - VERIFY REFERENCES)
- VERSION_PINS.md (dependency versions)
- PROJECT_TODO.md (active task tracking)
- RESUME_PROMPT_FINAL.md (current session resume template)
- Cargo.toml, Cargo.lock, Makefile (.dockerignore, etc.) - Build files, not docs
```

---

### Category 2: docs/ - Archive Session Summaries

#### 📦 ARCHIVE (Session Artifacts - Historical Value)

**2.1 Session Summaries**
```
Move to: docs/archive/session-summaries/

Files:
- docs/SESSION-SUMMARY-H4-COMPLETION.md
- docs/H4-DEPLOYMENT-SUMMARY.md
- docs/analyst-profile-summary.md
- docs/analyst-profile-checklist.md
- docs/department-field-enhancement.md
- docs/PHASE-4-UPDATE-SUMMARY.md

Rationale: Historical artifacts from completed work
Value: Keep for reference, but not needed for daily operations
```

**2.2 Old Planning/Strategy Docs**
```
Move to: docs/archive/planning/

Files:
- docs/MASTER-PLAN-OLLAMA-NER-UPDATE.md (Phase 2.2 completed)
- docs/UPSTREAM-CONTRIBUTION-STRATEGY.md (may still be relevant - REVIEW)

Action: User review before archiving UPSTREAM-CONTRIBUTION-STRATEGY.md
```

#### ✅ KEEP (Active Documentation)

```
Active Build Guides:
- docs/BUILD_PROCESS.md ✅
- docs/BUILD_QUICK_START.md ✅
- docs/QUICK-START-TESTING.md ✅

Active Architecture:
- docs/HOW-IT-ALL-FITS-TOGETHER.md ✅
- docs/README.md ✅

Active References:
- docs/THIRD_PARTY.md ✅ (license info)
- docs/THOUGHTS.md ✅ (design notes)

All subdirectories:
- docs/adr/ ✅ (architectural decisions - NEVER archive)
- docs/api/ ✅ (API specs)
- docs/architecture/ ✅ (system design)
- docs/database/ ✅ (schema docs)
- docs/guides/ ✅ (user guides)
- docs/privacy/ ✅ (privacy design)
- docs/security/ ✅ (security design)
- docs/tests/ ✅ (progress logs - active reference)
- docs/vault/ ✅ (Vault integration)
```

---

### Category 3: Technical Project Plan/ - Archive Completed Phases

#### 📦 ARCHIVE (Completed Phase Artifacts)

**3.1 Already Archived**
```
Good examples of proper archiving:
- Technical Project Plan/PM Phases/Phase-2/archive/
- Technical Project Plan/PM Phases/Phase-2.2/archive/

These contain session summaries, interim findings, etc.
Keep this pattern!
```

**3.2 Candidates for Phase-Level Archive**

**Phase 0 (Complete)** - Archive session artifacts:
```
Move to: Technical Project Plan/PM Phases/Phase-0/archive/

Candidates:
- Phase-0-Reorg-Proposal.md (historical)
- Phase-0-Repo-Structure-Evaluation.md (historical)

Keep:
- Phase-0-Checklist.md (reference)
- Phase-0-Execution-Plan.md (reference)
- Phase-0-Summary.md (completion record)
- Phase-0-Agent-State.json (STATE FILE - ALWAYS KEEP)
```

**Phase 1 & 1.2 (Complete)** - Archive session artifacts:
```
Similar pattern - archive interim docs, keep:
- Checklist
- Execution Plan
- Completion Summary
- Agent State JSON
```

**Phase 2 & 2.2 (Complete)** - Already has archive/ subdirs ✅
```
Review archive/ contents:
- Ensure all session summaries moved
- Keep only essential docs at phase level
```

**Phase 3 (Complete)** - Archive session artifacts:
```
Move to: Technical Project Plan/PM Phases/Phase-3/archive/

Candidates:
- SESSION-1-SUMMARY.md
- SESSION-2-SUMMARY.md
- SESSION-HANDOFF-B8.md
- RESUME-B8-SHELL-SCRIPTS.md

Keep:
- Checklist, Execution Plan, Completion Summary
- Agent State JSON
- TESTING-STRATEGY.md (reference)
```

**Phase 4 (Complete)** - Archive session artifacts:
```
Move to: Technical Project Plan/PM Phases/Phase-4/archive/

Candidates:
- PHASE-4-RESUME-PROMPT.md (historical)

Keep:
- Checklist, Execution Plan, Completion Summary
- Agent State JSON
- README.md
```

**Phase 5 (ACTIVE)** - Keep all ✅
```
Current phase - do not archive yet
All files needed for active work
```

#### ✅ KEEP (Essential Structure)

```
For EACH completed phase, retain:
1. Phase-X-Agent-State.json (STATE - NEVER ARCHIVE)
2. Phase-X-Checklist.md (reference)
3. Phase-X-Execution-Plan.md (reference)
4. Phase-X-Completion-Summary.md (completion record)
5. Phase-X-Orchestration-Prompt.md (if exists - template)

Archive everything else to phase-level archive/ subdirectory
```

---

### Category 4: Tests - Keep All ✅

```
tests/accuracy/README.md ✅
tests/accuracy/TESTING-NOTES.md ✅
tests/fixtures/README.md ✅
tests/workstream-b/README.md ✅

All test documentation remains active
Performance test results in tests/perf/results/ - keep for benchmarking
```

---

## Proposed Directory Structure (After Cleanup)

```
/
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── DOCS_INDEX.md
├── VERSION_PINS.md
├── PROJECT_TODO.md
├── RESUME_PROMPT_FINAL.md
│
├── docs/
│   ├── archive/
│   │   ├── session-summaries/          [NEW]
│   │   │   ├── H4-DEPLOYMENT-SUMMARY.md
│   │   │   ├── SESSION-SUMMARY-H4-COMPLETION.md
│   │   │   ├── analyst-profile-summary.md
│   │   │   ├── analyst-profile-checklist.md
│   │   │   └── ...
│   │   ├── planning/                   [NEW]
│   │   │   ├── MASTER-PLAN-OLLAMA-NER-UPDATE.md
│   │   │   └── ...
│   │   └── obsolete/                   [NEW]
│   │       ├── H4-COMPLETION-SUMMARY.md (duplicate from root)
│   │       ├── RESUME_PROMPT.md (old version from root)
│   │       ├── WORKSTREAM-D-STATUS.md (superseded)
│   │       └── ...
│   │
│   ├── BUILD_PROCESS.md
│   ├── BUILD_QUICK_START.md
│   ├── QUICK-START-TESTING.md
│   ├── HOW-IT-ALL-FITS-TOGETHER.md
│   ├── adr/ (all ADRs - keep)
│   ├── api/ (keep)
│   ├── architecture/ (keep)
│   ├── guides/ (keep)
│   ├── tests/ (progress logs - keep)
│   └── ... (all other subdirs - keep)
│
├── Technical Project Plan/
│   ├── master-technical-project-plan.md
│   └── PM Phases/
│       ├── Phase-0/
│       │   ├── archive/                [NEW]
│       │   │   ├── Phase-0-Reorg-Proposal.md
│       │   │   └── Phase-0-Repo-Structure-Evaluation.md
│       │   ├── Phase-0-Agent-State.json
│       │   ├── Phase-0-Checklist.md
│       │   ├── Phase-0-Execution-Plan.md
│       │   └── Phase-0-Summary.md
│       │
│       ├── Phase-1/ (similar structure)
│       ├── Phase-1.2/ (similar structure)
│       ├── Phase-2/ (already has archive/)
│       ├── Phase-2.2/ (already has archive/)
│       ├── Phase-2.5/ (review)
│       │
│       ├── Phase-3/
│       │   ├── archive/                [NEW]
│       │   │   ├── SESSION-1-SUMMARY.md
│       │   │   ├── SESSION-2-SUMMARY.md
│       │   │   ├── SESSION-HANDOFF-B8.md
│       │   │   └── RESUME-B8-SHELL-SCRIPTS.md
│       │   ├── Phase-3-Agent-State.json
│       │   ├── Phase-3-Checklist.md
│       │   ├── Phase-3-Execution-Plan.md
│       │   ├── Phase-3-Completion-Summary.md
│       │   └── TESTING-STRATEGY.md
│       │
│       ├── Phase-4/
│       │   ├── archive/                [NEW]
│       │   │   └── PHASE-4-RESUME-PROMPT.md
│       │   ├── Phase-4-Agent-State.json
│       │   ├── Phase-4-Checklist.md
│       │   ├── Phase-4-Completion-Summary.md
│       │   └── README.md
│       │
│       └── Phase-5/ (ACTIVE - keep all)
│
└── tests/ (keep all)
```

---

## Execution Plan

### Phase 1: Preparation (Safe - Read-Only)

```bash
# 1. Create archive directories
mkdir -p docs/archive/{session-summaries,planning,obsolete}
mkdir -p "Technical Project Plan/PM Phases/Phase-0/archive"
mkdir -p "Technical Project Plan/PM Phases/Phase-1/archive"
mkdir -p "Technical Project Plan/PM Phases/Phase-1.2/archive"
mkdir -p "Technical Project Plan/PM Phases/Phase-3/archive"
mkdir -p "Technical Project Plan/PM Phases/Phase-4/archive"

# 2. Verify current tests pass (baseline)
./tests/integration/test_org_chart_jwt.sh
./tests/integration/test_finance_pii_jwt.sh
./tests/integration/test_legal_local_jwt.sh
# Expected: 30/30 passing
```

### Phase 2: Move Operations (Reversible)

**2.1 Root Level**
```bash
# Archive old resume prompt
git mv RESUME_PROMPT.md docs/archive/obsolete/

# Archive duplicate H4 summary
git mv H4-COMPLETION-SUMMARY.md docs/archive/obsolete/

# Archive old status file (AFTER user confirms not needed)
git mv WORKSTREAM-D-STATUS.md docs/archive/obsolete/
```

**2.2 docs/ Level**
```bash
# Move session summaries
git mv docs/SESSION-SUMMARY-H4-COMPLETION.md docs/archive/session-summaries/
git mv docs/H4-DEPLOYMENT-SUMMARY.md docs/archive/session-summaries/
git mv docs/analyst-profile-summary.md docs/archive/session-summaries/
git mv docs/analyst-profile-checklist.md docs/archive/session-summaries/
git mv docs/department-field-enhancement.md docs/archive/session-summaries/
git mv docs/PHASE-4-UPDATE-SUMMARY.md docs/archive/session-summaries/

# Move planning docs (AFTER user reviews UPSTREAM-CONTRIBUTION-STRATEGY.md)
git mv docs/MASTER-PLAN-OLLAMA-NER-UPDATE.md docs/archive/planning/
```

**2.3 Phase Artifacts**
```bash
# Phase 0
cd "Technical Project Plan/PM Phases/Phase-0"
git mv Phase-0-Reorg-Proposal.md archive/
git mv Phase-0-Repo-Structure-Evaluation.md archive/
cd -

# Phase 1 (list candidates first)
cd "Technical Project Plan/PM Phases/Phase-1"
# TBD based on file review

# Phase 3
cd "Technical Project Plan/PM Phases/Phase-3"
git mv SESSION-1-SUMMARY.md archive/
git mv SESSION-2-SUMMARY.md archive/
git mv SESSION-HANDOFF-B8.md archive/
git mv RESUME-B8-SHELL-SCRIPTS.md archive/
cd -

# Phase 4
cd "Technical Project Plan/PM Phases/Phase-4"
git mv PHASE-4-RESUME-PROMPT.md archive/
cd -
```

### Phase 3: Verification

```bash
# 1. Verify no broken references in DOCS_INDEX.md
rg "H4-COMPLETION-SUMMARY|RESUME_PROMPT\.md|WORKSTREAM-D-STATUS" DOCS_INDEX.md
# Should find nothing (or update references to archive/ paths)

# 2. Verify tests still pass
./tests/integration/test_org_chart_jwt.sh
./tests/integration/test_finance_pii_jwt.sh
./tests/integration/test_legal_local_jwt.sh
# Expected: 30/30 passing (no change)

# 3. Verify app builds
cd deploy/compose
docker compose -f ce.dev.yml build controller
# Should succeed

# 4. Check for any unexpected references
rg "RESUME_PROMPT\.md" --type md
rg "H4-COMPLETION-SUMMARY" --type md
```

### Phase 4: Update References

```bash
# Update DOCS_INDEX.md if needed
# Add archive/ section
# Update any broken links

# Update .goosehints if references moved docs
# (Current check shows no hardcoded doc paths)
```

### Phase 5: Commit & Monitor

```bash
# Create feature branch
git checkout -b chore/cleanup-documentation

# Review all changes
git status
git diff --staged

# Commit with conventional commit
git commit -m "chore: reorganize documentation into archive structure

- Archive completed session summaries to docs/archive/session-summaries/
- Archive obsolete resume prompts and status files to docs/archive/obsolete/
- Archive phase interim artifacts to phase-level archive/ subdirectories
- Preserve all state JSON, checklists, execution plans, completion summaries
- No code changes, documentation only
- All integration tests verified: 30/30 passing

Resolves documentation organization for easier navigation"

# Push and create PR
git push origin chore/cleanup-documentation
```

### Phase 6: Trial Period

```
Keep branch for 1 sprint (2 weeks)
If any issues found, easy to revert (git mv is tracked)
If no issues, merge to main
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Broken doc references | Low | Low | Pre-scan DOCS_INDEX.md, update references |
| Code references docs | Very Low | Medium | Already verified - no code refs found |
| User can't find historical docs | Low | Low | Archive structure clearly named |
| Accidental deletion | Very Low | High | Using `git mv` (tracked), not `rm` |
| Test failures | Very Low | High | Verify 30/30 passing before/after |

**Overall Risk**: ⬇️ **LOW** - Documentation-only changes with tracked git operations

---

## Questions for User Approval

### 1. WORKSTREAM-D-STATUS.md
**Question**: Is this file still actively used, or superseded by Phase-5-Agent-State.json?
- [ ] Archive it (it's obsolete)
- [ ] Keep it (still reference it)

### 2. UPSTREAM-CONTRIBUTION-STRATEGY.md
**Question**: Is this still relevant for future work, or was it specific to completed phases?
- [ ] Archive it (no longer relevant)
- [ ] Keep it in docs/ (still planning to contribute upstream)

### 3. Phase-1, Phase-1.2, Phase-2.5 Artifacts
**Question**: Should I do detailed review of these phase directories for archivable session artifacts?
- [ ] Yes, do full cleanup of all completed phases
- [ ] No, just do Phase-0, Phase-3, Phase-4 as proposed

### 4. Execution Preference
**Question**: How to proceed?
- [ ] Execute all changes at once (full cleanup)
- [ ] Execute in stages (root → docs → phases, with verification between)
- [ ] Show me the git commands first, I'll review before running

---

## Success Criteria

✅ **Documentation organized** - Clear active vs archived structure  
✅ **No code impact** - All tests passing (30/30)  
✅ **No broken references** - DOCS_INDEX.md updated  
✅ **Reversible** - All changes tracked in git  
✅ **Discoverable** - Archive structure clearly named  

---

## Next Steps

**USER ACTION REQUIRED**:
1. Review this proposal
2. Answer the 4 questions above
3. Approve execution plan
4. I'll execute with verification at each step
5. Monitor for 1 sprint, then merge if no issues

**Estimated Time**: 30 minutes (with verification between stages)

---

**Document Version**: 1.0  
**Last Updated**: 2025-11-06  
**Author**: goose (automated documentation cleanup proposal)
