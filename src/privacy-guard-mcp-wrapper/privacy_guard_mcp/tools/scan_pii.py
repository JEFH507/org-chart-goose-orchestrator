"""scan_pii MCP Tool

Detect PII in text using Privacy Guard service.
"""

import os
from typing import Any

from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests


class ScanPiiParams(BaseModel):
    """Parameters for the scan_pii tool."""
    
    text: str = Field(
        description="Text to scan for PII (e.g., 'My SSN is 123-45-6789 and email is john@example.com')"
    )
    mode: str = Field(
        default="hybrid",
        description="Detection mode: 'rules_only' (regex patterns), 'ner_only' (AI model), or 'hybrid' (both)"
    )


async def scan_pii_handler(params: ScanPiiParams) -> list[TextContent]:
    """
    Scan text for PII using Privacy Guard service.
    
    Features:
    - Regex-based pattern matching (SSN, email, phone, credit cards)
    - AI-powered NER detection (names, locations, organizations)
    - Hybrid mode combining both approaches
    
    Environment Variables:
    - PRIVACY_GUARD_URL: Privacy Guard API base URL (default: http://localhost:8089)
    - TENANT_ID: Tenant identifier for multi-tenant isolation (default: test-tenant)
    
    Args:
        params: ScanPiiParams with text and detection mode
        
    Returns:
        list[TextContent]: List of detected PII entities with categories and positions
    """
    # Get configuration from environment
    guard_url = os.getenv("PRIVACY_GUARD_URL", "http://localhost:8089")
    tenant_id = os.getenv("TENANT_ID", "test-tenant")
    
    try:
        # Make HTTP POST request to Privacy Guard API
        response = requests.post(
            f"{guard_url}/guard/scan",
            headers={
                "Content-Type": "application/json",
            },
            json={
                "text": params.text,
                "mode": params.mode,
                "tenant_id": tenant_id,
            },
            timeout=30,
        )
        
        # Raise exception for HTTP errors (4xx, 5xx)
        response.raise_for_status()
        
        # Parse JSON response
        data = response.json()
        
        # Extract findings
        findings = data.get("findings", [])
        
        if not findings:
            return [TextContent(
                type="text",
                text="✅ No PII detected in the provided text.\n\n"
                     f"**Scan Mode:** {params.mode}\n"
                     f"**Text Length:** {len(params.text)} characters"
            )]
        
        # Format findings
        result_lines = [
            f"🔍 **PII Detection Results** ({len(findings)} findings)\n",
            f"**Mode:** {params.mode}",
            f"**Tenant:** {tenant_id}\n",
        ]
        
        for i, finding in enumerate(findings, 1):
            category = finding.get("category", "unknown")
            text_value = finding.get("text", "")
            start = finding.get("start", 0)
            end = finding.get("end", 0)
            confidence = finding.get("confidence", 1.0)
            
            result_lines.append(
                f"{i}. **{category.upper()}**\n"
                f"   - Text: `{text_value}`\n"
                f"   - Position: {start}-{end}\n"
                f"   - Confidence: {confidence:.2%}"
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
                 f"**Troubleshooting:**\n"
                 f"- Check PRIVACY_GUARD_URL: {guard_url}\n"
                 f"- Verify Privacy Guard service is running\n"
                 f"- Check mode is valid: {params.mode}"
        )]
    
    except requests.exceptions.ConnectionError as e:
        return [TextContent(
            type="text",
            text=f"❌ Connection Error\n\n"
                 f"Could not connect to Privacy Guard service:\n"
                 f"{str(e)}\n\n"
                 f"**Troubleshooting:**\n"
                 f"1. Check Privacy Guard is running: `curl {guard_url}/health`\n"
                 f"2. Verify PRIVACY_GUARD_URL is correct: {guard_url}\n"
                 f"3. Check network connectivity"
        )]
    
    except Exception as e:
        return [TextContent(
            type="text",
            text=f"❌ Unexpected error: {type(e).__name__}\n\n"
                 f"{str(e)}"
        )]


# MCP Tool definition
scan_pii_tool = Tool(
    name="scan_pii",
    description=(
        "Detect PII (Personally Identifiable Information) in text using Privacy Guard service. "
        "Supports regex patterns (SSN, email, phone, credit cards) and AI-powered NER. "
        "Returns detailed findings with categories, positions, and confidence scores."
    ),
    inputSchema=ScanPiiParams.model_json_schema(),
)

# Attach handler to tool
scan_pii_tool.call = scan_pii_handler
