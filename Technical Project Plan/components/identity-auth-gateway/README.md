# Identity/Auth Gateway

Overview: Front door for OIDC SSO and JWT issuance; bridges to goosed (X-Secret-Key) for compatibility. Enforces short token TTLs and role claims injection from IdP → profiles.

## KPIs
- Login success rate ≥ 99%
- Token TTL ≤ 30m, rotation within 5m grace
- Median login time ≤ 3s
- Authn errors < 0.5%

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - Keycloak dev notes: ../../docs/guides/keycloak-dev.md\n- Secrets bootstrap: ../../docs/security/secrets-bootstrap.md
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - Dev-only seeding docs/scripts; runtime gateway deferred
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
