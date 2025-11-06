"""get_privacy_status MCP Tool

Query Privacy Guard configuration and status.
"""

import os

from mcp.types import Tool, TextContent
from pydantic import BaseModel
import requests


class GetPrivacyStatusParams(BaseModel):
    """Parameters for the get_privacy_status tool (no parameters needed)."""
    pass


async def get_privacy_status_handler(params: GetPrivacyStatusParams) -> list[TextContent]:
    """
    Get Privacy Guard configuration and status.
    
    Returns:
    - Current detection mode
    - Available PII categories
    - Service health status
    - Tenant configuration
    
    Environment Variables:
    - PRIVACY_GUARD_URL: Privacy Guard API base URL (default: http://localhost:8089)
    - TENANT_ID: Tenant identifier for multi-tenant isolation (default: test-tenant)
    
    Args:
        params: GetPrivacyStatusParams (no parameters)
        
    Returns:
        list[TextContent]: Privacy Guard status and configuration
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    try:
        # Make HTTP GET request to Privacy Guard API
        response = requests.get(
            f"{guard_url}/guard/status",
            headers={
                "Content-Type": "application/json",
            },
            params={
                "tenant_id": tenant_id,
            },
            timeout=10,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Parse JSON response
        data = response.json()
        
        # Extract status info
        mode = data.get("mode", "unknown")
        categories = data.get("categories", [])
        health = data.get("health", "unknown")
        
        # Format result
        result_lines = [
            "📊 **Privacy Guard Status**\n",
            f"**Health:** {health}",
            f"**Mode:** {mode}",
            f"**Tenant:** {tenant_id}",
            f"**Service URL:** {guard_url}\n",
            f"**Supported PII Categories:** ({len(categories)} total)",
        ]
        
        # Group categories by type
        regex_categories = [c for c in categories if c.get("type") == "regex"]
        ner_categories = [c for c in categories if c.get("type") == "ner"]
        
        if regex_categories:
            result_lines.append(f"\n**Regex Patterns:**")
            for cat in regex_categories:
                result_lines.append(f"  - {cat.get('name', 'unknown')}")
        
        if ner_categories:
            result_lines.append(f"\n**NER Categories:**")
            for cat in ner_categories:
                result_lines.append(f"  - {cat.get('name', 'unknown')}")
        
        return [TextContent(
            type="text",
            text="\n".join(result_lines)
        )]
    
    except requests.exceptions.HTTPError as e:
        status_code = e.response.status_code if e.response else None
        error_detail = e.response.text if e.response else str(e)
        
        return [TextContent(
            type="text",
            text=f"❌ HTTP {status_code} Error\n\n"
                 f"Could not retrieve privacy status:\n{error_detail}"
        )]
    
    except requests.exceptions.ConnectionError as e:
        return [TextContent(
            type="text",
            text=f"❌ Connection Error\n\n"
                 f"Could not connect to Privacy Guard service:\n{str(e)}\n\n"
                 f"**Check:**\n"
                 f"1. Service running: `curl {guard_url}/health`\n"
                 f"2. URL correct: {guard_url}"
        )]
    
    except Exception as e:
        return [TextContent(
            type="text",
            text=f"❌ Unexpected error: {type(e).__name__}\n\n{str(e)}"
        )]


# MCP Tool definition
get_privacy_status_tool = Tool(
    name="get_privacy_status",
    description=(
        "Get Privacy Guard configuration and status. "
        "Returns current mode, supported PII categories, health, and tenant info."
    ),
    inputSchema=GetPrivacyStatusParams.model_json_schema(),
)

# Attach handler to tool
get_privacy_status_tool.call = get_privacy_status_handler
