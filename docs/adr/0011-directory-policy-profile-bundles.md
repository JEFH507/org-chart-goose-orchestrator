# ADR 0011: Signed Profile Bundles and Policy Evaluate API

Status: Accepted (MVP)
Date: 2025-10-27

## Context
Agents must bootstrap from trusted role profiles with clear allowlists and prompts. Policy enforcement must be explainable and auditable.

## Decision
- Profiles packaged as YAML bundles signed with Ed25519. Agents verify signature at startup; reject unsigned/invalid bundles.
- Provide /profiles/{role} and /policy/evaluate APIs with ABAC-lite on claims and labels.

## Consequences
- Secure bootstrap; requires key custody and rotation ops.

## Alternatives
- Unsigned config, or encryption-only with SOPS; weaker trust guarantees.
