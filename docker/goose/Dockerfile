# Goose Docker Image for Multi-Agent Testing
# Based on official tutorial: https://block.github.io/goose/docs/tutorials/goose-in-docker/
# Community discussion: https://github.com/block/goose/discussions/1496
#
# Key Design Decisions:
# - Use Ubuntu 24.04 (proven in community, 523MB size)
# - NO keyring support (keyring does NOT work in Docker Ubuntu)
# - ALL configuration via environment variables
# - Profile fetched from Controller API at runtime
# - Auto-configure Goose without interactive prompts

FROM ubuntu:24.04

LABEL maintainer="goose-orchestrator-project"
LABEL description="Goose AI agent with auto-configuration for multi-agent testing"

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Core dependencies
    curl \
    ca-certificates \
    jq \
    # Text editors (for debugging)
    nano \
    vim \
    # X11 libraries (required by Goose)
    libxcb1 \
    # Networking tools
    netcat-openbsd \
    # Python (for config generation script)
    python3 \
    python3-pip \
    python3-yaml \
    python3-requests \
    && rm -rf /var/lib/apt/lists/*

# Install Goose using official install script
# CONFIGURE=false prevents interactive setup (we'll configure via script)
RUN curl -fsSL https://github.com/block/goose/releases/download/stable/download_cli.sh | CONFIGURE=false bash

# Add Goose to PATH
ENV PATH="/root/.local/bin:$PATH"

# Create workspace directory
RUN mkdir -p /workspace
WORKDIR /workspace

# Copy entrypoint and config generation scripts
COPY docker/goose/docker-goose-entrypoint.sh /usr/local/bin/
COPY docker/goose/generate-goose-config.py /usr/local/bin/

# Make scripts executable
RUN chmod +x /usr/local/bin/docker-goose-entrypoint.sh \
    && chmod +x /usr/local/bin/generate-goose-config.py

# Copy and install Agent Mesh MCP extension
COPY src/agent-mesh /opt/agent-mesh
WORKDIR /opt/agent-mesh

# Install dependencies (without editable install)
RUN pip3 install --break-system-packages mcp>=1.20.0 requests>=2.32.5 pydantic>=2.12.3 python-dotenv>=1.0.1 pyyaml

# Make agent_mesh_server.py executable as a module
ENV PYTHONPATH="/opt/agent-mesh:$PYTHONPATH"

# Return to workspace
WORKDIR /workspace

# Environment variables (can be overridden at runtime)
# These will be set by docker-compose for each agent
ENV GOOSE_ROLE="" \
    CONTROLLER_URL="http://controller:8088" \
    PRIVACY_GUARD_PROXY_URL="http://privacy-guard-proxy:8090" \
    OPENROUTER_API_KEY="" \
    GOOSE_PROVIDER="openrouter" \
    GOOSE_MODEL="openai/gpt-4o-mini" \
    KEYCLOAK_URL="http://host.docker.internal:8080" \
    KEYCLOAK_REALM="dev" \
    KEYCLOAK_CLIENT_ID="goose-controller" \
    KEYCLOAK_CLIENT_SECRET=""

# Default command: start Goose session
# Entrypoint script will:
# 1. Fetch profile from Controller
# 2. Generate config.yaml
# 3. Start Goose session
CMD ["/usr/local/bin/docker-goose-entrypoint.sh"]
