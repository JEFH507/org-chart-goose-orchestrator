# Identity/Auth Gateway

Overview: Front door for OIDC SSO and JWT issuance; bridges to goosed (X-Secret-Key) for compatibility. Enforces short token TTLs and role claims injection from IdP → profiles.

## KPIs
- Login success rate ≥ 99%
- Token TTL ≤ 30m, rotation within 5m grace
- Median login time ≤ 3s
- Authn errors < 0.5%
