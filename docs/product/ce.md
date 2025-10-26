# Community Edition (CE)

Self‑hostable, OSS‑first deployment you can run in minutes — no vendor lock‑in.

## What you get
- docker‑compose stack: Keycloak (OIDC), Vault OSS (+ Transit), Postgres (metadata), MinIO (optional artifacts), Ollama (local models)
- HTTP‑only orchestrator MVP with role profiles, routing, approvals, and audit metadata
- Agent pre/post Privacy Guard (local‑first); optional provider wrapper as a safety belt
- Export/import tools for policies, recipes, and audit (JSON/JSONL/TAR)

## Why CE (benefits)
- Local‑first privacy: desktop holds content; server stores metadata only
- Standards: OIDC, S3‑compatible, OpenAI‑compatible, MCP; adapters for portability
- Open governance: file‑based policies/recipes; Git‑friendly

## CE vs SaaS mapping
- Identity: CE = Keycloak; SaaS = any OIDC (Okta/Azure/Auth0)
- Secrets: CE = Vault OSS + Transit; SaaS = managed Vault + cloud KMS
- Storage: CE = Postgres + MinIO; SaaS = managed SQL + S3/GCS
- Guard: CE = agent pre/post primary; SaaS = provider wrapper ON by default

---
Quick nav: [Docs Home](../README.md) • [Architecture One‑Pager](../architecture/one-pager.md) • [Privacy by Design](./privacy.md)
