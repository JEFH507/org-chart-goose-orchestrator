# ADR 0006: Identity/Auth Bridge (OIDC → JWT → goosed)

Status: Accepted (MVP)
Date: 2025-10-27

## Context
goose server defaults to X-Secret-Key authentication for its API. The project requires OIDC SSO and JWTs across services. We need a minimal bridge for MVP to avoid invasive changes to goosed while adopting OIDC/JWT.

## Decision
- Implement an identity gateway that performs OIDC (Code Flow + PKCE), mints RS256 JWTs with tenant/role claims, and forwards proxied requests to goosed with X-Secret-Key for MVP.
- Post-MVP, embed a JWT verifier in goosed to deprecate X-Secret-Key and accept Bearer JWTs directly.

## Consequences
- MVP integration is fast with minimal server changes; adds a sidecar/gateway.
- Later, security posture simplifies when JWT is verified directly in goosed.

## Alternatives
- Patch goosed immediately to accept JWT and OIDC; increases complexity now.
- Use mTLS for all intra-service calls in MVP; heavier ops overhead.
- Place an API gateway with OPA/OPA bundles; heavier configuration for MVP.
