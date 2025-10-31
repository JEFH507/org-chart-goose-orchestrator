# Statement of Work (SOW) — Org‑Aware Orchestrator Pilot (Community Edition)

This Statement of Work ("SOW") is entered into between <Client> ("Client") and <Vendor> ("Vendor") for a bounded pilot deployment of the Org‑Aware Orchestrator (Community Edition) built on Goose.

## 1. Scope of Work
Vendor will deliver a pilot deployment of the Community Edition (CE) orchestrator in Client’s environment, including:
- CE docker‑compose stack: Keycloak (OIDC), Vault OSS (+ Transit), Postgres (metadata), optional S3-compatible object storage (SeaweedFS default; MinIO/Garage optional), Ollama (local models)
- Minimal orchestrator (HTTP‑only) with Directory/Profiles & Policies, Task Router, Session Broker, Audit baseline
- Agent pre/post Privacy Guard (local‑first). Optional provider wrapper for defense‑in‑depth (configurable)
- 1–2 role‑specific workflows (recipes + approvals) selected with Client (e.g., Finance, Marketing)
- Export/import tooling and documentation

## 2. Deliverables
- Running CE stack with configuration saved/exported for reproducibility
- 1–2 org‑aware workflows with approvals and audit events visible/exportable (JSONL)
- Quickstart runbook and admin notes; recorded demo
- Pilot report: outcomes vs success criteria, latency/cost metrics, risks & recommendations

## 3. Timeline and Milestones
- Duration: 4–6 weeks from kickoff
- Milestones:
  1) Kickoff and environment bootstrap (Week 1)
  2) First workflow E2E demo with Privacy Guard and approvals (Week 3)
  3) Final delivery and report (Week 4); optional Week 6 enhancements

## 4. Success Criteria
- Safety & Privacy: masking before cloud; no raw PII in logs or URLs; short‑lived tokens; secrets not stored in plaintext
- Reliability & UX: interactive p50 ≤ 5s, p95 ≤ 15s; approval flow completion ≥ 95%
- Governance: approvals and audit visible; role policies enforced; export/import works

## 5. Fees and Payment
- Essentials (4 weeks, 1 workflow): $15,000–$20,000 fixed fee
- Enhanced (6 weeks, 2 workflows + whiteboard‑to‑workflow thin slice): $25,000–$35,000 fixed fee
- Invoices: 50% at kickoff, 50% at final delivery; net 15 days
- Exclusions: cloud costs; travel; third‑party licenses; VAT as applicable

## 6. Assumptions and Client Responsibilities
- Client provides environment access and a test IdP (OIDC) or uses provided Keycloak
- Client nominates stakeholders (1 manager sponsor; ≥3 ICs) and sample, non‑sensitive test data
- Client acknowledges CE is Apache‑2.0 licensed; no vendor lock‑in; pilot is non‑production

## 7. Confidentiality and IP
- Mutual NDA (if required). Vendor retains IP for CE components and templates released under Apache‑2.0; Client owns their data/configuration

## 8. Acceptance
- Deliverables are accepted upon meeting success criteria or Client’s written acceptance at final demo

## 9. Termination
- Either party may terminate with 7 days’ notice; fees prorated for work performed

## 10. Contacts
- Vendor PM/Tech Lead: <name, email>
- Client Sponsor: <name, email>

---
Signatures:

Client: __________________________  Date: ____/____/____

Vendor: __________________________  Date: ____/____/____
