"""set_privacy_mode MCP Tool

Configure Privacy Guard detection mode.
"""

import os
import requests


async def set_privacy_mode_handler(mode: str) -> str:
    """
    Configure Privacy Guard detection mode.
    
    Modes:
    - off: PII detection disabled
    - rules_only: Regex patterns only (fast, lower accuracy)
    - ner_only: AI NER model only (slower, higher accuracy)
    - hybrid: Both regex + AI (recommended, best accuracy)
    
    Environment Variables:
    - PRIVACY_GUARD_URL: Privacy Guard API base URL (default: http://localhost:8089)
    - TENANT_ID: Tenant identifier for multi-tenant isolation (default: test-tenant)
    
    Args:
        mode: Detection mode - 'off', 'rules_only', 'ner_only', or 'hybrid'
        
    Returns:
        str: Confirmation message with mode description
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    # Validate mode
    valid_modes = ["off", "rules_only", "ner_only", "hybrid"]
    if mode not in valid_modes:
        return (
            f"❌ Invalid mode: '{mode}'\n\n"
            f"**Valid modes:** {', '.join(valid_modes)}"
        )
    
    try:
        # Make HTTP POST request to Privacy Guard API
        response = requests.post(
            f"{guard_url}/guard/config",
            headers={"Content-Type": "application/json"},
            json={"mode": mode, "tenant_id": tenant_id},
            timeout=30,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Mode descriptions
        mode_descriptions = {
            "off": "PII detection disabled",
            "rules_only": "Regex patterns only (fast, lower accuracy)",
            "ner_only": "AI NER model only (slower, higher accuracy)",
            "hybrid": "Both regex + AI (recommended, best accuracy)",
        }
        
        return (
            f"✅ **Privacy Mode Updated**\n\n"
            f"**Mode:** {mode}\n"
            f"**Description:** {mode_descriptions.get(mode, 'Unknown')}\n"
            f"**Tenant:** {tenant_id}"
        )
    
    except requests.exceptions.HTTPError as e:
        status_code = e.response.status_code if e.response else None
        error_detail = e.response.text if e.response else str(e)
        
        return (
            f"❌ HTTP {status_code} Error\n\n"
            f"Privacy Guard API rejected the request:\n"
            f"{error_detail}"
        )
    
    except Exception as e:
        return f"❌ Unexpected error: {type(e).__name__}\n\n{str(e)}"
