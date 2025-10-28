# Requirements

## Functional
- detect(text) → entities; process(text, policy) → masked + mapRef
- reidentify(masked, mapRef) for authorized endpoints only.

## Non-functional
- Security: Keys from Vault/KMS; no key material in logs.
- Performance: P50 ≤ 500ms.
