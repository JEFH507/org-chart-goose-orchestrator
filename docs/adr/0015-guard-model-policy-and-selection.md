# ADR 0015: Guard Model Policy and Selection

- Status: Accepted (MVP)
- Date: 2025-10-31
- Authors: @owner

## Context
ADR-0002 fixates guard placement (agent-side pre/post) but did not set model choice, whether native tool-calling is required, or how to handle resource constraints on CE hardware (~8 GB RAM).

## Decision
- User-selectable local model with a sensible default and alternatives. Model weights are not bundled; first-run pull is explicit and consented.
- Default: Meta Llama 3.2 1B Instruct (CPU-friendly).
- Quality option: Meta Llama 3.2 3B Instruct (better accuracy; more resource use).
- Fallback: TinyLlama 1.1B (very small; pair with conservative regex/rules).
- Optional: Microsoft Phi‑3 Mini (3.8B); Qwen2.5 (~1.5B) instruct variants.
- Native tool/function-calling in the model is OPTIONAL. Goose orchestrates redaction via MCP tools; the guard pipeline does not rely on model-native tools.
- On model change, run a PII redaction smoke test; warn and fallback to default if failing.
- Record model identifier (name:tag:quant) in audit metadata for guard runs.

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
