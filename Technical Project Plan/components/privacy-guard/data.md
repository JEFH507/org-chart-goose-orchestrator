# Data

- Storage: Token maps ephemeral in-memory; optionally persisted encrypted locally if needed per session (TTL ≤ 24h).
- Retention: Redaction maps only as long as needed for re-identification in same session; purge afterwards.
