# Data

## Postgres tables
- directory_nodes(id, parent_id, labels jsonb)
- profiles(id, role, version, bundle yaml, signature, created_at)
- policies(id, expr, effect, version)

## Retention
- Keep history 1 year; soft delete.
