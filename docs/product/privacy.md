# Privacy by Design

Mask before cloud. Keep content local by default. Audit what matters, not what’s sensitive.

## Guard strategy
- Agent pre/post Privacy Guard is primary (local‑first; rules + small model via Ollama)
- Optional provider middleware wrapper as defense‑in‑depth (off by default in CE)
- Deterministic pseudonymization with per‑tenant keys; re‑identify only at authorized endpoints

## Data minimization
- Server stores metadata only (IDs, statuses, timestamps); content remains desktop‑local
- No PII in URLs; redacted structured logs; exportable audit (JSONL)

## Identity & secrets
- OIDC SSO (Keycloak for CE); controller mints short‑lived JWT for services
- Vault OSS + Transit (CE); desktop uses OS keychain/sops

---
Quick nav: [Docs Home](../README.md) • [Community Edition](./ce.md) • [Architecture One‑Pager](../architecture/one-pager.md)
