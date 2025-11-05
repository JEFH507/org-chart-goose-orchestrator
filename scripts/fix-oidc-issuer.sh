#!/bin/bash
# Fix OIDC Issuer URL mismatch between Keycloak and Controller
#
# Issue: Keycloak issues tokens with issuer=http://localhost:8080/realms/dev
#        Controller expects issuer=http://keycloak:8080/realms/dev
#
# Solution: Update OIDC_ISSUER_URL to use localhost (matching the token)
#          Keep OIDC_JWKS_URL using keycloak hostname (Controller can fetch from inside Docker)

echo "================================================"
echo "OIDC Issuer Fix - Updating .env.ce"
echo "================================================"
echo ""

ENV_FILE="deploy/compose/.env.ce"

if [ ! -f "$ENV_FILE" ]; then
  echo "❌ Error: $ENV_FILE not found"
  exit 1
fi

echo "Current OIDC configuration:"
grep "^OIDC_" "$ENV_FILE" || echo "(No OIDC variables found)"
echo ""

echo "The issue:"
echo "- Keycloak generates tokens with issuer: http://localhost:8080/realms/dev"
echo "- Controller currently expects: http://keycloak:8080/realms/dev"
echo "- Result: JWT validation fails with 'InvalidIssuer'"
echo ""

echo "The fix:"
echo "- Change OIDC_ISSUER_URL to: http://localhost:8080/realms/dev"
echo "- Keep OIDC_JWKS_URL as: http://keycloak:8080/realms/dev/protocol/openid-connect/certs"
echo "  (Controller can fetch JWKS from inside Docker network)"
echo ""

read -p "Apply fix? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Cancelled."
  exit 0
fi

# Update OIDC_ISSUER_URL
sed -i 's|^OIDC_ISSUER_URL=http://keycloak:8080/realms/dev|OIDC_ISSUER_URL=http://localhost:8080/realms/dev|' "$ENV_FILE"

echo ""
echo "✅ Updated $ENV_FILE"
echo ""
echo "New OIDC configuration:"
grep "^OIDC_" "$ENV_FILE"
echo ""
echo "================================================"
echo "Next steps:"
echo "================================================"
echo "1. Recreate the controller container:"
echo "   cd deploy/compose"
echo "   export \$(cat .env.ce | grep -v '^#' | xargs)"
echo "   docker compose -f ce.dev.yml up -d --force-recreate controller"
echo ""
echo "2. Verify JWT verification enabled:"
echo "   docker logs ce_controller 2>&1 | grep JWT"
echo "   # Should see: \"JWT verification enabled\""
echo ""
echo "3. Test authentication:"
echo "   ./scripts/test-jwt-auth.sh"
echo ""
echo "================================================"
