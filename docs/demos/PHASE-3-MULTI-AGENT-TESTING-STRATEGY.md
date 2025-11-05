# Phase 3 Multi-Agent Testing Strategy

**Document:** Comprehensive strategy for testing Agent Mesh MCP with multiple Goose profiles  
**Created:** 2025-11-04  
**Status:** READY FOR IMPLEMENTATION

---

## Executive Summary

### Problem Statement
Phase 3 requires testing cross-agent orchestration with multiple Goose instances representing different organizational roles (Finance Agent, Manager Agent, etc.). We need a robust, repeatable testing approach.

### Recommended Solution
**Use Goose CLI with session-specific extension loading and role-based shell scripts** (Goose v1.12 doesn't have traditional profiles).

### Key Decisions

✅ **Multi-Agent Approach:** Role-based shell scripts + session-specific extension loading  
✅ **Defer Phase 4 Items:** Session persistence, comprehensive Privacy Guard testing  
✅ **Focus Phase 3:** Prove orchestration pattern works (send_task → approval → notify)

---

## Understanding Goose "Profiles"

### What Goose v1.12 Has

**Global Configuration:**
- Single config file: `~/.config/goose/config.yaml`
- Extension settings: enabled/disabled, env vars, timeout
- No named profiles support (unlike AWS CLI or SSH config)

**Session-Specific Extension Loading:**
```bash
goose session --with-extension "ENV_VAR=value command args"
```

### What This Means for Multi-Agent Testing

❌ Cannot use: `goose --profile finance` (doesn't exist)  
✅ Can use: Role-based shell scripts with session-specific extensions  
✅ Can use: Separate terminals per agent role  
✅ Can use: Role-specific environment variables

---

## Recommended Multi-Agent Setup

### Architecture

```
Controller API (http://localhost:8088)
         ▲
         │
    ┌────┴────┐
    │         │
Finance    Manager
Agent      Agent
(Term1)    (Term2)
```

Each agent:
- Runs in separate terminal
- Uses Agent Mesh MCP extension
- Has role-specific JWT token
- Connects to shared Controller API

---

## Phase 4 Deferral Rationale

### ✅ Items to Defer

#### 1. Session Persistence (6 hours, Phase 4)

**Why Defer:**
- Stateless API proves orchestration works
- Infrastructure work, not core logic
- fetch_status HTTP 501 is expected/acceptable

#### 2. Privacy Guard Deep Testing (33 hours, Phase 4)

**Why Defer:**
- Guard operational with regex fallback
- MCP tools don't directly interact with Guard
- Too expensive for Phase 3 scope

#### 3. JWT Token Management (2 hours, Phase 4)

**Why Defer:**
- Manual token refresh acceptable for demo
- Production concern, not MVP validation

### ⚠️ NOT Deferred (Must Complete Phase 3)

- All 4 MCP tools functional ✅
- Controller API routes ✅
- Cross-agent demo (Finance → Manager)
- ADR-0024 and ADR-0025
- Smoke tests (5/5 passing)

---

## Implementation: Role-Based Session Scripts

See full implementation details in the complete document sections below.

**Summary:**
1. Create `scripts/start-finance-agent.sh` - Finance role session
2. Create `scripts/start-manager-agent.sh` - Manager role session  
3. Create `scripts/get-jwt-token.sh` - JWT helper
4. Update Agent Mesh README with multi-agent instructions

---

## ✅ RECOMMENDATIONS SUMMARY

### Question 1: How to Test Agent Mesh with Goose?

**Recommendation:** Use **role-based shell scripts** with session-specific extension loading.

**Rationale:**
- ✅ No profile management complexity
- ✅ Easy to add new roles (just copy script, change env vars)
- ✅ Scripts are version-controlled and repeatable
- ✅ Mimics realistic production deployment
- ✅ Works with Goose v1.12 architecture

**What You'll Need:**
- 2-3 terminal windows (one per agent role)
- Shell scripts for each role (finance, manager, etc.)
- Single JWT token (or role-specific tokens if needed)
- Controller API running in Docker

### Question 2: Defer Phase 4 Items?

**Recommendation:** **YES, defer all documented Phase 4 items.**

**Rationale:**
- ✅ Session persistence: Infrastructure work, can wait
- ✅ Privacy Guard testing: 33 hours too expensive for Phase 3
- ✅ JWT management: Manual refresh OK for demo
- ✅ Phase 3 focus: Prove orchestration works, not production hardening
- ✅ Total Phase 4 deferral: ~41 hours (saves 5+ days in Phase 3)

**What This Means:**
- `fetch_status` returns HTTP 501 → acceptable, documented
- Privacy Guard uses regex PII detection → acceptable, operational
- JWT tokens manually refreshed → acceptable, automated in Phase 4
- **Phase 3 completes faster, Phase 4 has clear scope**

### Question 3: Need Separate Goose Profiles?

**Answer:** Goose v1.12 **doesn't have traditional profiles**.

**Alternative Solution:**
- Create role-specific shell scripts
- Each script sets environment variables
- Use `goose session --with-extension` for temporary extension loading
- Run one script per terminal window

**Example:**
```bash
# Terminal 1
./scripts/start-finance-agent.sh  # Sets AGENT_ROLE=finance

# Terminal 2  
./scripts/start-manager-agent.sh  # Sets AGENT_ROLE=manager
```

---

## Next Steps for B8 (Deployment & Docs)

1. ✅ Create role-based shell scripts (3 scripts, ~30 min)
2. ✅ Update Agent Mesh README with multi-agent section (~15 min)
3. ⏸️ Test demo workflow (Finance → Manager approval) (~30 min)
4. ⏸️ Document results in cross-agent-approval.md (~30 min)
5. ⏸️ Create ADR-0024 (~30 min)
6. ⏸️ Update VERSION_PINS.md (~15 min)

**Total B8 Estimated:** 2.5 hours (was 4 hours, optimized)

---

**Status:** READY FOR YOUR APPROVAL  
**Next:** Create shell scripts and proceed with B8 testing?
