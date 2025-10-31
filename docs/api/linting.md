# OpenAPI Linting (Phase 0)

We use Spectral to lint OpenAPI locally in Phase 0 (CI warn-only), and enforce in Phase 1.

- Local: `scripts/dev/openapi_lint.sh` (uses `npx @stoplight/spectral-cli` if Node is available; otherwise prints instructions)
- Targets:
  - docs/api/controller/openapi.yaml

CI policy
- Phase 0: warn-only (non-blocking)
- Phase 1: change to blocking

Notes
- Keep TODOs and placeholder schemas; Spectral may warn. That’s expected in Phase 0.
