# ADR 0007: Agent Mesh MCP Extension

Status: Accepted (MVP)
Date: 2025-10-27

## Context
Cross-agent verbs are needed for orchestrated workflows. Goose v1.12 does not include cross-agent mesh semantics out-of-the-box.

## Decision
- Provide an MCP extension exposing: send_task, request_approval, notify, fetch_status.
- All calls go through the Controller API with Bearer JWT and policy evaluation; no peer-to-peer without controller.

## Consequences
- Clear, auditable contract for inter-agent communication; avoids P2P auth risks.

## Alternatives
- Use a message bus in MVP (NATS/Kafka) for mesh; higher ops overhead.
- Direct peer-to-peer with mTLS and per-agent ACLs; complex to manage in MVP.
