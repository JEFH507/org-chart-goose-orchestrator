"""get_privacy_status MCP Tool

Query Privacy Guard configuration and health.
"""

import os
import requests


async def get_privacy_status_handler() -> str:
    """
    Query Privacy Guard configuration and health status.
    
    Returns information about:
    - Current detection mode
    - Supported PII categories (regex + NER)
    - Service health status
    
    Environment Variables:
    - PRIVACY_GUARD_URL: Privacy Guard API base URL (default: http://localhost:8089)
    - TENANT_ID: Tenant identifier for multi-tenant isolation (default: test-tenant)
    
    Returns:
        str: Privacy Guard status report
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    try:
        # Make HTTP GET request to Privacy Guard API
        response = requests.get(
            f"{guard_url}/status",
            timeout=30,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Parse JSON response
        data = response.json()
        status = data.get("status", "unknown")
        mode = data.get("mode", "unknown")
        rule_count = data.get("rule_count", 0)
        config_loaded = data.get("config_loaded", False)
        model_enabled = data.get("model_enabled", False)
        model_name = data.get("model_name", "none")
        
        # Format result
        result_lines = [
            "📊 **Privacy Guard Status**\n",
            f"**Health:** {status}",
            f"**Mode:** {mode}",
            f"**Tenant:** {tenant_id}",
            f"**Service URL:** {guard_url}\n",
            "**Configuration:**",
            f"  - Config Loaded: {'✅ Yes' if config_loaded else '❌ No'}",
            f"  - Rule Count: {rule_count}",
            f"  - AI Model: {'✅ Enabled' if model_enabled else '❌ Disabled'}",
        ]
        
        if model_enabled:
            result_lines.append(f"  - Model Name: {model_name}")
        
        result_lines.append(
            "\n**Detection Capabilities:**\n"
            "  - Regex Rules: SSN, Email, Phone, Credit Card, etc.\n"
            "  - AI NER: Person names, Organizations, Locations, Dates"
        )
        
        return "\n".join(result_lines)
    
    except requests.exceptions.HTTPError as e:
        status_code = e.response.status_code if e.response else None
        error_detail = e.response.text if e.response else str(e)
        
        return (
            f"❌ HTTP {status_code} Error\n\n"
            f"Privacy Guard API rejected the request:\n"
            f"{error_detail}"
        )
    
    except requests.exceptions.ConnectionError as e:
        return (
            f"❌ Connection Error\n\n"
            f"Could not connect to Privacy Guard service:\n"
            f"{str(e)}\n\n"
            f"**Check:** Privacy Guard running at {guard_url}"
        )
    
    except Exception as e:
        return f"❌ Unexpected error: {type(e).__name__}\n\n{str(e)}"
