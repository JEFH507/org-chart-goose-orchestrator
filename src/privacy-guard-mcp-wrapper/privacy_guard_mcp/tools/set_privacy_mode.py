"""set_privacy_mode MCP Tool

Configure Privacy Guard detection mode.
"""

import os

from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests


class SetPrivacyModeParams(BaseModel):
    """Parameters for the set_privacy_mode tool."""
    
    mode: str = Field(
        description="Detection mode to set: 'off' (disabled), 'rules_only' (regex), 'ner_only' (AI), or 'hybrid' (both)"
    )


async def set_privacy_mode_handler(params: SetPrivacyModeParams) -> list[TextContent]:
    """
    Set Privacy Guard detection mode.
    
    Modes:
    - off: Disable PII detection
    - rules_only: Use only regex patterns (fast, limited)
    - ner_only: Use only AI NER model (slower, comprehensive)
    - hybrid: Use both (recommended)
    
    Environment Variables:
    - PRIVACY_GUARD_URL: Privacy Guard API base URL (default: http://localhost:8089)
    - TENANT_ID: Tenant identifier for multi-tenant isolation (default: test-tenant)
    
    Args:
        params: SetPrivacyModeParams with mode
        
    Returns:
        list[TextContent]: Confirmation message
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    # Validate mode
    valid_modes = ["off", "rules_only", "ner_only", "hybrid"]
    if params.mode not in valid_modes:
        return [TextContent(
            type="text",
            text=f"❌ Invalid mode: {params.mode}\n\n"
                 f"**Valid modes:** {', '.join(valid_modes)}"
        )]
    
    try:
        # Make HTTP POST request to Privacy Guard API
        response = requests.post(
            f"{guard_url}/guard/config",
            headers={
                "Content-Type": "application/json",
            },
            json={
                "mode": params.mode,
                "tenant_id": tenant_id,
            },
            timeout=10,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Success
        mode_descriptions = {
            "off": "PII detection disabled",
            "rules_only": "Regex patterns only (fast)",
            "ner_only": "AI NER model only (accurate)",
            "hybrid": "Both regex + AI (recommended)",
        }
        
        return [TextContent(
            type="text",
            text=f"✅ **Privacy mode updated**\n\n"
                 f"**Mode:** {params.mode}\n"
                 f"**Description:** {mode_descriptions.get(params.mode, 'Unknown')}\n"
                 f"**Tenant:** {tenant_id}"
        )]
    
    except requests.exceptions.HTTPError as e:
        status_code = e.response.status_code if e.response else None
        error_detail = e.response.text if e.response else str(e)
        
        return [TextContent(
            type="text",
            text=f"❌ HTTP {status_code} Error\n\n"
                 f"Could not update privacy mode:\n{error_detail}"
        )]
    
    except Exception as e:
        return [TextContent(
            type="text",
            text=f"❌ Unexpected error: {type(e).__name__}\n\n{str(e)}"
        )]


# MCP Tool definition
set_privacy_mode_tool = Tool(
    name="set_privacy_mode",
    description=(
        "Set Privacy Guard detection mode: 'off', 'rules_only', 'ner_only', or 'hybrid'. "
        "Controls how PII is detected in subsequent scan/mask operations."
    ),
    inputSchema=SetPrivacyModeParams.model_json_schema(),
)

# Attach handler to tool
set_privacy_mode_tool.call = set_privacy_mode_handler
