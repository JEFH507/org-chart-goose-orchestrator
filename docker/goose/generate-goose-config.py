#!/usr/bin/env python3
"""
Generate Goose config.yaml from Controller profile JSON.

This script converts a profile fetched from the Controller API
into a Goose-compatible config.yaml file with environment variable
substitution for API keys (avoiding keyring issues in Docker).
"""

import argparse
import json
import sys
import yaml


def generate_config(profile_json, provider, model, api_key, proxy_url):
    """
    Generate Goose config.yaml from profile JSON.
    
    Args:
        profile_json (str): JSON string from Controller API
        provider (str): LLM provider name (e.g., "openrouter")
        model (str): LLM model name (e.g., "openai/gpt-4o-mini")
        api_key (str): API key (will be used as env var reference)
        proxy_url (str): Privacy Guard Proxy URL
    
    Returns:
        dict: Config dictionary ready to serialize to YAML
    """
    try:
        profile = json.loads(profile_json)
    except json.JSONDecodeError as e:
        print(f"ERROR: Failed to parse profile JSON: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Extract profile fields
    role = profile.get("role", "unknown")
    display_name = profile.get("display_name", "Unknown Role")
    
    # Build config structure
    config = {
        # Provider configuration
        "provider": provider,
        "model": model,
        
        # Use environment variable for API key (avoid keyring)
        # Goose will read from $OPENROUTER_API_KEY at runtime
        "api_key_env": "OPENROUTER_API_KEY",
        
        # Privacy Guard Proxy integration
        # Override api_base to route through proxy
        "api_base": f"{proxy_url}/v1",
        
        # Extensions from profile
        "extensions": {},
        
        # Additional settings
        "role": role,
        "display_name": display_name,
    }
    
    # Process extensions from profile
    profile_extensions = profile.get("extensions", [])
    for ext in profile_extensions:
        if isinstance(ext, dict):
            ext_name = ext.get("name")
            if ext_name:
                # Special handling for agent_mesh extension (needs MCP config)
                if ext_name == "agent_mesh":
                    config["extensions"]["agent_mesh"] = {
                        "type": "mcp",
                        "command": ["python3", "-m", "agent_mesh_server"],
                        "working_dir": "/opt/agent-mesh",
                        "env": {
                            "CONTROLLER_URL": "${CONTROLLER_URL}",
                            "MESH_JWT_TOKEN": "${MESH_JWT_TOKEN}",
                            "MESH_RETRY_COUNT": "3",
                            "MESH_TIMEOUT_SECS": "30"
                        }
                    }
                else:
                    # Other extensions use config from profile
                    config["extensions"][ext_name] = ext.get("config", {})
        elif isinstance(ext, str):
            # Simple extension name
            if ext == "agent_mesh":
                # Add MCP configuration for agent_mesh
                config["extensions"]["agent_mesh"] = {
                    "type": "mcp",
                    "command": ["python3", "-m", "agent_mesh_server"],
                    "working_dir": "/opt/agent-mesh",
                    "env": {
                        "CONTROLLER_URL": "${CONTROLLER_URL}",
                        "MESH_JWT_TOKEN": "${MESH_JWT_TOKEN}",
                        "MESH_RETRY_COUNT": "3",
                        "MESH_TIMEOUT_SECS": "30"
                    }
                }
            else:
                # Other extensions use defaults
                config["extensions"][ext] = {}
    
    # Add privacy settings if present in profile
    if "privacy" in profile:
        config["privacy"] = profile["privacy"]
    
    # Add policies if present in profile
    if "policies" in profile:
        config["policies"] = profile["policies"]
    
    return config


def main():
    parser = argparse.ArgumentParser(
        description="Generate Goose config.yaml from Controller profile"
    )
    parser.add_argument(
        "--profile",
        required=True,
        help="Profile JSON from Controller API"
    )
    parser.add_argument(
        "--provider",
        default="openrouter",
        help="LLM provider (default: openrouter)"
    )
    parser.add_argument(
        "--model",
        default="openai/gpt-4o-mini",
        help="LLM model (default: openai/gpt-4o-mini)"
    )
    parser.add_argument(
        "--api-key",
        required=True,
        help="API key for LLM provider"
    )
    parser.add_argument(
        "--proxy-url",
        default="http://privacy-guard-proxy:8090",
        help="Privacy Guard Proxy URL (default: http://privacy-guard-proxy:8090)"
    )
    parser.add_argument(
        "--output",
        default="config.yaml",
        help="Output file path (default: config.yaml)"
    )
    
    args = parser.parse_args()
    
    # Generate config
    config = generate_config(
        args.profile,
        args.provider,
        args.model,
        args.api_key,
        args.proxy_url
    )
    
    # Write to YAML file
    try:
        with open(args.output, 'w') as f:
            yaml.dump(config, f, default_flow_style=False, sort_keys=False)
        print(f"✓ Config written to {args.output}", file=sys.stderr)
    except Exception as e:
        print(f"ERROR: Failed to write config: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
