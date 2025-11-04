# Guard Model Selection (CE/MVP)
> See also: ADR 0015 (docs/adr/0015-guard-model-policy-and-selection.md)
> **Updated:** 2025-11-04 (Phase 2.2) - Default changed to qwen3:0.6b

## Decision summary
- Users can choose their local guard LLM. We provide a sensible default and a compatibility list. Model weights are NOT bundled.
- **Default (Phase 2.2+):** Alibaba Qwen3 0.6B Instruct (CPU‑friendly, 523MB, 40K context). First‑run pull with explicit consent.
- Quality mode: Meta Llama 3.2 3B Instruct (better accuracy, higher CPU/RAM).
- Alternative 1B models: Meta Llama 3.2 1B, Qwen3 1.7B, Gemma3 1B.
- Fallback tiny: TinyLlama 1.1B (very small; pair with conservative regex/rules).
- Optional: Microsoft Phi‑3 Mini (3.8B), Gemma3 4B.
- Tool/function calling in the model is OPTIONAL: Goose orchestrates redaction via MCP tools; models without native tools still work.

Rationale
- ADR‑0002 requires agent‑side privacy guard. Small CPU models suffice for classification/redaction recommendations when combined with deterministic regex/rules. 
- Goose v1.12 handles tools via MCP; native tool calling is not required for the guard pipeline.

Hardware guidance
- Floor: CPU‑only on ~8 GB RAM
  - Default (1B) recommended for broad compatibility.
  - 3B is a “quality mode” on machines with more headroom.

Pinned suggestions (not auto‑pulled)
- **Default (Phase 2.2+):** `qwen3:0.6b` (523MB, 40K context, Nov 2024)
- Alternative 1B: `llama3.2:1b` (instruct), `qwen3:1.7b`, `gemma3:1b`
- Quality: `llama3.2:3b` (instruct), `qwen3:4b`, `gemma3:4b`
- Tiny fallback: `tinyllama:1.1b`
- Optional: `phi3:3.8b`

**Phase 2.2 Model Selection (2025-11-04):**
- Selected: `qwen3:0.6b` for default
- Reason: Smaller (523MB vs 1GB), more recent (Nov 2024), larger context (40K vs 8K)
- Hardware: Optimized for 8GB RAM systems, CPU-only execution
- Benchmarks: 26.5 MMLU, 51.2 IF Eval, 62.3 HellaSwag (0-shot)

Operational notes
- We never embed model weights in images or this repo. Provide a first‑run “pull with consent” step.
- Record model identifier (name:tag:quant) in audit metadata when guard runs.
- On model change, run a quick PII redaction smoke test. If it fails, warn and fall back to the default.

References
- Qwen3 (default): https://ollama.com/library/qwen3
- Llama 3.2: https://ollama.com/search?q=llama3.2
- Gemma3: https://ollama.com/library/gemma3
- TinyLlama: https://ollama.com/library/tinyllama
- Phi‑3: https://ollama.com/library/phi3
