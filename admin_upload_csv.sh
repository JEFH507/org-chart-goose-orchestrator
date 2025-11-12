#!/bin/bash
# Admin CSV Upload Script with JWT Authentication

CSV_FILE="${1:-test_data/demo_org_chart.csv}"

echo "🔐 Getting JWT token from Keycloak..."
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8" \
  | jq -r '.access_token')

if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
    echo "❌ Failed to get JWT token"
    exit 1
fi

echo "✅ Token obtained (valid for 10 hours)"
echo ""

echo "📤 Uploading CSV: $CSV_FILE"
RESPONSE=$(curl -s -X POST http://localhost:8088/admin/org/import \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@$CSV_FILE")

echo "Response:"
echo "$RESPONSE" | jq '.'

echo ""
echo "💡 Your token (save this for reuse within 10 hours):"
echo "$TOKEN"
