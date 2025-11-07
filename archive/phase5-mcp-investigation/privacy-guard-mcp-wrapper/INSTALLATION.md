# Privacy Guard MCP Wrapper - Installation Guide

## Prerequisites

1. **Python 3.10+** with pip/venv installed
2. **Privacy Guard Service** running on port 8089
3. **Goose Desktop** installed (for testing)

## Installation

### 1. Install Python Dependencies

```bash
cd src/privacy-guard-mcp-wrapper

# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install package in editable mode
pip install -e .
```

### 2. Test the MCP Server

```bash
# Set environment variables
export PRIVACY_GUARD_URL="http://localhost:8089"
export TENANT_ID="test-tenant"

# Run the server (it will wait for stdio input)
python -m privacy_guard_mcp
```

Expected output:
```
Privacy Guard MCP Server starting...
Version: 0.1.0
Waiting for MCP client connection via stdio...
```

Press Ctrl+C to stop.

## Register with Goose Desktop

### 3. Add to MCP Configuration

Edit `~/.config/goose/mcp-servers.json`:

```json
{
  "privacy-guard": {
    "command": "python3",
    "args": ["-m", "privacy_guard_mcp"],
    "cwd": "/home/papadoc/Gooseprojects/goose-org-twin/src/privacy-guard-mcp-wrapper",
    "env": {
      "PRIVACY_GUARD_URL": "http://localhost:8089",
      "TENANT_ID": "test-tenant",
      "PYTHONPATH": "/home/papadoc/Gooseprojects/goose-org-twin/src/privacy-guard-mcp-wrapper"
    }
  }
}
```

**Note**: Adjust paths to match your installation.

### 4. Restart Goose Desktop

The Privacy Guard tools should now appear in Goose Desktop's tool list.

## Usage in Goose Desktop

### Example Conversations

**Detect PII:**
```
User: "Scan this text for PII: My SSN is 123-45-6789 and email is john@example.com"
Goose: [calls scan_pii tool]
Result: 
🔍 **PII Detection Results** (2 findings)
**Mode:** hybrid
**Tenant:** test-tenant

1. **SSN**
   - Text: `123-45-6789`
   - Position: 11-22
   - Confidence: 100.00%

2. **EMAIL**
   - Text: `john@example.com`
   - Position: 35-52
   - Confidence: 100.00%
```

**Mask PII:**
```
User: "Mask PII in this text: Contact John Smith at 555-1234"
Goose: [calls mask_pii tool]
Result:
✅ **PII Masking Complete**
**Method:** pseudonym
**Mode:** hybrid
**Replacements:** 2

**Masked Text:**
```
Contact Jane Doe at 555-9876
```

**Replacement Details:**
1. PERSON: `John Smith` → `Jane Doe`
2. PHONE: `555-1234` → `555-9876`
```

**Change Privacy Mode:**
```
User: "Set privacy detection to rules-only mode"
Goose: [calls set_privacy_mode tool]
Result:
✅ **Privacy mode updated**
**Mode:** rules_only
**Description:** Regex patterns only (fast)
**Tenant:** test-tenant
```

**Check Status:**
```
User: "What's the privacy guard status?"
Goose: [calls get_privacy_status tool]
Result:
📊 **Privacy Guard Status**
**Health:** healthy
**Mode:** hybrid
**Tenant:** test-tenant
**Service URL:** http://localhost:8089

**Supported PII Categories:** (10 total)

**Regex Patterns:**
  - SSN
  - EMAIL
  - PHONE
  - CREDIT_CARD

**NER Categories:**
  - PERSON
  - ORG
  - LOCATION
  - DATE
  - MONEY
  - TIME
```

## Troubleshooting

### MCP Server Not Appearing in Goose Desktop

1. Check logs: `~/.config/goose/logs/`
2. Verify JSON syntax in `mcp-servers.json`
3. Ensure Privacy Guard service is running:
   ```bash
   curl http://localhost:8089/health
   ```

### Python Import Errors

Ensure PYTHONPATH includes the wrapper directory:
```bash
export PYTHONPATH=/home/papadoc/Gooseprojects/goose-org-twin/src/privacy-guard-mcp-wrapper:$PYTHONPATH
```

### Connection Errors

1. Verify Privacy Guard is running: `docker ps | grep privacy-guard`
2. Check URL environment variable: `echo $PRIVACY_GUARD_URL`
3. Test manually: `curl -X POST http://localhost:8089/guard/scan -H "Content-Type: application/json" -d '{"text":"test","mode":"hybrid","tenant_id":"test-tenant"}'`

## Architecture

```
┌─────────────────────┐
│  Goose Desktop      │
│  (User Interface)   │
└──────────┬──────────┘
           │ stdio MCP
           │
┌──────────▼──────────────────┐
│  privacy-guard-mcp-wrapper  │
│  (Python MCP Server)        │
│                              │
│  Tools:                      │
│  - scan_pii                  │
│  - mask_pii                  │
│  - set_privacy_mode          │
│  - get_privacy_status        │
└──────────┬───────────────────┘
           │ HTTP
           │
┌──────────▼──────────┐
│  Privacy Guard      │
│  (Rust Service)     │
│  Port: 8089         │
└──────────┬──────────┘
           │ HTTP
           │
┌──────────▼──────────┐
│  Controller API     │
│  (Audit Log)        │
│  Port: 8088         │
└─────────────────────┘
```

## Next Steps

1. **Test with real data** in Goose Desktop
2. **Screenshot Quick Action Buttons** (when implemented in UI)
3. **Document user workflows** for common PII scenarios
4. **Performance tuning** (adjust timeouts, caching)
5. **Error handling improvements** based on production usage
