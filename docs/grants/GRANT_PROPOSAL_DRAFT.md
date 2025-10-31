# Goose Grant Proposal — Org-Chart Orchestrated AI Framework (Community Edition + Pilot)

Author: @owner
License: Apache-2.0 (core)
Repository: https://github.com/... (to be provided)
Contact: Discord @... (to be provided)

## Executive Summary
Enterprises need safe, org-aware assistants that respect privacy and governance without vendor lock-in. We propose an open-source, org-chart–aware orchestrator for Goose that is local-first, agentic, and standards-driven. It orchestrates cross-agent workflows with approvals, enforces a local privacy guard, integrates OIDC SSO and Vault OSS + KMS, and remains portable via adapter interfaces. The Community Edition (CE) is fully self-hostable with docker-compose and prioritizes data minimization. We will validate product-market fit through paid pilots during the MVP cycle, demonstrating sustainable OSS.

## Problem
- Role-irrelevant copilots, weak governance, and vendor lock-in stall enterprise adoption.
- Privacy/compliance blockers prevent data sharing with cloud LLMs.
- Multi-agent workflows lack clear approvals, auditability, and org-aware routing.

## Solution
- Minimal orchestrator built on Goose with:
  - HTTP-only orchestration (MVP) with idempotent endpoints and audit.
  - Agent pre/post privacy guard (local-first; Ollama + rules). Optional provider middleware as defense-in-depth.
  - OIDC SSO (Keycloak for CE). Controller mints short-lived JWT for services.
  - Secrets: Vault OSS + Transit (CE) and cloud KMS optional; desktop uses OS keychain/sops.
-  - Storage: Postgres (metadata), optional S3-compatible object storage (SeaweedFS default; MinIO/Garage optional). Stateless orchestrator for content (desktop-local by default).
  - Provider interfaces (Auth/Secrets/Storage/Model/Bus) with conformance tests and open adapters.
- CE docker-compose for frictionless self-hosting; export/import tools for portability.

## What makes it novel (aligned with goose values)
- New interactions (pick 1 to emphasize — TBD):
  - Whiteboard-to-Workflow: generate runnable recipes from Mermaid mindmaps.
  - Voice/Meeting Approvals: in-chat, role-aware approvals captured during meetings.
- Self-flying (safe autonomy) — optional, lightweight baseline: background tasks with policy gates and escalation (HTTP-only).
- Self-improving (opt-in, privacy-preserving): propose recipe/profile improvements using audit metadata, with human approval.

## Milestones (12 months, quarterly check-ins)
### Q1 — CE Baseline, MVP Foundations, and First Paid Pilot
- Orchestrator HTTP API v1 (OpenAPI): tasks/route, sessions, approvals, status, profiles, audit.
- Agent Mesh MCP tools: send_task, request_approval, fetch_status, notify.
- Guard SDK: agent pre/post extension (local-first), provider wrapper optional (off by default in CE).
- Identity/Secrets: OIDC SSO (Keycloak), Vault OSS + Transit; desktop OS keychain/sops.
-  - CE docker-compose (Keycloak, Vault, Postgres, S3-compatible object storage — SeaweedFS default; MinIO/Garage optional, Ollama; Prometheus/Loki/Grafana optional).
- Docs: ADRs accepted; quickstart and demo; export/import CLI for policies/recipes/audit.
- Paid pilot(s) target by end of Q1 (bounded scope, privacy-first). Success: signed SOW or paid POC, N≥1.

### Q2 — Interaction (Whiteboard-to-Workflow) + Pilot Expansion
- Novel interaction v1 emphasis: Whiteboard-to-Workflow (Mermaid → runnable recipes) and chat approvals UX.
- Background mode (safe autonomy) with approval gates (optional, thin slice).
- Expand to N≥2 paid pilots with tight feedback loops.
- OTel-ready example (optional) and tuning playbooks.

### Q3 — Improvement Loop + Adapters
- Self-improving loop (opt-in): propose diffs to recipes/profiles; evaluation harness.
- - Provider interfaces finalized; conformance test suite; adapters (Keycloak, Vault, Postgres, S3-compatible object storage, Ollama) with badges.
- Additional reference profiles/recipes.

### Q4 — Scale Options + Packaging
- Optional BusProvider adapters (NATS/Redis Streams/Kafka), same APIs; default remains HTTP.
- Hardening: idempotency keys, retries/backoff; rate limits; open Helm chart optional.
- Case studies and community showcases; blog post.

## Deliverables per quarter
- Q1: CE compose, HTTP API v1 + OpenAPI, guard SDK, OIDC + Vault OSS wiring, demo + docs, export/import CLI.
- Q2: Interaction v1 + approvals UX, background mode, paid pilot(s) results, performance tuning notes.
- Q3: Improvement loop, conformance suite, adapters, more profiles/recipes.
- Q4: BusProvider adapters, hardening + packaging, case studies.

## Success Metrics
- Community: stars/forks, cookbook recipes, adapters passing conformance, Discord engagement.
- Adoption: CE downloads, time-to-first-demo, active pilots, pilot conversion to paid.
- Quality: latency p50/p95, approval completion rates, guard quality from synthetic tests.
- Portability: successful export/import runs and CE↔SaaS migration exercises.

## Openness & Modularity
- - Apache-2.0 for core. OSS-first defaults (Keycloak, Vault OSS, Postgres, S3-compatible object storage, Ollama).
- Adapter interfaces for Auth/Secrets/Storage/Model/Bus, config-not-code.
- Export/import tooling, file-based policies/recipes, documented schemas.

## Team & Budget
- Solo founder with option to hire OSS contributors and contractors (Panama-based to optimize cost) for critical path items (auth/guard/adapter work, docs, tests).
- Budget allocates to OSS engineering, documentation, conformance tests, community support, security review.

## Risks & Mitigation
- Guard accuracy → rules + small model; evaluation harness; manual overrides.
- Identity complexity → provide Keycloak profile + “known good” configs.
- Vendor drift → strict interfaces, conformance suite, S3/OpenAI/OIDC compatibility.
- PMF risk → paid pilot(s) in Q2; iterative focus on highest-value flows.

## Community Impact
- Reusable guard SDK and privacy patterns for Goose.
- CE compose stack lowers barrier to experimentation.
- Adapter ecosystem fosters diversity in infra choices.
