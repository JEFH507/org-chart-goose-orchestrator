# Phase 1 — Execution Plan (Detailed, non-prompt)

Status: Planned
Owner: Phase 1 Orchestrator

## Objectives
- Introduce minimal runtime for Controller consistent with OpenAPI stub
- Establish CI (linkcheck, spectral, compose health)
- Provide dev seeding scripts (Keycloak, Vault) and migration enhancements
- Document observability practices and Phase 1 smoke tests

## Scope (In)
- Minimal Controller runtime (status, audit ingest; other stubs)
- Compose integration (controller profile + healthcheck)
- CI pipeline and link/ref integrity
- Dev-only seeding scripts (idempotent)
- DB metadata indexes/FKs where safe
- Observability docs and log field alignment with ADR-0008

## Scope (Out)
- Full feature implementation of all endpoints
- Production hardening, HA, or k8s deployment
- Persisting transcripts/content beyond metadata

## Deliverables
- src/controller/* baseline (language TBD)
- .github/workflows/phase1-ci.yml
- scripts/dev/keycloak_seed.sh, scripts/dev/vault_dev_bootstrap.sh
- deploy/compose updates and controller healthcheck
- db migrations and runner docs
- docs/tests/smoke-phase1.md; CHANGELOG updated

## Dependencies
- Phase 0 artifacts (compose infra, schemas, OpenAPI stub)
- VERSION_PINS.md for images

## Workstreams and Tasks
- Optional: G — Repo-wide documentation/file audit & cleanup (approval-gated)
- A: Planning + CI foundation
  - A1: Author Phase-1 docs (Execution Plan, Checklist, Assumptions)
  - A2: CI skeleton (linkcheck, spectral, compose health)
- B: Controller baseline
  - B1: Implement minimal endpoints and logs
  - B2: Compose integration and healthcheck
- C: Seeding scripts
  - C1: Keycloak dev realm seeding
  - C2: Vault dev bootstrap
- D: DB Phase 1
  - D1: Metadata indexes/FKs; runner docs
- E: Observability
  - E1: Logging fields and OTLP placeholders
- F: Acceptance
  - F1: Smoke-phase1 doc; CHANGELOG update

## Acceptance Criteria
- CI passes all checks on PRs
- Compose with controller profile runs and healthchecks pass
- /status → 200; /audit/ingest → 202 and logs metadata-only fields
- Docs and migrations present and validated locally

## Risks / Mitigations
- Runtime language choice ambiguity → decide upfront (default rust)
- Link breaks after doc updates → CI linkcheck and local audit
- Compose flakiness → simple, reliable health scripts and retry guidance

## Timeline (suggested)
- Week 1: Workstreams A, B1
- Week 2: B2, C1, C2
- Week 3: D, E, F and PR merges

## Tracking and Logging
- Progress: docs/tests/phase1-progress.md (append-only)
- State: Technical Project Plan/PM Phases/Phase-1/Phase-1-Agent-State.json
- Branching: as per orchestrator prompt
