# Compliance Posture (MVP / Phase 0)

This project is not a certified SOC2/GDPR-compliant product; it is an MVP with a privacy-by-design posture. Key points:

- Privacy guard at source (ADR‑0002): Agent-side pre/post filter masks and pseudonymizes before any egress.
- Metadata‑only server (ADR‑0005/0012): The controller stores only minimal metadata (no raw prompts/responses).
- Identity & Auth (ADR‑0004/0006): OIDC SSO; JWT bridge to goosed in MVP. Short‑lived tokens.
- Auditability (ADR‑0008): Common audit schema and redaction summaries. ndjson export planned.
- Observability: OTLP‑ready traces/logs; sampling to reduce sensitive surface.

Disclaimer
- MVP only; not a substitute for formal compliance. Additional controls, policies, and audits are required for production compliance.
- Server logs and metrics must be configured to avoid PII content; redaction happens at source.

Roadmap
- Post‑MVP: stronger policy composition, dashboards, message bus adapter, and compliance packs.
