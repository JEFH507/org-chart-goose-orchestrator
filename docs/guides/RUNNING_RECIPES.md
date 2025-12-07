# Recipes Runbook

This guide explains which recipes to run, in what order, and why. It separates local (project-scoped) recipes from global (system-wide) recipes.

## Terminology
- Local recipes (project): `./.goose/recipes/`
- Global recipes (system): `~/.config/goose/recipes/`
- Source of truth docs:
  - Product: `./productdescription.md`
  - Architecture tour: `./docs/architecture/index.html`
  - Orchestrator (minimal): `./docs/architecture/orchestrator_min.html`
  - “How goose works”: `./goose-versions-references/how-goose-works-docs/docs/architecture-map.html`

## Recommended order (from idea → plan → spec → tasks → delivery)
1) Product Owner (local)
   - Recipe: `./.goose/recipes/product-owner.yaml`
   - Output: `./requirements.md`
   - Purpose: capture personas, journeys, constraints, NFRs (company‑agnostic) grounded in `productdescription.md`.

2) Planner (local)
   - Recipe: `./.goose/recipes/planner.yaml`
   - Input: `./requirements.md` (default)
   - Output: `./plan.mmd` (Mermaid mindmap)
   - Purpose: organize goals, features, constraints, risks, assumptions into a one‑screen plan.

3) Technical Requirements (local)
   - Recipe: `./.goose/recipes/technical-requirements.yaml`
   - Inputs: `./productdescription.md`, `./requirements.md` (optional), `./plan.mmd` (optional)
   - Output: `./technical-requirements.md`
   - Purpose: define NFR targets, system decomposition, interfaces, data model, stack options, deployment, observability, risks.

4) ADRs (ad hoc, as decisions arise)
   - Template: `./docs/adr/adr-template.md`
   - Creator Recipe: `./.goose/recipes/adr-create.yaml`
   - Output: `./docs/adr/NNNN-title.md`
   - Purpose: record significant technical decisions (why/how/trade‑offs). Suggested initial ADRs: 0001–0005 already created.

5) Architect (global)
   - Recipe: `~/.config/goose/recipes/architect.yaml`
   - Input: `project_root = <absolute path>`
   - Purpose: scaffold folders/placeholders guided by `technical-requirements.md` and key ADRs.
   - Tip: ensure a git feature branch is used for scaffolding.

6) Project Manager (global)
   - Recipe: `~/.config/goose/recipes/project-manager.yaml`
   - Inputs: `plan_path = ./plan.mmd`, `project_root = <absolute path>`
   - Outputs: `./project-board.mmd`, appended items in `./PROJECT_TODO.md`, JSON task summary
   - Purpose: create actionable tasks and a lightweight Kanban.

7) Business Model Validation (local)
   - Recipe: `./.goose/recipes/business-model-validation.yaml`
   - Inputs: `./productdescription.md`, optional `./plan.mmd`, `./requirements.md`
   - Output: `./business-model.md`
   - Purpose: outline viable open‑core/enterprise/SaaS models, with a comparison and next validation steps.

8) Release Manager (global; milestone boundaries)
   - Recipe: `~/.config/goose/recipes/release.yaml`
   - Inputs: `project_root = <absolute path>`
   - Purpose: draft release notes, propose version bump, prepare CHANGELOG diff, and tag (with explicit approval).

## Tips
- Keep ADRs short and scannable; supersede rather than edit history.
- Prefer minimal, reversible changes in early phases; defer heavy infra choices unless required by NFRs.
- Theme consistency for docs is already wired; keep new pages aligned.

## Quick commands (examples)
- Local recipe (from project root): run via goose Desktop → Recipes → select the YAML.
- Global recipe: same flow, but pick from `~/.config/goose/recipes/` and provide `project_root`.
