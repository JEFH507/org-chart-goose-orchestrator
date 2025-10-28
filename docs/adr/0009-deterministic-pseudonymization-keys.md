# ADR 0009: Deterministic Pseudonymization Keys (Per-Tenant)

Status: Accepted (MVP)
Date: 2025-10-27

## Context
Consistent masking of sensitive tokens (PII/secrets) across a session/tenant enables coherent workflows. Keys must be per-tenant.

## Decision
- Use HMAC-SHA256-derived pseudonymization for stable mapping, per tenant. Store keys in Vault/KMS. Where format-preserving is needed (e.g., phone), use reversible FPE with strict authorization.
- Maintain ephemeral mapping state only as long as needed to re-identify on allowed endpoints.

## Consequences
- Consistent masking improves UX; key rotation and authorization need clear ops.

## Alternatives
- One-time masking with no re-identification; simpler but reduces utility.
