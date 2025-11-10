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


def generate_config(profile_json, provider, model, api_key, proxy_url, controller_url, mesh_jwt_token):
    """
    Generate Goose config.yaml from profile JSON.
    
    Args:
        profile_json (str): JSON string from Controller API
        provider (str): LLM provider name (e.g., "openrouter")
        model (str): LLM model name (e.g., "openai/gpt-4o-mini")
        api_key (str): API key (will be used as env var reference)
        proxy_url (str): Privacy Guard Proxy URL
        controller_url (str): Controller API URL (for agent_mesh)
        mesh_jwt_token (str): JWT token for agent_mesh authentication
    
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
                # Use Goose's stdio extension format (not "mcp" type)
                # Reference: https://block.github.io/goose/docs/getting-started/using-extensions
                if ext_name == "agent_mesh":
                    config["extensions"]["agent_mesh"] = {
                        "type": "stdio",  # Use "stdio" for MCP extensions
                        "cmd": "python3",  # Base command
                        "args": ["-m", "agent_mesh_server"],  # Command arguments
                        "enabled": True,
                        "timeout": 300,
                        "envs": {  # Pass actual values, not ${VAR} substitution
                            "CONTROLLER_URL": controller_url,
                            "MESH_JWT_TOKEN": mesh_jwt_token,
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
                # Use Goose's stdio extension format
                config["extensions"]["agent_mesh"] = {
                    "type": "stdio",
                    "cmd": "python3",
                    "args": ["-m", "agent_mesh_server"],
                    "enabled": True,
                    "timeout": 300,
                    "envs": {
                        "CONTROLLER_URL": controller_url,
                        "MESH_JWT_TOKEN": mesh_jwt_token,
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
        "--controller-url",
        required=True,
        help="Controller API URL for agent_mesh extension"
    )
    parser.add_argument(
        "--mesh-jwt-token",
        required=True,
        help="JWT token for agent_mesh authentication"
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
        args.proxy_url,
        args.controller_url,
        args.mesh_jwt_token
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
