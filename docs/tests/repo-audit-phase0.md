# Repo Audit (Phase 0, read-only)

## Inventory (counts by top-level):
- .github/: 3 files
- .goose/: 7 files
- Technical Project Plan/: 103 files
- backups/: 14 files
- config/: 1 files
- db/: 2 files
- deploy/: 7 files
- docs/: 85 files
- scripts/: 6 files
- tests/: 1 files

## Markdown link check
- total: 53, broken: 5
  - [BROKEN] backups/20251027-173349/docs/architecture/one-pager.md -> ../README.md
  - [BROKEN] backups/20251027-173349/docs/architecture/one-pager.md -> ./overview.md
  - [BROKEN] docs/README.md -> ../product/productdescription.md
  - [BROKEN] docs/README.md -> ../product/requirements.md
  - [BROKEN] docs/README.md -> ../architecture/technical-requirements.md

## $ref check (YAML/JSON)
- total: 3, broken: 2
  - [BROKEN] docs/api/controller/openapi.yaml -> ../schemas/task.schema.json
  - [BROKEN] docs/api/controller/openapi.yaml -> '
