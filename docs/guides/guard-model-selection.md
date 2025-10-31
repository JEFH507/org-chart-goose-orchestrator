# Guard Model Selection (CE/MVP)
> See also: ADR 0015 (docs/adr/0015-guard-model-policy-and-selection.md)


Decision summary
- Users can choose their local guard LLM. We provide a sensible default and a compatibility list. Model weights are NOT bundled.
- Default: Meta Llama 3.2 1B Instruct (CPU‑friendly). First‑run pull with explicit consent.
- Quality mode: Meta Llama 3.2 3B Instruct (better accuracy, higher CPU/RAM).
- Fallback tiny: TinyLlama 1.1B (very small; pair with conservative regex/rules).
- Optional: Microsoft Phi‑3 Mini (3.8B) and Qwen2.5 (~1.5B) instruct variants.
- Tool/function calling in the model is OPTIONAL: Goose orchestrates redaction via MCP tools; models without native tools still work.

Rationale
- ADR‑0002 requires agent‑side privacy guard. Small CPU models suffice for classification/redaction recommendations when combined with deterministic regex/rules. 
- Goose v1.12 handles tools via MCP; native tool calling is not required for the guard pipeline.

Hardware guidance
- Floor: CPU‑only on ~8 GB RAM
  - Default (1B) recommended for broad compatibility.
  - 3B is a “quality mode” on machines with more headroom.

Pinned suggestions (not auto‑pulled)
- Default: `llama3.2:1b` (instruct), quant Q4_K_M suggested.
- Quality: `llama3.2:3b` (instruct), quant Q4_K_M/Q5_K_M.
- Tiny fallback: `tinyllama:1.1b`.
- Optional: `phi3:3.8b`, `qwen2.5:<~1.5b instruct>`.

Operational notes
- We never embed model weights in images or this repo. Provide a first‑run “pull with consent” step.
- Record model identifier (name:tag:quant) in audit metadata when guard runs.
- On model change, run a quick PII redaction smoke test. If it fails, warn and fall back to the default.

References
- Llama 3.2 search: https://ollama.com/search?q=llama3.2
- TinyLlama: https://ollama.com/library/tinyllama
- Phi‑3: https://ollama.com/library/phi3
