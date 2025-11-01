#!/usr/bin/env bash
set -euo pipefail

# Idempotent Keycloak dev seeding: realm, client, roles
KC_CONTAINER="${KEYCLOAK_CONTAINER:-ce_keycloak}"
KC_ADMIN_USER="${KEYCLOAK_ADMIN:-admin}"
KC_ADMIN_PASS="${KEYCLOAK_ADMIN_PASSWORD:-admin}"
REALM="${KEYCLOAK_REALM:-goose-dev}"
CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
ROLES=(orchestrator auditor)

if ! docker ps --format '{{.Names}}' | grep -qx "$KC_CONTAINER"; then
  echo "[keycloak_seed] ERROR: Container $KC_CONTAINER not running. Start compose keycloak first." >&2
  exit 1
fi

KCADM="/opt/keycloak/bin/kcadm.sh"

exec_kc() {
  docker exec -e KEYCLOAK_ADMIN="$KC_ADMIN_USER" -e KEYCLOAK_ADMIN_PASSWORD="$KC_ADMIN_PASS" "$KC_CONTAINER" bash -lc "$@"
}

# Login to master realm
exec_kc "$KCADM config credentials --server http://localhost:8080 --realm master --user $KC_ADMIN_USER --password $KC_ADMIN_PASS"

# Realm
if exec_kc "$KCADM get realms/$REALM >/dev/null 2>&1"; then
  echo "[keycloak_seed] Realm '$REALM' exists."
else
  echo "[keycloak_seed] Creating realm '$REALM'..."
  exec_kc "$KCADM create realms -s realm=$REALM -s enabled=true"
fi

# Client
if exec_kc "$KCADM get clients -r $REALM -q clientId=$CLIENT_ID | grep -q '"clientId"'"; then
  echo "[keycloak_seed] Client '$CLIENT_ID' exists."
else
  echo "[keycloak_seed] Creating client '$CLIENT_ID'..."
  exec_kc "$KCADM create clients -r $REALM -s clientId=$CLIENT_ID -s protocol=openid-connect -s publicClient=true -s enabled=true -s 'redirectUris=[\"http://localhost/*\"]'"
fi

# Roles
for role in "${ROLES[@]}"; do
  if exec_kc "$KCADM get roles -r $REALM | grep -q '"name":\s*\"$role\"'"; then
    echo "[keycloak_seed] Role '$role' exists."
  else
    echo "[keycloak_seed] Creating role '$role'..."
    exec_kc "$KCADM create roles -r $REALM -s name=$role -s 'description=$role role for dev'"
  fi
done

echo "[keycloak_seed] Done."
