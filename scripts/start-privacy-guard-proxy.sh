#!/bin/bash
# Start Privacy Guard Proxy with auto-open browser

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_DIR="$PROJECT_ROOT/deploy/compose"

echo "🚀 Starting Privacy Guard Proxy..."

# Navigate to compose directory
cd "$COMPOSE_DIR"

# Start the service
docker compose --profile privacy-guard-proxy up -d

echo "⏳ Waiting for Privacy Guard Proxy to be healthy..."

# Wait for health check
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -sf http://localhost:8090/api/status > /dev/null 2>&1; then
        echo "✅ Privacy Guard Proxy is healthy!"
        break
    fi
    
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo "   Attempt $RETRY_COUNT/$MAX_RETRIES..."
    sleep 1
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "❌ Privacy Guard Proxy failed to start. Check logs with: docker compose logs privacy-guard-proxy"
    exit 1
fi

# Open browser
echo "🌐 Opening Control Panel in browser..."

if command -v xdg-open > /dev/null 2>&1; then
    # Linux
    xdg-open http://localhost:8090/ui
elif command -v open > /dev/null 2>&1; then
    # macOS
    open http://localhost:8090/ui
else
    echo "ℹ️  Could not auto-open browser. Please visit: http://localhost:8090/ui"
fi

echo ""
echo "✨ Privacy Guard Proxy is ready!"
echo "   Control Panel: http://localhost:8090/ui"
echo "   Proxy API: http://localhost:8090/v1/*"
echo ""
echo "   To view logs: docker compose logs -f privacy-guard-proxy"
echo "   To stop: docker compose --profile privacy-guard-proxy down"
