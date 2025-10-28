# Roadmap (High Level)

This roadmap expresses intent and sequencing. It will evolve as ADRs and technical requirements are accepted.

## v1.2 (MVP)
- Minimal Orchestrator: Directory & Policy, Router, Session Broker, Audit baseline
- Privacy Guard: agent pre/post (local-first with open models); optional provider middleware as defense-in-depth
- Identity: OIDC SSO at Orchestrator; controller mints short-lived JWTs for services
- Messaging: HTTP-only orchestration (no message bus in MVP)
- Storage: Minimal server-side metadata; desktop-local content by default; Postgres for metadata; object storage optional
- Observability: Structured JSON logs by default; OTel-ready optional
- Demo scenario, docs, and initial deployment guide
- Business validation: target first paid pilot by end of Q1; expand to 2+ in Q2

## v1.3 (Policy & Integrations)
- Policy composition/graph evaluation service
- Profile packs (department overlays), initial marketplace plumbing
- Optional message bus evaluation/adapter (NATS/Redis Streams) behind BusProvider interface
- Observability enhancements (dashboards via Prometheus/Loki/Grafana; OTel collector)

## v1.4 (Scale & Compliance)
- Multi-tenant hardening, quotas/limits, cost controls
- Advanced approval chains and exception handling
- Compliance packs (GDPR/CPRA templates, audit exports)
- Analytics and insights on agent/org productivity

## Always-on tracks
- Security & privacy hardening (red-team, threat modeling)
- CI/CD and test automation improvements
- Developer experience and templates (extensions, profiles)

## Dependencies & references
- ../adr/
- ../../productdescription.md
- ../../technical-requirements.md
- ./mvp.md
