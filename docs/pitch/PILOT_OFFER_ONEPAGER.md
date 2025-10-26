# Org‑Aware Orchestrator Pilot — One‑Pager (Community Edition)

Audience: CIO/CTO, CISO, Department Leaders (Finance/Marketing), IT Ops
Duration: 4–6 weeks (bounded scope, privacy‑first)
License: Apache‑2.0 (core). Community Edition (CE) is fully self‑hostable.

## Value Proposition
Unlock safe, role‑aware AI workflows without vendor lock‑in. Deploy a local‑first, open‑source orchestrator that maps assistants to your org chart, enforces privacy guardrails before any cloud call, and provides approvals and audit you can trust.

## What You Get (Pilot Scope)
- A working CE deployment (docker‑compose) in your environment:
  - Keycloak (OIDC SSO), Vault OSS (+ Transit), Postgres (metadata), MinIO (optional), Ollama (local models)
- Minimal orchestrator (HTTP‑only) with:
  - Org Directory/Profiles & Policies, Task Router (rules), Session Broker (handoffs), Audit baseline
  - Agent pre/post Privacy Guard (local‑first; rules + small model). Optional provider wrapper for defense‑in‑depth
- 1–2 role‑specific workflows (choose Finance/Marketing) with in‑chat approvals
- Export/import tooling for portability (no lock‑in)
- Documentation and a recorded demo walkthrough

## Success Criteria (examples — tailored in kickoff)
- Safety & Privacy
  - Mask before cloud: zero raw PII leaks in tests; redaction present in logs; no PII in URLs
  - Key custody: Vault OSS in place; short‑lived tokens; no plaintext secrets at rest
- Reliability & Experience
  - Interactive p50 ≤ 5s, p95 ≤ 15s on reference flows
  - Approval flow completion ≥ 95%; zero critical errors in E2E demo
  - Stakeholder acceptance: ≥ 1 manager sponsor + ≥ 3 ICs rate ≥ 4/5 usefulness
- Governance
  - Approvals and audit events visible; exportable as JSONL; role policies enforced

## Privacy & Security Posture
- Data minimization: server stores metadata only (IDs/status/timestamps). Prompts, files, transcripts remain desktop‑local by default
- Identity: OIDC SSO (Keycloak or your IdP). Controller issues short‑lived JWT for service calls
- Secrets: Vault OSS + Transit; desktop uses OS keychain/sops. No secrets in logs
- Observability: structured JSON logs; OpenTelemetry optional

## Deliverables
- Running CE stack and orchestrator in your environment
- 1–2 org‑aware workflows (recipes + approvals) with Privacy Guard enabled
- Quickstart doc + ops notes; export/import examples; configuration backup
- Pilot report: results vs success criteria, cost/latency chart, risks & next steps

## Timeline (illustrative)
- Week 1: Kickoff; success criteria; identity/secrets bootstrap; CE stack up
- Week 2: Orchestrator HTTP API, profiles/policies, first workflow draft
- Week 3: Privacy Guard tuning; approvals + audit; E2E demo of workflow 1
- Week 4: Workflow 2 (optional); hardening; documentation; sign‑off
- Weeks 5–6 (extended option): Add “Whiteboard‑to‑Workflow” thin slice and expand user testing

## Pricing (pilot)
- Essentials (4 weeks, 1 workflow, CE stack): $15k–$20k fixed fee
- Enhanced (6 weeks, 2 workflows, whiteboard‑to‑workflow demo): $25k–$35k fixed fee
- Notes: Remote delivery; pricing excludes cloud costs; travel/T&M optional. Volume or follow‑on discounts available

## Next Steps
- 30‑min fit call → NDA (if required) → 60‑min technical kickoff
- We provide a short SOW for signature with milestones, success criteria, and pricing
