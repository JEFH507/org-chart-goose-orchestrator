# Agent Mesh MCP Server - Development Container
# Based on Python 3.13-slim (validated in Phase 2.5)

FROM python:3.13-slim

WORKDIR /app

# Install system dependencies (if needed)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy project files
COPY pyproject.toml ./
COPY agent_mesh_server.py ./
COPY tools/ ./tools/
COPY tests/ ./tests/

# Install Python dependencies
RUN pip install --no-cache-dir -e .

# Optional: Install dev dependencies
RUN pip install --no-cache-dir -e ".[dev]"

# Set Python to unbuffered mode for better logging
ENV PYTHONUNBUFFERED=1

# Entry point for MCP server (stdio mode)
CMD ["python", "-m", "agent_mesh_server"]
