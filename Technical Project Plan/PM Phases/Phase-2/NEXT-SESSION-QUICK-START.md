# Phase 2 — Next Session Quick Start

**Task:** D3 Smoke Test Execution  
**Estimated Time:** 1-2 hours  
**Date:** Ready for execution

---

## 🚀 Quick Resume (Copy & Paste)

```markdown
Resume Phase 2 - D3 Smoke Test Execution

**Status:** D3 documentation complete, execution pending (90% Phase 2 complete)

**Read First:**
- Technical Project Plan/PM Phases/Phase-2/D3-EXECUTION-HANDOFF.md (complete guide)
- docs/tests/smoke-phase2.md (test procedure)

**Execute:**
1. `cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose`
2. `docker compose --profile privacy-guard up -d`
3. Follow smoke-phase2.md tests 1-12 (minimum 1-10)
4. Run benchmark script (Test 10)
5. Record results and update tracking
6. Proceed to D4

**Branch:** docs/phase2-guides (already checked out)
```

---

## 📋 Execution Checklist

### Pre-Flight
- [ ] Navigate to project: `cd /home/papadoc/Gooseprojects/goose-org-twin`
- [ ] Check current branch: `git branch --show-current` (should be docs/phase2-guides)
- [ ] Read handoff: `Technical Project Plan/PM Phases/Phase-2/D3-EXECUTION-HANDOFF.md`

### Service Startup
- [ ] Navigate to compose: `cd deploy/compose`
- [ ] Verify .env.ce has PSEUDO_SALT: `grep PSEUDO_SALT .env.ce`
- [ ] Start services: `docker compose --profile privacy-guard up -d`
- [ ] Wait for healthy: `docker compose ps` (all services should show "healthy")

### Execute Tests (Required: 1-10)
- [ ] Test 1: Healthcheck - `curl http://localhost:8089/status`
- [ ] Test 2: PII Detection - POST /guard/scan
- [ ] Test 3: Masking - POST /guard/mask (email, IP)
- [ ] Test 4: FPE Phone - Verify format preservation
- [ ] Test 5: FPE SSN - Verify last-4 preservation
- [ ] Test 6: Determinism - Same input twice
- [ ] Test 7: Tenant Isolation - Different tenants
- [ ] Test 8: Reidentification - With JWT (requires Phase 1.2 JWT)
- [ ] Test 9: Audit Logs - Verify no PII in logs
- [ ] Test 10: Performance - Run benchmark script

### Execute Tests (Optional: 11-12)
- [ ] Test 11: Controller Integration (if GUARD_ENABLED=true)
- [ ] Test 12: Flush Session State

### Record Results
- [ ] Update smoke-phase2.md results table (PASS/FAIL for each test)
- [ ] Record P50/P95/P99 metrics in smoke-phase2.md
- [ ] Update state JSON performance_results with actual values
- [ ] Update state JSON checklist.D3 = "done"
- [ ] Update state JSON current_task_id = "D4"
- [ ] Clear pending_questions in state JSON
- [ ] Add progress log entry with results
- [ ] Update checklist with execution items checked

### Commit & Continue
- [ ] Commit tracking updates: `git commit -m "test(phase2): D3 execution complete - smoke tests results"`
- [ ] Proceed to D4: Update Project Docs

---

## 🎯 Success Criteria

**Minimum to proceed:**
- 8/10 required tests PASS
- Performance measured (even if targets missed)
- No PII in logs verified

**Ideal:**
- 10/10 required tests PASS
- P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s
- All optional tests PASS or SKIP with reason

---

## 📊 Performance Benchmark Commands

From `deploy/compose/` directory:

```bash
# Create benchmark script
cat > bench_guard.sh << 'SCRIPT'
#!/bin/bash
for i in {1..100}; do
  start=$(date +%s%N)
  curl -s -X POST http://localhost:8089/guard/mask \
    -H 'Content-Type: application/json' \
    -d '{
      "text": "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789. Credit card: 4532015112830366. From IP: 192.168.1.100",
      "tenant_id": "test-org"
    }' > /dev/null
  end=$(date +%s%N)
  echo $(( (end - start) / 1000000 ))
done | sort -n | awk '
  {arr[NR]=$1; sum+=$1}
  END {
    print "P50: " arr[int(NR*0.50)] " ms"
    print "P95: " arr[int(NR*0.95)] " ms"
    print "P99: " arr[int(NR*0.99)] " ms"
  }
'
SCRIPT

# Run benchmark
chmod +x "Technical Project Plan/PM Phases/Phase-2/bench_guard.sh"
./Technical Project Plan/PM Phases/Phase-2/bench_guard.sh
```

---

## 📝 Update Template

After executing tests, update state JSON:

```json
{
  "current_task_id": "D4",
  "last_step_completed": "D3 EXECUTION COMPLETE: Smoke tests executed. Results: X/12 PASS. Performance: P50=___ms, P95=___ms, P99=___ms. [All targets met | Note: Some targets missed]. No PII in logs verified. Determinism verified. Ready for D4.",
  "checklist": {
    "D3": "done"
  },
  "performance_results": {
    "p50_ms": <actual_value>,
    "p95_ms": <actual_value>,
    "p99_ms": <actual_value>,
    "test_date": "2025-11-03"
  },
  "pending_questions": []
}
```

---

## 🔍 Quick Troubleshooting

### Services Won't Start
```bash
docker compose logs privacy-guard --tail 50
docker compose logs vault --tail 20
# Check for PSEUDO_SALT, config loading errors
```

### No Detections
```bash
curl http://localhost:8089/status | jq .rule_count
# Should be > 0, if 0 then rules.yaml not loading
```

### Slow Performance
- Run benchmark 2-3 times (first run may be slow - cold start)
- Check system resources: `docker stats privacy-guard`
- Document actual values even if off target

### Need JWT for Test 8
- See `docs/tests/smoke-phase1.2.md` for JWT retrieval
- Or SKIP Test 8 and document in results

---

## 📚 Reference Files

**Must Read:**
- `Technical Project Plan/PM Phases/Phase-2/D3-EXECUTION-HANDOFF.md` - Complete guide
- `docs/tests/smoke-phase2.md` - Test procedures

**Supporting Docs:**
- `docs/guides/privacy-guard-integration.md` - API reference
- `docs/guides/privacy-guard-config.md` - Configuration
- `Technical Project Plan/PM Phases/Phase-2/DEVIATIONS-LOG.md` - Known issues

---

## ⏱️ Timeline

**Remaining Phase 2 Work:**
- D3 Execution: 1-2 hours (next session)
- D4 Project Docs: 1-2 hours
- ADR Finalization: 30 minutes
- Completion Summary: 1 hour
- PR Preparation: 30 minutes

**Total:** 4-6 hours to Phase 2 completion

---

**Created:** 2025-11-03 21:30  
**Ready for:** Next session execution  
**All changes:** Committed and pushed to GitHub ✅
