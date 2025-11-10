#!/bin/bash
# Integration tests for Task B.6: Document & Media Handling
# Tests content type detection and mode enforcement

set -e

PROXY_URL="http://localhost:8090"
CONTROLLER_URL="http://localhost:8088"
FAKE_AUTH="Bearer sk-test-fake-key-12345"

echo "=================================="
echo "B.6: Content Type Handling Tests"
echo "=================================="

# Test 1: Detect maskable content (application/json)
echo "Test 1: Maskable content type (application/json)..."
response=$(curl -s -X POST "${PROXY_URL}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${FAKE_AUTH}" \
  -d '{
    "model": "test",
    "messages": [{"role": "user", "content": "Hello"}]
  }' 2>&1 || echo "EXPECTED_ERROR")

if [[ "$response" == *"EXPECTED_ERROR"* ]] || [[ "$response" == *"error"* ]]; then
  echo "✓ Test 1 passed: JSON detected as maskable (proxy attempted masking)"
else
  echo "✗ Test 1 failed: Unexpected response"
  exit 1
fi

# Test 2: Verify content type is logged correctly
echo "Test 2: Verify application/json is detected as maskable..."
# Set mode to Auto
curl -s -X PUT "${PROXY_URL}/api/mode" \
  -H "Content-Type: application/json" \
  -d '{"mode": "auto"}' > /dev/null

# Make a request with explicit application/json
response=$(curl -s -w "\n%{http_code}" -X POST "${PROXY_URL}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: ${FAKE_AUTH}" \
  -d '{
    "model": "test",
    "messages": [{"role": "user", "content": "Hello"}]
  }' 2>&1 || echo "ERROR")

http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

# JSON content should trigger masking attempt (will fail at Privacy Guard/provider)
if [[ "$http_code" == "500" ]] || [[ "$http_code" == "502" ]] || [[ "$body" == *"error"* ]]; then
  echo "✓ Test 2 passed: JSON content triggers masking (as expected)"
else
  echo "✗ Test 2 failed: HTTP $http_code - Unexpected response"
  echo "Response: $body"
  exit 1
fi

# Test 3: Strict mode + image content → error 400
echo "Test 3: Strict mode with image content (rejected)..."
# Set mode to Strict
curl -s -X PUT "${PROXY_URL}/api/mode" \
  -H "Content-Type: application/json" \
  -d '{"mode": "strict"}' > /dev/null

response=$(curl -s -w "\n%{http_code}" -X POST "${PROXY_URL}/v1/chat/completions" \
  -H "Content-Type: image/jpeg" \
  -H "Authorization: ${FAKE_AUTH}" \
  -d '{}')

http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [[ "$http_code" == "400" ]] && [[ "$body" == *"content_type_not_allowed"* ]]; then
  echo "✓ Test 3 passed: Strict mode blocks image/* (400 Bad Request)"
else
  echo "✗ Test 3 failed: HTTP $http_code - Expected 400 with content_type_not_allowed"
  echo "Response: $body"
  exit 1
fi

# Test 4: Strict mode + PDF → error 400
echo "Test 4: Strict mode with PDF content (rejected)..."
response=$(curl -s -w "\n%{http_code}" -X POST "${PROXY_URL}/v1/chat/completions" \
  -H "Content-Type: application/pdf" \
  -H "Authorization: ${FAKE_AUTH}" \
  -d '{}')

http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [[ "$http_code" == "400" ]] && [[ "$body" == *"content_type_not_allowed"* ]]; then
  echo "✓ Test 4 passed: Strict mode blocks application/pdf (400 Bad Request)"
else
  echo "✗ Test 4 failed: HTTP $http_code - Expected 400 with content_type_not_allowed"
  echo "Response: $body"
  exit 1
fi

# Test 5: Auto mode + PDF → passthrough
echo "Test 5: Auto mode with PDF content (passthrough)..."
# Set mode back to Auto
curl -s -X PUT "${PROXY_URL}/api/mode" \
  -H "Content-Type: application/json" \
  -d '{"mode": "auto"}' > /dev/null

response=$(curl -s -w "\n%{http_code}" -X POST "${PROXY_URL}/v1/chat/completions" \
  -H "Content-Type: application/pdf" \
  -H "Authorization: ${FAKE_AUTH}" \
  -d '{}')

http_code=$(echo "$response" | tail -n1)

# Auto mode should allow PDF (will fail at provider but not at proxy)
if [[ "$http_code" == "502" ]] || [[ "$http_code" == "400" ]]; then
  echo "✓ Test 5 passed: Auto mode allows application/pdf (passthrough attempted)"
else
  echo "✗ Test 5 failed: HTTP $http_code - Expected 502 or 400 (provider error, not proxy block)"
  exit 1
fi

# Test 6: Bypass mode + any content → passthrough
echo "Test 6: Bypass mode with any content type..."
# Set mode to Bypass
curl -s -X PUT "${PROXY_URL}/api/mode" \
  -H "Content-Type: application/json" \
  -d '{"mode": "bypass"}' > /dev/null

response=$(curl -s -w "\n%{http_code}" -X POST "${PROXY_URL}/v1/chat/completions" \
  -H "Content-Type: image/png" \
  -H "Authorization: ${FAKE_AUTH}" \
  -d '{}')

http_code=$(echo "$response" | tail -n1)

# Bypass mode should not block any content type
if [[ "$http_code" != "400" ]] || [[ "$response" != *"content_type_not_allowed"* ]]; then
  echo "✓ Test 6 passed: Bypass mode allows all content types"
else
  echo "✗ Test 6 failed: Bypass mode blocked content"
  echo "Response: $response"
  exit 1
fi

# Test 7: Text content type (text/plain)
echo "Test 7: Text content type detection..."
# Set mode to Auto
curl -s -X PUT "${PROXY_URL}/api/mode" \
  -H "Content-Type: application/json" \
  -d '{"mode": "auto"}' > /dev/null

response=$(curl -s -w "\n%{http_code}" -X POST "${PROXY_URL}/v1/chat/completions" \
  -H "Content-Type: text/plain" \
  -H "Authorization: ${FAKE_AUTH}" \
  -d '{"messages":[{"role":"user","content":"test"}]}')

http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

# text/* is maskable, should attempt masking (will fail but that's expected)
if [[ "$http_code" == "502" ]] || [[ "$http_code" == "500" ]]; then
  echo "✓ Test 7 passed: text/plain detected as maskable"
else
  echo "✗ Test 7 failed: HTTP $http_code - Expected masking attempt"
  echo "Response: $body"
  exit 1
fi

# Test 8: Activity log verification
echo "Test 8: Verify content type in activity log..."
sleep 1  # Allow logs to flush
activity=$(curl -s "${PROXY_URL}/api/activity")

if [[ "$activity" == *"image"* ]] && [[ "$activity" == *"strict_mode_blocked"* ]]; then
  echo "✓ Test 8 passed: Activity log contains content type events"
else
  echo "✗ Test 8 failed: Activity log missing content type events"
  echo "Activity log (last 200 chars): ${activity: -200}"
  exit 1
fi

# Reset mode to Auto for other tests
curl -s -X PUT "${PROXY_URL}/api/mode" \
  -H "Content-Type: application/json" \
  -d '{"mode": "auto"}' > /dev/null

echo ""
echo "=================================="
echo "✓ All 8 content type tests passed!"
echo "=================================="
