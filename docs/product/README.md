# Product One‑Pager

Org‑aware, local‑first orchestration that respects privacy and governance — without vendor lock‑in.

## Problem
- Role‑irrelevant copilots and tool sprawl stall adoption
- Privacy/compliance blockers: data can’t leave the boundary
- Multi‑agent flows lack org context, approvals, and audit

## Solution
- Org‑aware orchestration with role profiles and approvals
- Local‑first Privacy Guard (mask before cloud) and minimal custody
- HTTP‑only MVP; OIDC SSO; Vault OSS + Transit; export/import tooling

## Value
- Deploy anywhere (desktop/CE/SaaS) with the same open interfaces
- Governed autonomy: approvals, audit, and policy‑driven routing
- Lower risk: content stays local; server stores metadata only

## How we solve it
- Agent pre/post Privacy Guard; optional provider middleware
- Directory/Policy, Task Router, Session Broker, Audit baseline
- Open adapters: OIDC, Vault/KMS, Postgres+S3, OpenAI‑compatible, MCP

## Examples
- Finance: IC submits report → Manager → Dept Head (approvals)
- Marketing: cross‑dept review with scoped context and audit
- Planning: mindmap to recipes (whiteboard‑to‑workflow)

## What’s next
- First paid pilot (4–6 weeks): 1–2 workflows, approvals, audit
- Optional: Whiteboard‑to‑Workflow thin slice

---

Quick nav: [Docs Home](../README.md) • [Architecture One‑Pager](../architecture/one-pager.md) • [Pilot Offer](PILOT_OFFER_ONEPAGER.md)
