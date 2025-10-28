# ADR 0004: Identity and Auth (OIDC SSO in MVP)

- Status: Accepted (MVP)
- Date: 2025-10-26
- Authors: @owner
- Decision Drivers:
  - Business: Enterprise posture from day one; OSS-friendly
  - Technical: Clear identity boundaries; minimal coupling
  - Compliance/Security: Standards-based; least privilege; auditability
  - Cost/Latency: Use standards and open-source defaults
- Assumptions
  - OIDC-capable IdP available (Keycloak for CE; Okta/Azure AD/Auth0 for SaaS)

## Context
We need authentication for users and secure service-to-service calls. Vendor lock-in must be avoided; CE must be fully self-hostable.

## Decision
- MVP uses OIDC SSO.
  - CE default: Keycloak (OIDC) with standard client config; desktop uses Device Code or System Browser + PKCE.
  - SaaS: Any OIDC IdP (Okta/Azure AD/Auth0). Controller exchanges for short-lived JWTs for internal service calls.

## Technical details
- Tokens: short-lived access tokens; refresh via IdP; internal JWTs minted by controller with scoped claims
- Desktop: tokens stored in OS keychain; no plain-text storage
- Claims mapping: adapter maps IdP-specific claims to roles/policies; config-driven

## Security & privacy impact
- Strong identity with minimal PII; auditable sign-in/out; rotate signing keys

## Operational impact
- CE: docker-compose Keycloak profile for quick start; docs for IdP config
- SaaS: straightforward IdP integration via OIDC adapter

## Consequences
- Benefits: Enterprise-grade from MVP; OSS-first; portable
- Risks/Trade-offs: Setup time (IdP config); mitigated by templates/examples

## Decision lifecycle
- Revisit after first enterprise pilot; add SCIM provisioning if needed

## References
- ../../productdescription.md
- ../../requirements.md
- ../architecture/mvp.md
