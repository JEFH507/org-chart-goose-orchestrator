# Repo Structure Evaluation Memo — Phase 0

## Current State Assessment (~/Gooseprojects/goose-org-twin)

- Strengths
  - Clear planning artifacts: master plan, ADRs 0001–0013, architecture docs.
  - Docs-centric layout with pointers to product, architecture, ADRs, and grants.
  - Scripts and tests scaffolding placeholders exist.
  - Goose v1.12 references included for alignment.
- Gaps for Phase 0
  - No standardized deploy directory for CE compose.
  - No API schemas or profile/audit skeletons yet.
  - No centralized conventions/CONTRIBUTING docs.
  - No services/ directory map for future components (controller, directory-policy, identity-gateway).

## Recommendation: Keep single repository (monorepo) for MVP

- Rationale
  - Simplicity and velocity: shared ADRs, docs, and CE deploy live together.
  - Tight coupling between controller, directory-policy, and gateway during MVP.
  - Aligns with ADR-0001 (HTTP-only) and ADR-0012 (metadata-only) without cross-repo coordination.
- Revisit split post-MVP
  - If/when services mature, split into separate repos (e.g., controller, directory-policy) with versioned APIs.
  - For now, subdirectories under services/ are sufficient.

## Proposed Target Repo Layout (directories only)

- services/
  - controller/
  - directory-policy/
  - identity-gateway/
- docs/
  - adr/
  - api/
    - controller/
    - schemas/
  - guides/
  - conventions/
  - tests/
- deploy/
  - compose/
    - ce.dev.yml
    - .env.ce
    - healthchecks/
- config/
  - profiles/
  - policy/
  - models/
- db/
  - migrations/
    - metadata-only/
- scripts/
  - dev/
- .github/
  - ISSUE_TEMPLATE/
  - PULL_REQUEST_TEMPLATE.md
- src/ (empty for now)

## Minimal Reorganization Plan (no code changes)

- Create new directories: services/, deploy/compose/, docs/guides/, docs/api/, docs/conventions/, config/profiles/, db/migrations/metadata-only/
- Move nothing destructively; add only. Keep “Technical Project Plan/” where it is for now.
- Add placeholders/files as defined in Phase 0 deliverables.

## Branch Strategy and Conventional Commit Patterns (Phase 0)

- Branches
  - main: protected; requires PR, CI lint pass
  - feature branches: feat/phase0-compose, chore/docs-dev-setup, feat/openapi-stub, chore/ci-templates
- Conventional commits
  - feat: scaffolding or new placeholders (no runtime code)
  - docs: documentation, guides, ADR references
  - chore: config, templates, version pins
  - ci: future workflow configs
  - build: dependency or compose changes (version pin updates)
  - refactor/test: likely not used in Phase 0
