# Dependency LTS Version Research
**Date:** 2025-11-04  
**Purpose:** Identify latest LTS/stable versions for Phase 2.5 upgrade

---

## Research Summary

### 1. Vault (HashiCorp)

**Current:** 1.17.6 (Jul 2024)

**Latest LTS:** ✅ **1.18.3** (Nov 2024)
- Source: https://developer.hashicorp.com/vault/docs/updates/lts-tracker
- EOL: TBD (18 months typical)
- Status: Current LTS track

**Key Changes 1.17.6 → 1.18.3:**
- Security patches
- Better KMS support
- Backward compatible API

---

### 2. PostgreSQL

**Current:** 16.4 (Aug 2024)

**Latest Stable:** ✅ **17.2** (Nov 2024)
- Source: https://www.postgresql.org/
- LTS: Yes (5 years until 2029)
- Major version upgrade (16 → 17)

**Key Changes:**
- Better JSON handling
- Improved VACUUM
- Security patches
- **Safe:** No breaking changes for our basic CRUD usage

**Docker:** `postgres:17.2-alpine`

---

### 3. Keycloak

**Current:** 24.0.4 (Apr 2024)

**Latest Stable:** ✅ **26.0.4** (Nov 2024)
- Source: https://www.keycloak.org/docs/latest/release_notes/

**CRITICAL Security Fixes:**
- CVE-2024-8883 (HIGH) - Session fixation
- CVE-2024-7318 (MEDIUM) - Auth bypass
- CVE-2024-8698 (MEDIUM) - XSS

**Breaking Changes:** ❌ None for OIDC/JWT

**Docker:** `quay.io/keycloak/keycloak:26.0.4`

---

### 4. Ollama

**Current (VERSION_PINS.md):** 0.12.9

**Latest Stable:** ✅ **v0.12.9** (Oct 31, 2025)
- Source: https://github.com/ollama/ollama/releases/latest
- Verified: API confirms tag_name "v0.12.9"
- Status: **UP-TO-DATE** ✅

**Action:** KEEP 0.12.9 (already latest, qwen3:0.6b support validated in Phase 2.2)

**Docker:** `ollama/ollama:0.12.9`

---

## Development Tools (Phase 3+)

### 5. Python Runtime

**Current (System):** 3.12.3

**Latest Stable:** ✅ **3.13.9** (Nov 4, 2025)
- Source: https://devguide.python.org/versions/
- EOL: 2029-10 (5-year support)
- Use: Agent Mesh MCP server (Phase 3)

**Docker:** `python:3.13-slim`

**Action:** Use Docker image for consistency (system Python 3.12.3 compatible but not preferred)

---

### 6. Rust Toolchain

**Current (Docker - Local):** 1.83.0 (rust:1.83-bookworm)

**Latest Stable:** ✅ **1.91.0** (Oct 28, 2025)
- Source: https://releases.rs/ + https://static.rust-lang.org/dist/channel-rust-stable.toml
- Release Cycle: 6-week rolling stable
- Use: Controller API, Privacy Guard, Rust MCP extensions

**Docker:** `rust:1.91.0-bookworm` or `rust:1.91.0-slim`

**Action:** UPGRADE from 1.83.0 → 1.91.0 (8 minor versions behind)

---

## Final Upgrade Matrix

| Component | Current | Upgrade To | Priority | Type |
|-----------|---------|------------|----------|------|
| **Keycloak** | 24.0.4 | **26.0.4** | 🔴 HIGH | Runtime (Security CVEs) |
| **Vault** | 1.17.6 | **1.18.3** | 🟡 MEDIUM | Runtime (LTS) |
| **Postgres** | 16.4 | **17.2** | 🟢 LOW | Runtime (Performance) |
| **Ollama** | 0.12.9 | **0.12.9** | ✅ KEEP | Runtime (Already latest) |
| **Python** | 3.12.3 | **3.13.9** (Docker) | 🟡 MEDIUM | Dev Tool (Phase 3) |
| **Rust** | 1.83.0 | **1.91.0** (Docker) | 🟡 MEDIUM | Dev Tool (8 versions behind) |

**Total Effort:** ~6 hours (0.75 days)
