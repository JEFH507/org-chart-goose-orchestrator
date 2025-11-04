# Phase 2.2 Model Selection Decision Log

**Date:** 2025-11-04  
**Phase:** Phase 2.2 - Privacy Guard Enhancement  
**Decision:** qwen3:0.6b selected as default NER model (replacing llama3.2:1b)

---

## Context

Phase 2.2 adds local NER model support via Ollama to improve PII detection accuracy. The original planning documents (ADR-0015, execution plan) recommended `llama3.2:1b` as the default based on October 2023 state-of-the-art.

During implementation kickoff (2025-11-04), user provided actual hardware specifications and requested model recommendation review.

**Hardware Specifications:**
- **CPU:** AMD Ryzen 7 PRO 3700U (4 cores, 8 threads, 2.3-4.0 GHz)
- **RAM:** 8GB total (~1.7GB available after OS/services)
- **Architecture:** x86_64, CPU-only (no GPU)
- **Use Case:** Development laptop, conservative resource usage

**Requirements:**
- Model must run efficiently on CPU-only
- Memory footprint < 1GB (ideally < 600MB)
- Context window sufficient for typical PII-containing documents
- NER capability for PERSON, ORGANIZATION, EMAIL, PHONE, SSN, etc.
- Recent training data (2024+ preferred)

---

## Options Evaluated

### Option 1: llama3.2:1b (Original Plan)
- **Size:** ~1GB
- **Context:** 8K tokens
- **Release:** October 2023
- **Pros:** Known quantity, well-tested, good NER
- **Cons:** Tight fit for 8GB RAM, smaller context, older training data
- **Benchmarks:** Not specified in docs

### Option 2: qwen3:0.6b (Selected)
- **Size:** 523MB
- **Context:** 40K tokens
- **Release:** November 2024
- **Pros:** 
  - 50% smaller memory footprint than llama3.2:1b
  - 5x larger context window (40K vs 8K)
  - Most recent training data (Nov 2024)
  - Optimized for CPU/edge devices
  - Strong instruction following (51.2 IF Eval)
- **Cons:** Slightly lower benchmarks than larger models (acceptable trade-off)
- **Benchmarks:** 26.5 MMLU, 51.2 IF Eval, 62.3 HellaSwag (0-shot)

### Option 3: gemma3:1b
- **Size:** 815MB
- **Context:** 32K tokens
- **Release:** ~7 months ago
- **Pros:** Larger context than llama3.2:1b, Google backing
- **Cons:** Larger than qwen3:0.6b, less recent than qwen3

### Option 4: qwen3:1.7b
- **Size:** 1.4GB
- **Context:** 40K tokens
- **Release:** November 2024
- **Pros:** Better benchmarks, same context as 0.6b
- **Cons:** 2.7x larger than 0.6b, may cause memory pressure

---

## Decision Matrix

| Criterion | Weight | llama3.2:1b | qwen3:0.6b | gemma3:1b | qwen3:1.7b |
|-----------|--------|-------------|------------|-----------|------------|
| Memory Footprint | 25% | ⭐⭐⭐ (1GB) | ⭐⭐⭐⭐⭐ (523MB) | ⭐⭐⭐⭐ (815MB) | ⭐⭐ (1.4GB) |
| Context Window | 20% | ⭐⭐ (8K) | ⭐⭐⭐⭐⭐ (40K) | ⭐⭐⭐⭐ (32K) | ⭐⭐⭐⭐⭐ (40K) |
| Recency | 15% | ⭐⭐ (Oct 2023) | ⭐⭐⭐⭐⭐ (Nov 2024) | ⭐⭐⭐⭐ (~Apr 2024) | ⭐⭐⭐⭐⭐ (Nov 2024) |
| CPU Efficiency | 15% | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| NER Capability | 15% | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Proven Track Record | 10% | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Weighted Score** | **100%** | **3.45** | **4.45** | **3.95** | **3.40** |

**Winner:** qwen3:0.6b (4.45/5.0)

---

## Rationale

**Primary Factors:**
1. **Memory Efficiency:** 523MB fits comfortably in 1.7GB available RAM, leaves room for OS and other services
2. **Context Window:** 40K tokens handles long documents (emails, reports) without truncation
3. **Recency:** Nov 2024 training includes more current language patterns and PII formats
4. **Hardware Fit:** Explicitly optimized for CPU execution on edge devices (per Ollama docs)

**Secondary Factors:**
5. **Benchmarks:** 26.5 MMLU and 51.2 IF Eval sufficient for NER tasks (not general reasoning)
6. **Deployment:** Smaller model = faster pull, lower storage overhead in Docker
7. **Future-proofing:** Qwen3 series actively maintained, likely to receive updates

**Acceptable Trade-offs:**
- Slightly lower raw benchmarks than 1.7B variant (not material for NER use case)
- Less proven in field than Llama 3.2 (acceptable given clear advantages)

---

## Implementation Details

**Configuration:**
```bash
# Default (Phase 2.2+)
OLLAMA_MODEL=qwen3:0.6b
GUARD_MODEL_ENABLED=false  # Opt-in for backward compatibility
OLLAMA_URL=http://ollama:11434  # Docker internal network
```

**Deployment:**
- Isolated Docker Ollama instance (not shared with Goose Desktop)
- Model pulled on first use (explicit consent)
- Graceful fallback to regex-only if unavailable

**Performance Targets:**
- P50 latency: ≤ 700ms with model (vs 16ms regex-only)
- P95 latency: ≤ 1000ms with model (vs 22ms regex-only)
- Accuracy improvement: +10-20% over regex-only

---

## Alternatives for Different Use Cases

**If user needs higher accuracy:**
- Upgrade to `qwen3:1.7b` or `llama3.2:3b` or `qwen3:4b`
- Config: `OLLAMA_MODEL=qwen3:1.7b`
- Trade-off: Higher latency, more memory usage

**If user has even less RAM (<6GB):**
- Fallback to `tinyllama:1.1b` (637MB)
- Config: `OLLAMA_MODEL=tinyllama:1.1b`
- Trade-off: Lower accuracy, pair with strict regex rules

**If user prioritizes speed:**
- Disable model entirely: `GUARD_MODEL_ENABLED=false`
- Falls back to Phase 2 regex-only (P50=16ms)

---

## References

**Research:**
- Ollama library search: https://ollama.com/library
- Qwen3 documentation: https://ollama.com/library/qwen3
- Gemma3 documentation: https://ollama.com/library/gemma3
- User hardware specifications (provided 2025-11-04)

**Documentation Updated:**
- ADR-0015: Guard Model Policy and Selection
- docs/guides/guard-model-selection.md
- VERSION_PINS.md
- Phase 2.2 planning documents

**Code:**
- `src/privacy-guard/src/ollama_client.rs`: Default model set to qwen3:0.6b

---

## Sign-Off

**Decision:** ✅ Approved - qwen3:0.6b selected as Phase 2.2 default  
**Date:** 2025-11-04  
**Approver:** User (hardware owner)  
**Implementer:** Phase 2.2 Orchestrator AI

**Rationale:** Best fit for target hardware (8GB RAM, CPU-only), optimal balance of size/context/recency

**Next Review:** Phase 2.2 completion - validate actual performance and accuracy improvement
