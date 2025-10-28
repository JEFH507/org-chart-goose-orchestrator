# Plan

## Objectives
- Implement OIDC SSO, JWT mint/exchange, and goosed auth bridge.

## Scope
- MVP: OIDC Code Flow + PKCE, JWT (RS256) with tenant/role claims; gateway translates to goosed auth header.

## Responsibilities
- Token issuance/validation, claim normalization, session management, audit of auth events.

## WBS (Now/Next/Later; Effort)
- OIDC integration (Keycloak CE) [Now, M]
- JWT minting and verification libs [Now, S]
- Role claim mapping from IdP groups [Now, S]
- goosed bridge (X-Secret-Key or embedded middleware) [Now, M]
- Admin config (client IDs, redirect URIs) [Now, S]
- Audit hooks (login, token exchange) [Next, S]

## Timeline
- Weeks 1–2

## Milestones
- M1: Login + JWT + goosed bridge works locally; test flows documented.
