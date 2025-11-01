# Dev Secrets Bootstrap

This document describes how to bootstrap Vault in dev mode for Phase 1. No secrets are written.

Prereqs:
- docker + compose
- ce.dev.yml vault service running (dev mode, root token)

Start Vault:
```bash
docker compose -f deploy/compose/ce.dev.yml up -d vault
```

Run bootstrap script:
```bash
VAULT_ADDR=http://localhost:8200 VAULT_TOKEN=root scripts/dev/vault_dev_bootstrap.sh
```

What it ensures:
- KV v2 secrets engine at `secret/`
- Policy `goose-dev-read` with read/list permissions for `secret/data/*` and `secret/metadata/*`

Idempotent: Re-running the script updates/ensures mounts and policy without introducing secrets.
