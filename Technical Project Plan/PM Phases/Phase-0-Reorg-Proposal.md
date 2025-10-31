# Phase 0 — Repository Reorganization Proposal (non-destructive, approval-gated)

Status: Draft (awaiting approval)
Author: Phase 0 Orchestrator
Date: 2025-10-31

Objective
- Reduce root clutter and align the repository with the target layout proposed in Phase-0-Repo-Structure-Evaluation.md, without changing any implementation scope for Phase 0.
- Keep moves minimal, non-destructive, and limited to documentation and planning artifacts.
- Update internal links immediately. Leave backups/ intact.

Guardrails
- Only move low-risk documentation/planning files.
- Use git mv for all changes.
- Update references in README, docs/ADR, and docs/architecture as needed.
- No code or config behavior changes; no deletions.

References
- Technical Project Plan/PM Phases/Phase-0-Repo-Structure-Evaluation.md
- README.md

Current root-level files identified
- CHANGELOG.md (stay at root)
- ENTERPRISE_SSO_AND_LOGS_REVIEW.md (doc)
- Makefile (stay at root)
- "org chart goose project analysis" (doc; space in name)
- plan.mmd (diagram/plan)
- productdescription.md (product doc)
- PROJECT_TODO.md (stay at root)
- README.md (stay at root)
- requirements.md (requirements doc)
- RUNNING_RECIPES.md (how-to doc)
- technical-requirements.md (technical requirements)
- THOUGHTS.md (notes)
- VERSION_PINS.md (stay at root)

Proposed moves (Current → Target)
- productdescription.md → docs/product/productdescription.md
  - Rationale: Belongs with product docs alongside docs/product/README.md
- requirements.md → docs/product/requirements.md
  - Rationale: Product/PO output; referenced by architecture and ADRs
- technical-requirements.md → docs/architecture/technical-requirements.md
  - Rationale: Technical spec; referenced by architecture/mvp.md
- plan.mmd → docs/architecture/plan.mmd
  - Rationale: Planning diagram; referenced by roadmap/mvp
- THOUGHTS.md → docs/THOUGHTS.md
  - Rationale: Notes; centralize under docs/
- RUNNING_RECIPES.md → docs/guides/RUNNING_RECIPES.md
  - Rationale: How-to; fits guides
- ENTERPRISE_SSO_AND_LOGS_REVIEW.md → docs/compliance/ENTERPRISE_SSO_AND_LOGS_REVIEW.md
  - Rationale: Compliance/enterprise note
- "org chart goose project analysis" → docs/product/org-chart-project-analysis.md
  - Rationale: Product analysis; normalize filename to kebab-case

Files to keep at root (no change)
- README.md, CHANGELOG.md, PROJECT_TODO.md, Makefile, VERSION_PINS.md
- Technical Project Plan/ (directory remains in place)

Link update plan
- Update internal references using ripgrep and targeted replacements:
  - ../../productdescription.md → ../../product/productdescription.md (in ADRs)
  - ../../requirements.md → ../../product/requirements.md (in ADRs)
  - ../../technical-requirements.md → ../../architecture/technical-requirements.md (in ADRs)
  - ./productdescription.md → ./docs/product/productdescription.md (in README/backups)
  - ./THOUGHTS.md → ./docs/THOUGHTS.md (in README/backups)
  - plan.mmd → docs/architecture/plan.mmd (in docs/README.md)
  - RUNNING_RECIPES.md → docs/guides/RUNNING_RECIPES.md (where referenced)
  - ENTERPRISE_SSO_AND_LOGS_REVIEW.md → docs/compliance/ENTERPRISE_SSO_AND_LOGS_REVIEW.md
- Perform a final scan for the old paths after changes and patch remaining links.

Risk assessment
- Low: All are documentation moves. Some ADRs reference relative paths; these will be updated.
- Backups directory contains historical copies; we will not alter it in this pass to preserve history.

Execution steps (upon approval)
1) Create branch chore/phase0-reorg.
2) Apply git mv for each mapping above.
3) Update references in:
   - README.md
   - docs/ADR files that reference root docs (keeping backups/ unchanged)
   - docs/architecture/* and docs/README.md
4) Quick link checks with ripgrep for old paths; fix any found.
5) Commit with message: chore(docs): non-destructive repo organization (approved)
6) Push and open PR.

Approval checkbox (fill by approver)
- [ ] Approved to proceed with proposed moves
- [ ] Changes limited to documentation and planning artifacts
- [ ] Backups directory untouched

Notes/Requests from approver
- ...
