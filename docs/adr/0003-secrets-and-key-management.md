# ADR 0003: Secrets and Key Management (Vault + KMS)

- Status: Accepted (MVP)
- Date: 2025-10-26
- Authors: @owner
- Decision Drivers:
  - Business: Enterprise readiness with OSS-first defaults
  - Technical: Consistent secrets lifecycle; short-lived credentials
  - Compliance/Security: Central audit, rotation, and key custody
  - Cost/Latency: Prefer managed where available; CE remains self-hostable
- Assumptions
  - Multi-environment support (dev/stage/prod)

## Context
Agents/orchestrator need credentials for MCP tools, LLM providers, and deterministic masking keys. We want OSS-first and avoid vendor lock-in.

## Decision
- CE default: Vault OSS for secret brokering; Transit engine or cloud KMS for root keys; desktop uses OS keychain/sops for local secrets.
- SaaS default: Managed Vault (e.g., HCP Vault) + cloud KMS (AWS/GCP/Azure). The app talks only to Vault via a SecretsProvider interface; no direct KMS coupling in app code.

## MVP details
- Issue short-lived tokens scoped per role/tenant; least-privilege policies
- Store pseudonymization keys via Vault+KMS envelope; agents never persist raw keys
- Desktop: no Vault required; use OS keychain or age/sops for local-only installs

## Security & privacy impact
- Centralized audit for secret access; revocation and rotation supported

## Operational impact
- CE: Helm/chart or docker-compose for Vault; migration guides provided
- SaaS: managed Vault to minimize ops

## Consequences
- Benefits: Enterprise-accepted pattern; OSS-first; portable
- Risks/Trade-offs: Some ops complexity in CE; mitigated with tooling and guides

## Decision lifecycle
- Revisit after first pilot to tune TTLs and policies

## References
- ../../productdescription.md
- ../../requirements.md
- ../architecture/mvp.md
