# Version Pins (Phase 0)

Pin external service versions to reduce drift. Update deliberately via PR.

- Keycloak: 26.0.0 (example; adjust if needed)
- Vault OSS: 1.17.6 (example)
- Postgres: 16.4
- Ollama: 0.3.x (>= 0.3.0)
- MinIO (opt-in): RELEASE.2025-01-25T00-00-00Z (AGPLv3) — optional
- Garage (opt-in): latest stable (AGPLv3) — documented alternative only
- SeaweedFS (opt-in default for ALv2 option): 3.68 (Apache-2.0)

Guard models (documented; not bundled):
- Default: llama3.2:1b (instruct), quant Q4_K_M suggested
- Quality mode: llama3.2:3b (instruct), quant Q4_K_M/Q5_K_M
- Fallback tiny: tinyllama:1.1b
- Optional: phi3:3.8b; qwen2.5:~1.5b instruct

Note: Model weights are not distributed with this repo. First-run pulls are user-approved and logged.
