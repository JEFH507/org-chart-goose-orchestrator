# Administrator Guide

**Version:** v0.5.0  
**Last Updated:** 2025-11-07  
**Target Audience:** Enterprise system administrators, DevOps engineers

---

## Overview

The goose Organization Orchestrator provides enterprise-grade AI agent management with:
- **Role-based profiles** (Finance, Manager, Analyst, Marketing, Support, Legal)
- **Privacy Guard** (PII detection and masking)
- **Organizational hierarchy** (CSV import, tree visualization)
- **Audit logging** (compliance and security tracking)
- **JWT authentication** (Keycloak OIDC integration)
- **Vault integration** (secret management)

### Quick Links

- [Getting Started](#getting-started) - Installation and setup
- [User Management](#user-management) - Create users, import org chart
- [Security](#security) - JWT auth, Vault, network security
- [Monitoring](#monitoring) - Logs, metrics, health checks
- [Troubleshooting](#troubleshooting) - Common issues and solutions
- [Backup & Recovery](#backup--recovery) - Database backups, restore procedures

---

## Getting Started

### Prerequisites

**Hardware:**
- Minimum: 4 CPU cores, 8GB RAM, 20GB disk
- Recommended: 8 CPU cores, 16GB RAM, 50GB disk

**Software:**
- Docker 24.0+ with Compose V2
- Git
- curl/jq (for testing)

### Installation

#### 1. Clone Repository

```bash
git clone https://github.com/JEFH507/org-chart-goose-orchestrator.git
cd org-chart-goose-orchestrator
```

#### 2. Configure Environment

```bash
cd deploy/compose
cp .env.ce.example .env.ce
nano .env.ce
```

**Required Variables:**
```bash
# Controller
CONTROLLER_PORT=8088
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/postgres

# Keycloak OIDC
OIDC_ISSUER_URL=http://localhost:8080/realms/goose
OIDC_JWKS_URL=http://localhost:8080/realms/goose/protocol/openid-connect/certs
OIDC_AUDIENCE=controller-api
OIDC_CLIENT_SECRET=your-keycloak-client-secret

# Vault
VAULT_ADDR=http://vault:8200
VAULT_TOKEN=root  # CHANGE IN PRODUCTION

# Privacy Guard
GUARD_PORT=8089
GUARD_MODE=MASK
PSEUDO_SALT=$(openssl rand -base64 32)  # Generate secure value
```

#### 3. Start Services

```bash
# Start core stack
docker-compose -f ce.dev.yml --profile controller up -d

# Verify health
docker-compose ps
```

#### 4. Initialize Keycloak

```bash
# Access admin console
open http://localhost:8080/admin
# Login: admin / admin

# Create client:
# - Client ID: controller-api
# - Client type: confidential
# - Valid redirect URIs: http://localhost:8088/*
# - Copy client secret → Update .env.ce → Restart controller
```

#### 5. Run Database Migrations

```bash
docker exec -it ce_postgres psql -U postgres << 'EOF'
-- profiles table
CREATE TABLE IF NOT EXISTS profiles (
  role VARCHAR(50) PRIMARY KEY,
  display_name VARCHAR(200) NOT NULL,
  description TEXT,
  goosehints JSONB NOT NULL,
  gooseignore JSONB NOT NULL,
  policies JSONB NOT NULL,
  privacy JSONB NOT NULL,
  extensions JSONB NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- org_users table
CREATE TABLE IF NOT EXISTS org_users (
  user_id VARCHAR(50) PRIMARY KEY,
  reports_to_id VARCHAR(50) REFERENCES org_users(user_id),
  name VARCHAR(200) NOT NULL,
  role VARCHAR(50) NOT NULL REFERENCES profiles(role),
  email VARCHAR(200) UNIQUE NOT NULL,
  department VARCHAR(100) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- privacy_audit_logs table
CREATE TABLE IF NOT EXISTS privacy_audit_logs (
  id SERIAL PRIMARY KEY,
  session_id VARCHAR(100),
  tenant_id VARCHAR(100),
  redaction_count INTEGER,
  categories TEXT[],
  mode VARCHAR(20),
  timestamp TIMESTAMP DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_org_users_role ON org_users(role);
CREATE INDEX idx_org_users_department ON org_users(department);
EOF
```

#### 6. Verify Installation

```bash
# Test Controller
curl http://localhost:8088/status
# Expected: {"status":"healthy","version":"0.5.0"}

# Test Keycloak
curl http://localhost:8080/realms/goose/.well-known/openid-configuration
# Expected: JSON with issuer, jwks_uri

# Test Vault
curl http://localhost:8200/v1/sys/health
# Expected: {"initialized":true,"sealed":false}
```

---

## User Management

### Import Organizational Chart (CSV)

#### CSV Format

```csv
user_id,reports_to_id,name,role,email,department
usr_001,,Alice CEO,manager,alice@company.com,Executive
usr_002,usr_001,Bob CFO,finance,bob@company.com,Finance
usr_003,usr_001,Carol CTO,manager,carol@company.com,Engineering
```

**Field Descriptions:**
- `user_id`: Unique ID (required, max 50 chars)
- `reports_to_id`: Manager's user_id (empty for CEO/root)
- `name`: Full name (required, max 200 chars)
- `role`: Role profile (finance, manager, analyst, marketing, support, legal)
- `email`: Email address (required, unique)
- `department`: Department name (required)

#### Import via API

```bash
# Get JWT token
TOKEN=$(curl -X POST http://localhost:8080/realms/goose/protocol/openid-connect/token \
  -d "client_id=controller-api" \
  -d "client_secret=your-secret" \
  -d "grant_type=client_credentials" \
  | jq -r '.access_token')

# Upload CSV
curl -X POST http://localhost:8088/admin/org/import \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: text/csv" \
  --data-binary @org_chart.csv

# Response:
{
  "import_id": 1,
  "user_count": 3,
  "uploaded_by": "admin",
  "status": "completed"
}
```

#### Verify Import

```bash
# Get org tree
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8088/admin/org/tree | jq .

# Query database
docker exec -it ce_postgres psql -U postgres -c \
  "SELECT user_id, name, role, department FROM org_users;"
```

### Create Users in Keycloak

#### Via Admin Console

1. Access http://localhost:8080/admin
2. Navigate to **Users** → **Add user**
3. Fill details:
   - Username: `alice.finance`
   - Email: `alice@company.com`
   - First/Last name
4. Create user
5. Set password in **Credentials** tab
6. Assign role in **Role Mappings**

#### Via REST API

```bash
# Get admin token
ADMIN_TOKEN=$(curl -X POST http://localhost:8080/realms/master/protocol/openid-connect/token \
  -d "client_id=admin-cli" \
  -d "username=admin" \
  -d "password=admin" \
  -d "grant_type=password" \
  | jq -r '.access_token')

# Create user
curl -X POST http://localhost:8080/admin/realms/goose/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "bob.manager",
    "email": "bob@company.com",
    "enabled": true,
    "credentials": [{
      "type": "password",
      "value": "SecurePassword123!",
      "temporary": false
    }]
  }'
```

---

## Security

### JWT Authentication

#### Get JWT Token

```bash
# Client credentials (service-to-service)
TOKEN=$(curl -X POST http://localhost:8080/realms/goose/protocol/openid-connect/token \
  -d "client_id=controller-api" \
  -d "client_secret=your-secret" \
  -d "grant_type=client_credentials" \
  | jq -r '.access_token')

# Password flow (user auth)
TOKEN=$(curl -X POST http://localhost:8080/realms/goose/protocol/openid-connect/token \
  -d "client_id=goose-desktop" \
  -d "username=alice.finance" \
  -d "password=SecurePass!" \
  -d "grant_type=password" \
  | jq -r '.access_token')

# Decode token
echo $TOKEN | cut -d. -f2 | base64 -d | jq .
```

### Vault Secret Management

#### Store Secrets

```bash
# Store API key
docker exec ce_vault vault kv put secret/openrouter api_key=sk-or-v1-abc123

# Retrieve secret
docker exec ce_vault vault kv get -field=api_key secret/openrouter

# Store Privacy Guard salt
docker exec ce_vault vault kv put secret/privacy-guard \
  salt=$(openssl rand -base64 32)
```

### Network Security

#### Restrict External Access (Production)

```yaml
# docker-compose.yml
services:
  controller:
    # Remove port exposure (internal only)
    networks:
      - internal
  
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"  # Only nginx exposed
    networks:
      - internal
      - external
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
```

---

## Monitoring

### Health Checks

```bash
# Controller
curl http://localhost:8088/status
# Expected: {"status":"healthy","version":"0.5.0"}

# Privacy Guard
curl http://localhost:8089/status
# Expected: {"status":"healthy","mode":"Hybrid"}

# Database
docker exec -it ce_postgres pg_isready -U postgres
# Expected: accepting connections
```

### Logs

```bash
# Controller logs
docker logs ce_controller

# Filter errors
docker logs ce_controller | grep -i error

# Follow logs (real-time)
docker logs -f ce_controller

# Last 100 lines
docker logs --tail 100 ce_controller
```

### Metrics

#### Database Metrics

```bash
# Connection count
docker exec -it ce_postgres psql -U postgres -c \
  "SELECT count(*) FROM pg_stat_activity;"

# Database size
docker exec -it ce_postgres psql -U postgres -c \
  "SELECT pg_size_pretty(pg_database_size('postgres'));"

# Table sizes
docker exec -it ce_postgres psql -U postgres << 'EOF'
SELECT tablename, pg_size_pretty(pg_total_relation_size(tablename::text))
FROM pg_tables WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(tablename::text) DESC;
EOF
```

#### Privacy Audit Metrics

```bash
# Total PII redactions
docker exec -it ce_postgres psql -U postgres -c \
  "SELECT COUNT(*) FROM privacy_audit_logs;"

# Top PII categories
docker exec -it ce_postgres psql -U postgres << 'EOF'
SELECT unnest(categories) AS category, COUNT(*) AS count
FROM privacy_audit_logs
GROUP BY category ORDER BY count DESC;
EOF
```

---

## Backup & Recovery

### Database Backups

#### Manual Backup

```bash
# Full backup
docker exec ce_postgres pg_dumpall -U postgres > backup_$(date +%Y%m%d).sql

# Compressed backup
docker exec ce_postgres pg_dump -U postgres postgres | gzip > \
  backup_$(date +%Y%m%d).sql.gz
```

#### Automated Backups (Cron)

```bash
# Daily backup at 2 AM
crontab -e
0 2 * * * docker exec ce_postgres pg_dumpall -U postgres | gzip > \
  /backups/postgres_$(date +\%Y\%m\%d).sql.gz
```

#### Restore from Backup

```bash
# Stop services
docker-compose stop controller

# Restore
gunzip -c backup_20251107.sql.gz | \
  docker exec -i ce_postgres psql -U postgres

# Restart services
docker-compose start controller
```

### Vault Backups

```bash
# Export secrets (for migration)
docker exec ce_vault vault kv get -format=json secret/ > \
  vault_backup_$(date +%Y%m%d).json
```

---

## Troubleshooting

### Controller Not Starting

**Symptom:**
```bash
docker logs ce_controller
# Error: Failed to connect to database
```

**Solution:**
```bash
# Check Postgres health
docker ps | grep postgres

# Test connection
docker exec -it ce_postgres psql -U postgres -c "SELECT 1;"

# Restart services
docker-compose restart postgres controller
```

### JWT Validation Failing

**Symptom:**
```bash
curl -H "Authorization: Bearer $TOKEN" http://localhost:8088/profiles/finance
# Response: 401 Unauthorized
```

**Solution:**
```bash
# Decode token
echo $TOKEN | cut -d. -f2 | base64 -d | jq .

# Check expiration (exp > current time)
# Check issuer matches OIDC_ISSUER_URL

# Get fresh token
TOKEN=$(curl -X POST http://localhost:8080/realms/goose/protocol/openid-connect/token \
  -d "client_id=controller-api" \
  -d "client_secret=your-secret" \
  -d "grant_type=client_credentials" \
  | jq -r '.access_token')
```

### High Database CPU Usage

**Solution:**
```bash
# Add indexes
docker exec -it ce_postgres psql -U postgres << 'EOF'
CREATE INDEX idx_privacy_audit_timestamp ON privacy_audit_logs(timestamp DESC);
ANALYZE;
EOF

# Vacuum database
docker exec -it ce_postgres psql -U postgres -c "VACUUM ANALYZE;"
```

---

## Additional Resources

- **Privacy Guard HTTP API:** `docs/privacy/PRIVACY-GUARD-HTTP-API.md`
- **Privacy Guard MCP:** `docs/privacy/PRIVACY-GUARD-MCP.md`
- **OpenAPI Spec:** `docs/api/openapi-v0.5.0.yaml`
- **Profile Spec:** `docs/profiles/SPEC.md`
- **Test Results:** `docs/tests/phase5-test-results.md`
- **Migration Guide:** `docs/MIGRATION-PHASE5.md`

---

## Support

- **GitHub Issues:** https://github.com/JEFH507/org-chart-goose-orchestrator/issues
- **Contact:** Javier (132608441+JEFH507@users.noreply.github.com)
- **License:** Apache-2.0

---

**Last Updated:** 2025-11-07  
**Version:** v0.5.0 (Grant Application Ready)
