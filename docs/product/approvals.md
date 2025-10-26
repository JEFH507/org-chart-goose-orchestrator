# Org‑Aware Approvals

Role‑based approvals with audit and scoped context — simple HTTP, clear contracts, human‑in‑the‑loop.

## What it is
- In‑chat, role‑aware approvals for common flows (e.g., Finance approvals, Marketing reviews)
- HTTP‑only MVP with idempotent endpoints and short‑lived JWT auth
- Audit events for each approval with exportable JSONL

## How it works
- Task routed to approver via Directory/Policy and Router (skills/permissions)
- Approver responds in chat; orchestrator records decision and updates session state
- No content custody: only metadata on the server; content remains local

---
Quick nav: [Docs Home](../README.md) • [Flow: Approval](../architecture/seq_approval.md) • [Privacy by Design](./privacy.md)
