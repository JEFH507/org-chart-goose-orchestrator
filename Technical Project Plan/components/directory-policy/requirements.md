# Requirements

## Functional
- GET /profiles/{role}; POST /evaluate; GET /directory/{id}
- Signature verification; versioning; profile diff.

## Non-functional
- Security: Signature (Ed25519), keys in Vault; WORM logs of profile changes.
- Performance: p95 decision ≤ 50ms.
