# Privacy Guard MCP Wrapper

Python MCP server that exposes Privacy Guard functionality to Goose Desktop and other MCP clients.

## Architecture

```
Goose Desktop <--(stdio MCP)--> Python MCP Server <--(HTTP)--> Privacy Guard Service (port 8089)
```

## Tools

1. **scan_pii**: Detect PII in text
2. **mask_pii**: Mask PII with tokens (FPE or pseudonyms)
3. **set_privacy_mode**: Change detection mode (off/rules_only/ner_only/hybrid)
4. **get_privacy_status**: Query current privacy settings

## Installation

```bash
cd src/privacy-guard-mcp-wrapper
python3 -m venv .venv
source .venv/bin/activate
pip install -e .
```

## Usage

Register in Goose Desktop (~/.config/goose/mcp-servers.json):

```json
{
  "privacy-guard": {
    "command": "python",
    "args": ["-m", "privacy_guard_mcp"],
    "env": {
      "PRIVACY_GUARD_URL": "http://localhost:8089",
      "TENANT_ID": "test-tenant"
    }
  }
}
```

## Development

Based on Phase 3 Agent Mesh MCP pattern.
