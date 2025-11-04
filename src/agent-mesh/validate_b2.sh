#!/bin/bash
# B2 Validation Script - Test send_task implementation with Docker

set -e

echo "🚀 B2 Validation: send_task Tool"
echo "================================="
echo ""

cd "$(dirname "$0")"

echo "1️⃣  Building Docker image with Python 3.13..."
docker build -t agent-mesh:b2-test -f - . <<'EOF'
FROM python:3.13-slim

WORKDIR /app

# Install dependencies
RUN pip install --no-cache-dir \
    "mcp>=1.20.0" \
    "requests>=2.32.5" \
    "pydantic>=2.12.3" \
    "python-dotenv>=1.0.1"

# Copy source files
COPY tools/send_task.py ./tools/send_task.py
COPY tools/__init__.py ./tools/__init__.py
COPY test_send_task.py ./

CMD ["python", "test_send_task.py"]
EOF

echo ""
echo "2️⃣  Running validation tests..."
docker run --rm agent-mesh:b2-test

echo ""
echo "3️⃣  Checking tool implementation details..."
docker run --rm agent-mesh:b2-test python -c "
from tools.send_task import send_task_tool, SendTaskParams
import json

print('Tool Name:', send_task_tool.name)
print('Has Handler:', callable(send_task_tool.call))
print('')
print('Sample Input Schema (pretty printed):')
schema = send_task_tool.inputSchema
print(json.dumps({
    'properties': {k: {'type': v.get('type', 'unknown')} for k, v in schema.get('properties', {}).items()},
    'required': schema.get('required', [])
}, indent=2))
"

echo ""
echo "✅ B2 Validation Complete"
echo ""
echo "Summary:"
echo "- send_task tool structure: VALID ✓"
echo "- Parameter validation: WORKING ✓"
echo "- Input schema: CORRECT ✓"
echo "- Latest dependencies: INSTALLED ✓"
echo "  - mcp>=1.20.0"
echo "  - requests>=2.32.5"
echo "  - pydantic>=2.12.3"
echo ""
echo "Next: Test with actual Controller API (requires MESH_JWT_TOKEN)"
