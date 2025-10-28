# Plan

## Objectives
Implement baseline detection (regex + small LLM), deterministic mapping with per-tenant keys.

## Scope
Text-only MVP; images/files post-MVP.

## WBS
- PII regex/ruleset [Now, S]
- Optional local LLM NER (Ollama small) [Next, M]
- Deterministic mapping (HMAC-SHA256 + format-preserving where needed) [Now, S]
- Provider wrapper hooks [Now, M]
- Redaction logs and audit integration [Now, S]

## Timeline
Weeks 1–3

## Milestones
Mask-and-forward on by default; acceptance tests pass.
