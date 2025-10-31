# Profile Bundle Signing (CE)
> See also: ADR 0016 (docs/adr/0016-ce-profile-signing-key-management.md)


Decision summary
- Developers generate their own Ed25519 keypairs. Do not commit private keys.
- The repo may include a throwaway demo keypair only in documentation examples and never used in automation. Mark with loud warnings.
- Agents verify signatures on profile bundles at startup; unsigned/invalid bundles are rejected.

How to generate keys (example)
- Using age/ssh-keygen (example only) or your preferred tool. Store private keys securely (OS keychain, Vault dev).
- Document the public keys in a trust roots file (e.g., `config/policy/trust-roots.json`).

Operational notes
- Keep a rotation plan. Record key IDs in audit events when bundles are verified.
- Never embed private keys in Docker images or CI logs.

Alignment
- ADR‑0011 (signed bundles) and ADR‑0003 (secrets & key management).
