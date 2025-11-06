"""mask_pii MCP Tool

Mask PII in text using Privacy Guard service.
"""

import os
import requests


async def mask_pii_handler(text: str, method: str = "pseudonym", mode: str = "hybrid") -> str:
    """
    Mask PII in text using Privacy Guard service.
    
    Features:
    - FPE (Format-Preserving Encryption): Maintains data format
    - Pseudonym: Replaces with realistic fake values
    - Redaction: Removes or replaces with [REDACTED]
    
    Environment Variables:
    - PRIVACY_GUARD_URL: Privacy Guard API base URL (default: http://localhost:8089)
    - TENANT_ID: Tenant identifier for multi-tenant isolation (default: test-tenant)
    
    Args:
        text: Text containing PII to mask
        method: Masking method - 'fpe', 'pseudonym', or 'redact'
        mode: Detection mode - 'rules_only', 'ner_only', or 'hybrid'
        
    Returns:
        str: Masked text with replacement summary
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    try:
        # Make HTTP POST request to Privacy Guard API
        response = requests.post(
            f"{guard_url}/guard/mask",
            headers={"Content-Type": "application/json"},
            json={
                "text": text,
                "method": method,
                "mode": mode,
                "tenant_id": tenant_id,
            },
            timeout=30,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Parse JSON response
        data = response.json()
        masked_text = data.get("masked_text", text)
        replacements = data.get("replacements", [])
        
        # Format result
        result_lines = [
            "✅ **PII Masking Complete**\n",
            f"**Method:** {method}",
            f"**Mode:** {mode}",
            f"**Replacements:** {len(replacements)}\n",
            "**Masked Text:**",
            f"```\n{masked_text}\n```\n",
        ]
        
        if replacements:
            result_lines.append("**Replacement Details:**")
            for i, rep in enumerate(replacements, 1):
                category = rep.get("category", "unknown")
                original = rep.get("original", "")
                masked = rep.get("masked", "")
                result_lines.append(
                    f"{i}. {category.upper()}: `{original}` → `{masked}`"
                )
        
        return "\n".join(result_lines)
    
    except requests.exceptions.HTTPError as e:
        status_code = e.response.status_code if e.response else None
        error_detail = e.response.text if e.response else str(e)
        
        return (
            f"❌ HTTP {status_code} Error\n\n"
            f"Privacy Guard API rejected the request:\n"
            f"{error_detail}\n\n"
            f"**Check:** method={method}, mode={mode}"
        )
    
    except Exception as e:
        return f"❌ Unexpected error: {type(e).__name__}\n\n{str(e)}"
