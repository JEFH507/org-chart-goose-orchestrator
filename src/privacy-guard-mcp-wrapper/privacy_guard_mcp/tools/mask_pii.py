"""mask_pii MCP Tool

Mask PII in text using Privacy Guard service.
"""

import os
from typing import Any

from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests


class MaskPiiParams(BaseModel):
    """Parameters for the mask_pii tool."""
    
    text: str = Field(
        description="Text containing PII to mask (e.g., 'Contact John at john@example.com or 555-1234')"
    )
    method: str = Field(
        default="pseudonym",
        description="Masking method: 'fpe' (format-preserving encryption), 'pseudonym' (fake names), or 'redact' (remove)"
    )
    mode: str = Field(
        default="hybrid",
        description="Detection mode: 'rules_only', 'ner_only', or 'hybrid'"
    )


async def mask_pii_handler(params: MaskPiiParams) -> list[TextContent]:
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
        params: MaskPiiParams with text, method, and mode
        
    Returns:
        list[TextContent]: Masked text with summary of replacements
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    try:
        # Make HTTP POST request to Privacy Guard API
        response = requests.post(
            f"{guard_url}/guard/mask",
            headers={
                "Content-Type": "application/json",
            },
            json={
                "text": params.text,
                "method": params.method,
                "mode": params.mode,
                "tenant_id": tenant_id,
            },
            timeout=30,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Parse JSON response
        data = response.json()
        
        # Extract masked text and stats
        masked_text = data.get("masked_text", params.text)
        replacements = data.get("replacements", [])
        
        # Format result
        result_lines = [
            "✅ **PII Masking Complete**\n",
            f"**Method:** {params.method}",
            f"**Mode:** {params.mode}",
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
                 f"Privacy Guard API rejected the request:\n"
                 f"{error_detail}\n\n"
                 f"**Check:** method={params.method}, mode={params.mode}"
        )]
    
    except Exception as e:
        return [TextContent(
            type="text",
            text=f"❌ Unexpected error: {type(e).__name__}\n\n{str(e)}"
        )]


# MCP Tool definition
mask_pii_tool = Tool(
    name="mask_pii",
    description=(
        "Mask PII in text using Privacy Guard service. "
        "Supports FPE (format-preserving), pseudonyms, and redaction. "
        "Returns masked text with replacement summary."
    ),
    inputSchema=MaskPiiParams.model_json_schema(),
)

# Attach handler to tool
mask_pii_tool.call = mask_pii_handler
