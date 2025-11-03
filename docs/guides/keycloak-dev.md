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
KEYCLOAK_CONTAINER=ce_keycloak KEYCLOAK_REALM=dev KEYCLOAK_CLIENT_ID=goose-controller \
KEYCLOAK_ADMIN=admin KEYCLOAK_ADMIN_PASSWORD=admin scripts/dev/keycloak_seed.sh
```

Defaults created by the seed script (dev):
- Realm: dev
- Client: goose-controller (public)
- Roles (example): controller.ingest (optional), orchestrator, auditor

Get a dev token (password grant; dev-only):
```bash
curl -s -X POST \
  -d "grant_type=password" \
  -d "client_id=goose-controller" \
  -d "username=testuser" \
  -d "password=testpassword" \
  http://localhost:8080/realms/dev/protocol/openid-connect/token | jq -r .access_token
```

Notes
- The controller expects aud=goose-controller and iss=http://keycloak:8080/realms/dev in compose. Adjust for your host URLs if running outside compose.
- See also: docs/tests/smoke-phase1.2.md for end-to-end curl examples.

Idempotent: Re-running the script logs existing resources and makes no duplicate changes.
