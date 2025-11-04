# ADR 0015: Guard Model Policy and Selection

- Status: Accepted (MVP)
- Date: 2025-10-31
- Updated: 2025-11-04 (Phase 2.2 - qwen3:0.6b selected as default)
- Authors: @owner

## Context
ADR-0002 fixates guard placement (agent-side pre/post) but did not set model choice, whether native tool-calling is required, or how to handle resource constraints on CE hardware (~8 GB RAM).

## Decision
- User-selectable local model with a sensible default and alternatives. Model weights are not bundled; first-run pull is explicit and consented.
- **Default (Phase 2.2+):** Alibaba Qwen3 0.6B Instruct (CPU-friendly, 523MB, 40K context, Nov 2024).
- **Post-MVP Alternatives (User-Selectable):**
  - Google Gemma3 1B (gemma3:1b, 600MB, 8K context, Dec 2024) - Alternative small model
  - Microsoft Phi-4 3.8B Mini (phi4:3.8b-mini, 2.3GB, 16K context, Dec 2024) - Best accuracy (requires more RAM)
  - For more resources: Meta Llama 3.2 3B, Qwen3 4B, Gemma3 4B
- Native tool/function-calling in the model is OPTIONAL. Goose orchestrates redaction via MCP tools; the guard pipeline does not rely on model-native tools.
- On model change, run a PII redaction smoke test; warn and fallback to default if failing.
- Record model identifier (name:tag:quant) in audit metadata for guard runs.

## Phase 2.2 Update (2025-11-04)

**Model Selection Change:** qwen3:0.6b → Default (replacing llama3.2:1b)

**Rationale:**
- **Size:** 523MB vs ~1GB (llama3.2:1b) - better fit for 8GB RAM systems
- **Recency:** Nov 2024 vs Oct 2023 - more current training data
- **Context:** 40K tokens vs 8K tokens - handles longer documents
- **Performance:** Optimized for CPU execution on edge devices
- **Benchmarks:** 26.5 MMLU, 51.2 IF Eval (comparable NER capability)

**Hardware Target:**
- AMD Ryzen 7 PRO 3700U (4 cores, 8 threads)
- 8GB RAM (~1.7GB available for model)
- CPU-only execution (no GPU)

**Deployment:**
- Isolated Docker Ollama instance (aligns with production MVP)
- Model disabled by default (opt-in via GUARD_MODEL_ENABLED=true)
- Graceful fallback to regex-only if unavailable

## Consequences
- VERSION_PINS and guides list supported options; developers can tailor to hardware.
- A lightweight evaluation harness will be added in Phase 2 to validate guard quality on change.

## Alignment
- ADR-0002: Guard placement.
- ADR-0009: Deterministic pseudonymization keys.
- Privacy-by-design posture.

## References
- Guides: docs/guides/guard-model-selection.md
- VERSION_PINS.md
