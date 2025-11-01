# Keycloak Dev Seeding Guide

This guide shows how to seed a dev realm, client, and roles in Keycloak using the idempotent script.

Prereqs:
- docker + compose
- ce.dev.yml keycloak service running (admin:admin)

Start services:
```bash
docker compose -f deploy/compose/ce.dev.yml up -d keycloak
```

Run seeding script:
```bash
scripts/dev/keycloak_seed.sh
```
Environment overrides (optional):
```bash
KEYCLOAK_CONTAINER=ce_keycloak KEYCLOAK_REALM=goose-dev KEYCLOAK_CLIENT_ID=goose-controller \
KEYCLOAK_ADMIN=admin KEYCLOAK_ADMIN_PASSWORD=admin scripts/dev/keycloak_seed.sh
```

What it creates:
- Realm: goose-dev
- Client: goose-controller (public)
- Roles: orchestrator, auditor

Idempotent: Re-running the script logs existing resources and makes no duplicate changes.
