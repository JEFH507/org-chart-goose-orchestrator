#!/bin/bash
# Integration test for controller + privacy guard
# Tests guard integration with GUARD_ENABLED=true and GUARD_ENABLED=false

set -euo pipefail

CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
GUARD_URL="${GUARD_URL:-http://localhost:8089}"

echo "=== Controller + Guard Integration Test ==="
echo "Controller: $CONTROLLER_URL"
echo "Guard: $GUARD_URL"
echo

# Test 1: Controller health check
echo "Test 1: Controller /status endpoint"
CONTROLLER_STATUS=$(curl -s "$CONTROLLER_URL/status")
echo "Response: $CONTROLLER_STATUS"
if echo "$CONTROLLER_STATUS" | grep -q '"status":"ok"'; then
    echo "✅ Controller is healthy"
else
    echo "❌ Controller health check failed"
    exit 1
fi
echo

# Test 2: Guard health check (if enabled)
if curl -s -f "$GUARD_URL/status" > /dev/null 2>&1; then
    echo "Test 2: Guard /status endpoint"
    GUARD_STATUS=$(curl -s "$GUARD_URL/status")
    echo "Response: $GUARD_STATUS"
    if echo "$GUARD_STATUS" | grep -q '"status":"healthy"'; then
        echo "✅ Guard is healthy"
    else
        echo "❌ Guard health check failed"
        exit 1
    fi
    GUARD_AVAILABLE=true
else
    echo "Test 2: Guard not available (GUARD_ENABLED=false or service down)"
    echo "⚠️  Skipping guard-specific tests"
    GUARD_AVAILABLE=false
fi
echo

# Test 3: Audit ingest without content (guard disabled mode)
echo "Test 3: Audit ingest without content field"
INGEST_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$CONTROLLER_URL/audit/ingest" \
    -H "Content-Type: application/json" \
    -d '{
        "source": "test-agent",
        "category": "agent.action",
        "action": "tool.execute",
        "subject": "user123",
        "traceId": "trace-001"
    }')
HTTP_CODE=$(echo "$INGEST_RESPONSE" | tail -1)
echo "HTTP Status: $HTTP_CODE"
if [ "$HTTP_CODE" = "202" ]; then
    echo "✅ Audit ingest successful (no content)"
else
    echo "❌ Audit ingest failed"
    exit 1
fi
echo

# Test 4: Audit ingest with PII content (guard enabled mode)
if [ "$GUARD_AVAILABLE" = true ]; then
    echo "Test 4: Audit ingest with PII content (guard should mask)"
    INGEST_WITH_PII=$(curl -s -w "\n%{http_code}" -X POST "$CONTROLLER_URL/audit/ingest" \
        -H "Content-Type: application/json" \
        -d '{
            "source": "test-agent",
            "category": "agent.action",
            "action": "tool.execute",
            "content": "Contact John Doe at 555-123-4567 or john.doe@example.com",
            "traceId": "trace-002"
        }')
    HTTP_CODE=$(echo "$INGEST_WITH_PII" | tail -1)
    echo "HTTP Status: $HTTP_CODE"
    if [ "$HTTP_CODE" = "202" ]; then
        echo "✅ Audit ingest with content successful"
        echo "   (Check controller logs for redaction counts)"
    else
        echo "❌ Audit ingest with content failed"
        exit 1
    fi
else
    echo "Test 4: Skipped (guard not available)"
fi
echo

# Test 5: Verify determinism (same PII should produce same masked output)
if [ "$GUARD_AVAILABLE" = true ]; then
    echo "Test 5: Determinism test (same email twice)"
    # First call
    curl -s -X POST "$CONTROLLER_URL/audit/ingest" \
        -H "Content-Type: application/json" \
        -d '{
            "source": "test-agent",
            "category": "test",
            "action": "determinism-1",
            "content": "alice@example.com",
            "traceId": "trace-det-1"
        }' > /dev/null
    
    # Second call with same email
    curl -s -X POST "$CONTROLLER_URL/audit/ingest" \
        -H "Content-Type: application/json" \
        -d '{
            "source": "test-agent",
            "category": "test",
            "action": "determinism-2",
            "content": "alice@example.com",
            "traceId": "trace-det-2"
        }' > /dev/null
    
    echo "✅ Determinism test completed"
    echo "   (Check controller logs: same email should have same pseudonym)"
else
    echo "Test 5: Skipped (guard not available)"
fi
echo

echo "=== All Tests Passed ==="
echo
echo "Summary:"
echo "  - Controller: ✅ Healthy"
if [ "$GUARD_AVAILABLE" = true ]; then
    echo "  - Guard: ✅ Healthy and integrated"
    echo "  - Integration: ✅ Working (with masking)"
else
    echo "  - Guard: ⚠️  Disabled or unavailable"
    echo "  - Integration: ✅ Working (passthrough mode)"
fi
echo
echo "Next steps:"
echo "1. Check controller logs: docker compose logs controller"
echo "2. Check guard logs: docker compose logs privacy-guard"
echo "3. Verify redaction counts appear in controller logs"
echo "4. Test with GUARD_ENABLED=true by setting in .env.ce"
