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
            f"{guard_url}/guard/status",
            params={"tenant_id": tenant_id},
            timeout=30,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Parse JSON response
        data = response.json()
        mode = data.get("mode", "unknown")
        categories = data.get("categories", [])
        health = data.get("health", "unknown")
        
        # Group categories by type
        regex_categories = [c for c in categories if c.get("type") == "regex"]
        ner_categories = [c for c in categories if c.get("type") == "ner"]
        
        # Format result
        result_lines = [
            "📊 **Privacy Guard Status**\n",
            f"**Health:** {health}",
            f"**Mode:** {mode}",
            f"**Tenant:** {tenant_id}",
            f"**Service URL:** {guard_url}\n",
            f"**Supported PII Categories:** ({len(categories)} total)\n",
        ]
        
        if regex_categories:
            result_lines.append("**Regex Patterns:**")
            for cat in regex_categories:
                result_lines.append(f"  - {cat.get('name', 'unknown').upper()}")
            result_lines.append("")
        
        if ner_categories:
            result_lines.append("**NER Categories:**")
            for cat in ner_categories:
                result_lines.append(f"  - {cat.get('name', 'unknown').upper()}")
        
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
