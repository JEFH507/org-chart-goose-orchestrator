#!/usr/bin/env bash
# Setup environment configuration for docker-compose
# This script ensures .env.ce is properly configured and symlinked

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_DIR="$PROJECT_ROOT/deploy/compose"

echo "=== Environment Setup for Docker Compose ==="
echo

# Step 1: Check if .env.ce exists
if [ ! -f "$COMPOSE_DIR/.env.ce" ]; then
    echo "❌ .env.ce not found. Creating from .env.ce.example..."
    cp "$COMPOSE_DIR/.env.ce.example" "$COMPOSE_DIR/.env.ce"
    echo "✅ Created $COMPOSE_DIR/.env.ce"
    echo
    echo "⚠️  IMPORTANT: You must update the following values in .env.ce:"
    echo "   - OIDC_CLIENT_SECRET (get from Keycloak UI: Clients → goose-controller → Credentials)"
    echo "   - DATABASE_URL should be: postgresql://postgres:postgres@postgres:5432/orchestrator"
    echo
    read -p "Press Enter after you've updated .env.ce..."
else
    echo "✅ .env.ce exists"
fi

# Step 2: Verify critical variables are set
echo
echo "Checking critical environment variables..."

# Read .env.ce (simple check - doesn't handle complex bash substitution)
if ! grep -q "^DATABASE_URL=.*orchestrator" "$COMPOSE_DIR/.env.ce"; then
    echo "⚠️  WARNING: DATABASE_URL should point to 'orchestrator' database"
    echo "   Current: $(grep "^DATABASE_URL=" "$COMPOSE_DIR/.env.ce" || echo "NOT SET")"
    echo "   Expected: DATABASE_URL=postgresql://postgres:postgres@postgres:5432/orchestrator"
fi

if grep -q "^OIDC_CLIENT_SECRET=CHANGE_ME" "$COMPOSE_DIR/.env.ce"; then
    echo "⚠️  WARNING: OIDC_CLIENT_SECRET is still set to placeholder value"
    echo "   Get the actual value from Keycloak UI: Clients → goose-controller → Credentials"
fi

# Step 3: Create symlink for docker-compose auto-loading
echo
if [ -L "$COMPOSE_DIR/.env" ]; then
    CURRENT_TARGET="$(readlink "$COMPOSE_DIR/.env")"
    if [ "$CURRENT_TARGET" = ".env.ce" ]; then
        echo "✅ Symlink .env → .env.ce already exists"
    else
        echo "⚠️  Symlink .env exists but points to: $CURRENT_TARGET"
        echo "   Updating to point to .env.ce..."
        rm "$COMPOSE_DIR/.env"
        ln -s .env.ce "$COMPOSE_DIR/.env"
        echo "✅ Updated symlink"
    fi
elif [ -e "$COMPOSE_DIR/.env" ]; then
    echo "⚠️  WARNING: .env exists as a regular file (not a symlink)"
    echo "   Backing up to .env.backup and creating symlink..."
    mv "$COMPOSE_DIR/.env" "$COMPOSE_DIR/.env.backup"
    ln -s .env.ce "$COMPOSE_DIR/.env"
    echo "✅ Created symlink (old file backed up to .env.backup)"
else
    echo "Creating symlink .env → .env.ce..."
    cd "$COMPOSE_DIR"
    ln -s .env.ce .env
    cd "$PROJECT_ROOT"
    echo "✅ Created symlink"
fi

echo
echo "=== Setup Complete ==="
echo
echo "Docker Compose will now automatically load .env.ce via the .env symlink"
echo
echo "To verify your configuration:"
echo "  cd deploy/compose && docker compose -f ce.dev.yml config | grep -A5 OIDC_ISSUER_URL"
echo
echo "To start services with environment loaded:"
echo "  cd deploy/compose && docker compose -f ce.dev.yml --profile controller --profile redis up -d"
