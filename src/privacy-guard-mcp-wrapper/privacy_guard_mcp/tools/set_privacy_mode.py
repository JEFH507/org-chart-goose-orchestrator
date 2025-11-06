"""set_privacy_mode MCP Tool

Configure Privacy Guard detection mode.
"""

import os
import requests


async def set_privacy_mode_handler(mode: str) -> str:
    """
    Query or explain Privacy Guard detection modes.
    
    **Note:** The Privacy Guard mode is currently set via environment variables
    (GUARD_MODE) when the service starts. This tool explains the available modes
    but cannot change them at runtime.
    
    Modes:
    - Detect: Scan for PII but don't modify text (detection only)
    - Mask: Detect and mask PII automatically (default)
    - Audit: Detection with detailed logging
    
    The detection method (regex vs AI NER) is controlled by GUARD_MODEL_ENABLED.
    
    Environment Variables:
    - PRIVACY_GUARD_URL: Privacy Guard API base URL (default: http://localhost:8089)
    - TENANT_ID: Tenant identifier for multi-tenant isolation (default: test-tenant)
    
    Args:
        mode: Mode to query about - 'detect', 'mask', or 'audit'
        
    Returns:
        str: Mode description and current status
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    # Mode descriptions
    mode_descriptions = {
        "detect": {
            "name": "Detect",
            "description": "Scan for PII but don't modify text (detection only)",
            "use_case": "When you need to identify PII without changing it",
        },
        "mask": {
            "name": "Mask",
            "description": "Detect and mask PII automatically (default)",
            "use_case": "When you need to protect PII in production data",
        },
        "audit": {
            "name": "Audit",
            "description": "Detection with detailed logging",
            "use_case": "When you need comprehensive audit trails",
        },
    }
    
    mode_lower = mode.lower()
    
    try:
        # Get current status
        response = requests.get(f"{guard_url}/status", timeout=30)
        response.raise_for_status()
        data = response.json()
        current_mode = data.get("mode", "unknown")
        
        if mode_lower in mode_descriptions:
            info = mode_descriptions[mode_lower]
            return (
                f"ℹ️ **{info['name']} Mode Information**\n\n"
                f"**Description:** {info['description']}\n"
                f"**Use Case:** {info['use_case']}\n\n"
                f"**Current Mode:** {current_mode}\n"
                f"**Tenant:** {tenant_id}\n\n"
                f"**Note:** To change the mode, update the GUARD_MODE environment variable "
                f"in the Privacy Guard service configuration and restart the service."
            )
        else:
            return (
                f"📋 **Available Privacy Guard Modes**\n\n"
                f"**Current Mode:** {current_mode}\n\n"
                + "\n\n".join(
                    f"**{info['name']}:** {info['description']}"
                    for info in mode_descriptions.values()
                )
                + f"\n\n**Note:** Mode is configured via GUARD_MODE environment variable."
            )
    
    except Exception as e:
        return (
            f"⚠️ **Mode Query Information**\n\n"
            f"**Requested:** {mode}\n\n"
            f"Available modes: Detect, Mask, Audit\n\n"
            f"**Note:** Could not query current status. Error: {type(e).__name__}"
        )
