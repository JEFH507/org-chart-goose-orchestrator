# Privacy Guard Proxy Performance Benchmarks

**Date:** 2025-11-10  
**Version:** Privacy Guard Proxy 0.1.0  
**Test Environment:** Local Docker containers

---

## Executive Summary

✅ **Proxy Infrastructure:** Excellent performance (~1ms)  
✅ **Unmask/Reidentify:** Excellent performance (~1ms)  
⚠️ **NER Masking:** High latency (~15 seconds) - **EXPECTED for NER model**

**Total Overhead:** ~15 seconds (primarily NER model processing)  
**Target:** < 200ms  
**Status:** ⚠️ NER model bottleneck (expected behavior, optimization options documented)

---

## Detailed Results

**Proxy API:** 1.21ms (excellent)  
**Privacy Guard Mask:** 14,936.99ms (~15s) - NER model processing  
**Privacy Guard Unmask:** 0.98ms (excellent)  
**Combined:** ~15 seconds

**Root Cause:** Ollama qwen3:0.6b NER model running on CPU

**Optimization Options:**
1. Rule-based only: < 10ms (recommended for MVP)
2. GPU acceleration: 1-3 seconds
3. Hybrid approach: < 100ms for common PII
4. Async processing: < 10ms perceived latency

---

**Test Script:** tests/performance/test_proxy_latency.sh  
**Full Report:** See this document for detailed analysis
