#!/bin/bash
# Setup Vault audit log rotation
# This script configures log rotation for Vault audit logs
# Run once during deployment setup

set -e

echo "🔄 Setting up Vault Audit Log Rotation"
echo ""

# Check if running as root/sudo
if [ "$EUID" -ne 0 ]; then 
    echo "⚠️  This script needs sudo/root to configure system logrotate"
    echo "   Re-running with sudo..."
    exec sudo "$0" "$@"
fi

# Configuration
LOGROTATE_CONF="/etc/logrotate.d/vault-audit"
VAULT_LOG_PATH="/var/lib/docker/volumes/compose_vault_logs/_data/*.log"

echo "📁 Installing logrotate configuration to: $LOGROTATE_CONF"

# Create logrotate configuration
cat > "$LOGROTATE_CONF" << 'EOF'
# Vault Audit Log Rotation
# Rotates daily, keeps 30 days, compresses old logs

/var/lib/docker/volumes/compose_vault_logs/_data/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    missingok
    # Don't error if file is missing
    create 0644 100 100
    # UID 100 = vault user inside container
    sharedscripts
    postrotate
        # Vault automatically handles log file rotation
        # No HUP signal needed
    endscript
}
EOF

echo "✅ Logrotate configuration installed"
echo ""

# Test the configuration
echo "🧪 Testing logrotate configuration..."
if logrotate -d "$LOGROTATE_CONF" 2>&1 | grep -q "error"; then
    echo "❌ Logrotate configuration test failed"
    cat "$LOGROTATE_CONF"
    exit 1
else
    echo "✅ Logrotate configuration valid"
fi

echo ""
echo "📋 Summary:"
echo "   Config file: $LOGROTATE_CONF"
echo "   Log path: $VAULT_LOG_PATH"
echo "   Rotation: Daily"
echo "   Retention: 30 days"
echo "   Compression: Yes (gzip)"
echo ""
echo "📌 Logrotate will run daily via cron (automatic)"
echo "   Manual test: sudo logrotate -f $LOGROTATE_CONF"
echo ""
echo "✅ Vault audit log rotation configured!"
