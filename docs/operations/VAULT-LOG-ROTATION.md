# Vault Audit Log Rotation - Production Guide

## Overview

Vault audit logs must be rotated to prevent disk space exhaustion and maintain compliance with log retention policies. This guide covers **two production-ready approaches**.

---

## 🎯 Recommended Approach: Docker Native Logging

**Best for:** Cloud deployments, Kubernetes, Docker Swarm

### Configuration (Already Enabled)

The `deploy/compose/ce.dev.yml` includes Docker-level log rotation on the vault service:

```yaml
vault:
  logging:
    driver: "json-file"
    options:
      max-size: "10m"     # Rotate when log reaches 10MB
      max-file: "30"      # Keep 30 files (~30 days)
      compress: "true"    # Compress rotated logs (gzip)
```

### How It Works

1. **Docker daemon** manages log rotation automatically
2. When container logs reach 10MB → rotated
3. Old logs compressed: `vault.log.1.gz`, `vault.log.2.gz`, ...
4. Keeps 30 files maximum (oldest deleted automatically)
5. **No cron jobs needed** - handled by Docker

### Verification

```bash
# Check Docker logging configuration
docker inspect ce_vault | jq '.[0].HostConfig.LogConfig'

# View container logs (with rotation)
docker logs ce_vault --tail 100
```

### Advantages

✅ **Cloud-native** - Works in Kubernetes, ECS, Docker Swarm  
✅ **No host dependencies** - No logrotate installation needed  
✅ **Automatic** - Docker daemon handles everything  
✅ **Portable** - Same config across dev/staging/prod

---

## 🛠️ Alternative Approach: System Logrotate (For Audit Logs in Volumes)

**Best for:** Traditional VM deployments, when you need to rotate files inside Docker volumes

### Setup (One-Time)

Run the setup script with sudo:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
sudo ./scripts/setup-vault-logrotate.sh
```

This installs `/etc/logrotate.d/vault-audit` configuration.

### Verification

```bash
# Test configuration (dry-run)
sudo logrotate -d /etc/logrotate.d/vault-audit

# Force rotation (manual test)
sudo logrotate -f /etc/logrotate.d/vault-audit

# Check rotated logs
docker exec ce_vault ls -lh /vault/logs/
```

---

## 📋 Production Recommendations

**Current Setup (Phase 6 A4):**
- ✅ Docker native logging configured (max-size: 10m, max-file: 30)
- ✅ Handles container STDOUT/STDERR logs automatically
- ✅ vault_logs volume for audit log persistence
- ✅ Logrotate script available for host-level management

**For Production Deployments:**
1. Docker logging handles container logs (already configured)
2. Optional: Run `setup-vault-logrotate.sh` for volume-based audit log rotation
3. Optional: Forward logs to centralized logging (ELK, Splunk, CloudWatch)

---

**Last Updated:** 2025-11-07  
**Phase:** 6 (Production Hardening)  
**Task:** A4 (Audit Device)
