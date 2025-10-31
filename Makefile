# Minimal Makefile delegating to scripts (Phase 0)

.PHONY: preflight up down lint-openapi

preflight:
	./scripts/dev/preflight_ports.sh

up:
	echo "Use docker compose -f deploy/compose/ce.dev.yml up -d (to be added in Phase 1)."

down:
	echo "Use docker compose -f deploy/compose/ce.dev.yml down (to be added in Phase 1)."

lint-openapi:
	./scripts/dev/openapi_lint.sh docs/api/controller/openapi.yaml
