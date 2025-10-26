# goose-org-twin
This is the workspace for the Org-Chart Orchestrated AI Framework.

## Project Template
- Put project docs in ./docs
- Recipes live in ./.goose/recipes
- Use feature branches, conventional commits, and PRs

## Quick links
- Product description: ./productdescription.md
- Project TODO: ./PROJECT_TODO.md
- Docs (GitHub-native): ./docs/README.md
- Architecture: ./docs/architecture/ (Markdown)
- Product pages: ./docs/product/
- Pitch: ./docs/pitch/
- Guides: ./docs/guides/ (TBD)
- API docs: ./docs/api/ (TBD)

## Structure
- .git
- .goose
- config/: Configuration files and templates
- docs/
  - architecture/: Diagrams and architecture notes
  - api/: API documentation (to be added in the technical phase)
  - guides/: User and admin guides (to be added)
- goose-versions-references/ (upstream Goose references + our analysis docs)
- scripts/: Automation scripts (setup, deploy, backup)  
- src/: Source code (to be added in implementation phases)
- tests/: Test suites (to be added)
- .env.example
- .gitignore
- .goosehints
- .gooseignore
- CHANGELOG.md: Version history
- productdescription.md
- PROJECT_TODO.md
- README.md
- THOUGHTS.md


# Notes
- Keep large artifacts and secrets out of Git; see .gitignore and .gooseignore
- Project recipes live under ./.goose/recipes

## Grant alignment
- License: Apache-2.0 (core)
- Goose Grant Program alignment: docs/grants/GRANT_PROPOSAL_DRAFT.md
- Community Edition (CE) focus: self-hostable, OSS defaults (Keycloak, Vault OSS, Postgres, MinIO, Ollama)
- Paid pilot validation in MVP: target 1–2 design partners in Q2 milestone
