#!/bin/bash
# Get JWT Token for Admin Dashboard

echo "🔐 Generating JWT Token for Admin Dashboard..."
echo ""

TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8" \
  | jq -r '.access_token')

echo "✅ Token generated (valid for 10 hours)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 COPY THIS TOKEN:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "$TOKEN"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 How to use in Browser Console (Admin Dashboard):"
echo "1. Open http://localhost:8088/admin"
echo "2. Press F12 to open Developer Console"
echo "3. Paste this code:"
echo ""
echo "localStorage.setItem('admin_token', '$TOKEN');"
echo ""
echo "4. Refresh the page - uploads will now work!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
