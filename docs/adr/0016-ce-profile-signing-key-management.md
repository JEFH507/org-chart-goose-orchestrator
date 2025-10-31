# ADR 0016: CE Profile Signing Key Management

- Status: Accepted (MVP)
- Date: 2025-10-31
- Authors: @owner

## Context
ADR-0011 requires signed profile bundles but does not prescribe how CE users manage signing keys safely.

## Decision
- CE uses developer-generated Ed25519 keypairs. Do not commit private keys to the repository or images.
- Trust roots are documented and distributed as public keys; agents verify signatures on startup and reject unsigned/invalid bundles.
- A throwaway demo keypair may appear only in documentation examples with explicit warnings; never used in automation.
- Plan for key rotation; record key IDs in audit events upon verification.

## Consequences
- Clear CE workflow for safe signing key management; reduces risk of private key leakage.

## Alignment
- ADR-0011: Signed bundles.
- ADR-0003: Secrets and key management.

## References
- Guides: docs/security/profile-bundle-signing.md
