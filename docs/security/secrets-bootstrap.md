# Dev Secrets Bootstrap (Phase 1.2)

This document describes how to bootstrap Vault in dev mode and manage the pseudonymization salt per ADR-0020.

## Prerequisites
- docker + compose
- ce.dev.yml vault service running (dev mode, root token)

## Bootstrap Steps

### 1. Start Vault
```bash
docker compose -f deploy/compose/ce.dev.yml up -d vault
```

### 2. Run Bootstrap Script
```bash
VAULT_ADDR=http://localhost:8200 VAULT_TOKEN=root scripts/dev/vault_dev_bootstrap.sh
```

What it ensures:
- KV v2 secrets engine at `secret/`
- Policy `goose-dev-read` with read/list permissions for `secret/data/*` and `secret/metadata/*`

Idempotent: Re-running the script updates/ensures mounts and policy without introducing secrets.

## Pseudonymization Salt Management

Per ADR-0020, the pseudonymization salt is stored at `secret/pseudonymization` with key `pseudo_salt`.

### Write a Development Salt
```bash
# Set environment for Vault CLI
export VAULT_ADDR=http://localhost:8200
export VAULT_TOKEN=root

# Write a sample salt (dev-only; generate a secure random value for production)
vault kv put secret/pseudonymization pseudo_salt="dev-sample-salt-$(date +%s)"
```

### Read the Salt
```bash
# Read the entire secret
vault kv get secret/pseudonymization

# Read just the pseudo_salt value
vault kv get -field=pseudo_salt secret/pseudonymization
```

### Export to Environment for Controller
The controller receives `PSEUDO_SALT` via environment variable (not by calling Vault directly in Phase 1.2):

```bash
# Export the salt to environment
export PSEUDO_SALT=$(vault kv get -field=pseudo_salt secret/pseudonymization)

# Verify
echo "PSEUDO_SALT=${PSEUDO_SALT}"

# Add to .env.ce for compose (DO NOT COMMIT)
echo "PSEUDO_SALT=${PSEUDO_SALT}" >> deploy/compose/.env.ce
```

### Using with Controller
The controller will hash/pseudonymize subject metadata using `PSEUDO_SALT` before logging if the environment variable is present.

Example compose usage:
```yaml
services:
  controller:
    environment:
      - PSEUDO_SALT=${PSEUDO_SALT}
```

## Security Notes
- **Dev mode only**: The root token and exposed port are NOT production-safe
- **DO NOT commit** actual salt values to git; use .env.ce (git-ignored)
- **Production**: Use proper Vault authentication, seal/unseal, and TLS
- **Rotation**: Changing the salt will change all pseudonymized identifiers
