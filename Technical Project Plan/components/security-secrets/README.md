# Security & Secrets

Overview: Vault + KMS (CE: Vault OSS, file KMS for dev). Manages signing keys, pseudonymization keys, provider API keys, and token lifecycles.

## KPIs
- Key rotation executed on schedule
- Zero secrets in git
- Access policy violations = 0

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - Secrets bootstrap: ../../docs/security/secrets-bootstrap.md\n- Keycloak dev guide: ../../docs/guides/keycloak-dev.md
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - Vault dev bootstrap script; Keycloak seeding script (idempotent)
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
