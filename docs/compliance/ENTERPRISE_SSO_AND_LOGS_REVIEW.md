# goose Enterprise SSO and Logging Review

Date: 2025-10-29
Author: Research Subagent (goose)

## Executive Summary
- Enterprise SSO/onboarding: goose OSS ships with local-first auth by default and provider OAuth helpers for MCP/LLM integrations. The server (goosed) authenticates via X-Secret-Key in v1.12, not OIDC/JWT. Block’s "download and auto-redirect to SSO" flow is achieved outside the open-source core by pairing enterprise-managed configuration and an identity gateway/OAuth bootstrap that redirects users into a corporate IdP on first-run, then provisions provider/model access centrally (Databricks AI Gateway in their example) and enforces extension allowlists.
- Out-of-the-box logging: goose provides local session records and system logs by default, plus structured tracing and optional OTLP/Langfuse exporters in code. Official docs emphasize local logs only (privacy-first). Your plan/ADRs add an audit event schema and ndjson export atop goose’s baseline—this is aligned and not duplicative of core.
- Alignment: Your master plan and ADRs (OIDC bridge, allowlists, audit schema, metadata-only storage) match goose 1.12’s architecture and documented guides. Differences are expected (JWT gateway, directory/policy, audit service) because OSS goose does not include enterprise IdP integration or central audit pipelines.

## Findings: Enterprise SSO and Onboarding

1) What OSS goose provides (v1.12)
- Server auth: goose-server ("goosed") uses an X-Secret-Key header for API auth. The technical report notes: "Auth: X-Secret-Key header checked by middleware" (goose-versions-references/how-goose-works-docs/...technical-architecture-report.md, lines ~159–161).
- OAuth helpers: The core includes a lightweight OAuth PKCE flow used to authenticate HTTP MCP servers or provider setups:
  - crates/goose/src/oauth/mod.rs spins a localhost callback server and opens a browser to complete OAuth; persists credentials in keyring/secrets (uses rmcp AuthorizationManager). This is for provider/tool auth, not user SSO into goosed.
- Config bootstrap: Desktop UI reads environment/config to build API URLs (ui/desktop/src/config.ts) and exposes many config keys in settings (ui/desktop/src/utils/configUtils.ts). Official docs cover config.yaml and environment variables for deployment-wide defaults.
- Extension allowlist: Admins can enforce a corporate extension list via GOOSE_ALLOWLIST URL; documented in Guides → Extension Allowlist. This is relevant to enterprise setup governance.

2) What Block described for enterprise onboarding (YouTube)
- Segment 16:45–20:33 (approx. 1005s–1233s). Key quote:
  - "...we get no setup for our employees. So if you download goose as a Block employee and you just click a button, you immediately get redirected to this like automatic login flow and you do your single sign on thing, and you have access now to that whole family of models." (around 1136–1176s)
- They centralize model access and telemetry using Databricks serving endpoints/AI Gateway, connecting ~25 models across providers. Admins manage credentials; employees don’t handle API keys. Telemetry and cost tracking happen at the gateway, not inside OSS goose.
- Interpretation: The "auto-redirect to enterprise SSO" appears to be implemented through managed configuration plus a first-run flow that launches the corporate SSO in a browser, likely with:
  - A corporate distribution profile or managed config (env vars/config.yaml) that points the app at enterprise endpoints (e.g., IdP discovery URL, gateway URL).
  - A gateway that handles OIDC with the company IdP and returns usable tokens/credentials to goose via provider integrations or MCP servers. This keeps OSS goose unmodified while giving "zero-setup" for employees.

3) How to replicate this in CE/Self-hosted
- Identity/JWT gateway: Your ADR 0006 proposes an "Identity/Auth Bridge (OIDC → JWT → goosed)". That mirrors Block’s pattern of keeping goosed unchanged (X-Secret-Key) while your gateway handles OIDC Code Flow + PKCE, mints RS256 JWTs, and proxies to goosed. Post-MVP you can embed JWT verification into goosed.
- First-run SSO bootstrap: Use the existing oauth_flow helper pattern for provider/MCP auth UX, but invoke the corporate IdP via the identity gateway and then write short-lived tokens to keyring. Ship the desktop app with preconfigured config.yaml/env (see Guides → Configuration File, Environment Variables) to point at your gateway and allowed extensions.
- Admin governance: Use GOOSE_ALLOWLIST to restrict extensions, and config.yaml to set provider defaults. Pair with profile bundles/policy (ADR 0011) for role-based startup.

4) Is there an out-of-the-box enterprise SSO in goose?
- Not in v1.12. The OSS server does not ship OIDC SSO. The enterprise "auto-redirect" Block mentions relies on external identity and serving gateways with managed config, not on a built-in goosed OIDC feature. Your gateway plan is the right complement.

## Findings: Logging and Observability

1) Official goose docs (Logs guide)
- Local-only by default: "All conversations and interactions (both CLI and Desktop) are stored locally" with paths:
  - Command history: ~/.config/goose/history.txt
  - Session records (JSONL): ~/.local/share/goose/sessions/
  - System logs: ~/.local/state/goose/logs/
  - Server logs (goosed): ~/.local/state/goose/logs/server/
  - CLI logs: ~/.local/state/goose/logs/cli/
  Source: https://block.github.io/goose/docs/guides/logs
- Privacy: Emphasizes logs are not sent externally by default.

2) Code-level observability hooks
- tracing + OTLP: Core tracing has an OTLP exporter (crates/goose/src/tracing/otlp_layer.rs) and layers for observation and Langfuse (observation_layer.rs, langfuse_layer.rs). The architecture report states: "Tracing via tracing crate; OTLP exporter configurable via env or config (OTEL_EXPORTER_OTLP_*). Optional Langfuse integration." (observability section)
- Local log dir helper: crates/goose/src/logging.rs creates component-specific log directories under the state dir.

3) Your plan vs goose defaults
- Master plan Phase 5: "OTLP export config; audit event schema; ndjson export." This adds a domain-specific audit layer beyond goose’s local logs/traces. ADR 0008 defines an AuditEvent schema with redaction maps and ndjson export. ADR 0005/0012 set metadata-only storage and retention.
- This is additive and aligned: goose gives local records and tracing hooks; you are adding centralized audit ingestion/export with redaction safeguards. No duplication—your audit service complements goose’s baseline.

## Comparison to Local Plans & ADRs

- ADR 0004: Identity and Auth (OIDC SSO in MVP)
  - Alignment: Matches enterprise need. goose OSS lacks OIDC SSO for goosed; your OIDC-first stance is compatible via the gateway.
- ADR 0006: Identity/Auth Bridge (OIDC → JWT → goosed)
  - Alignment: Mirrors Block’s enterprise pattern of externalizing SSO and leaving OSS core intact; plan to embed JWT later is sensible.
- ADR 0011: Signed Profile Bundles and Policy Evaluate API
  - Alignment: Provides secure bootstrap and ABAC-lite policy—complements goose’s extension allowlist and permission model.
- ADR 0005 and ADR 0012: Data retention, metadata-only storage
  - Alignment: Matches goose’s local-first/session-records and privacy posture; your retention TTLs and ndjson audit export fill a gap not covered by OSS docs.
- Master plan WBS: Identity (Phase 1), Audit/Observability (Phase 5) align with goose’s capabilities (OTLP, local logs) and documented guides (logs, config, allowlist, env vars).

Divergences (expected and appropriate):
- JWT/SSO at server boundary (your gateway) vs. goose’s X-Secret-Key. This is necessary for enterprise SSO.
- Central audit ingest/ndjson export vs. goose local-only logs. Also necessary for enterprise auditability.

## Recommendations

Quick wins
- Ship a managed config for desktop: Provide a pre-bundled config.yaml and env exports that set:
  - GOOSE_ALLOWLIST to a corporate URL
  - Default providers/models pointing to your gateway or enterprise AI gateway
  - Security prompts and router/toolshim settings per policy
- Implement first-run SSO bootstrap: On first app run, deep-link or auto-open the identity gateway’s OIDC login (browser) and write tokens to keyring. Reuse goose oauth_flow UX patterns for consistency.
- Turn on OTLP optionally: If you have an OTLP collector, set OTEL_EXPORTER_OTLP_* to capture traces; keep content out of traces per ADRs.

Planned deeper changes
- Build the identity/auth gateway from ADR 0006 and bridge to goosed with X-Secret-Key in MVP; add JWT verification in goosed post-MVP if contributing upstream is feasible.
- Implement audit ingest + ndjson export per ADR 0008, with redaction validation and retention jobs as in ADR 0005/0012.
- Package signed profile bundles (ADR 0011) and require them at agent startup to avoid misconfiguration; pair with allowlist enforcement.

Open questions to validate
- Employee "auto-redirect" mechanics: choose between a custom app link/deeplink, desktop post-install first-run page, or environment onboarding script to trigger the gateway.
- IdP details: exact OIDC client config (redirect URIs, scopes, PKCE), realm templates for Keycloak CE, and token lifetimes aligned with ADR 0004.
- Telemetry boundary: if using a model gateway (e.g., Databricks, LiteLLM, Bedrock), confirm which metrics are sourced there vs. from goose’s OTLP/Langfuse layers to prevent duplicated data.

## Sources & Citations
- Local architecture analysis (v1.12):
  - goose-versions-references/how-goose-works-docs/docs/goose-v1.12.00-technical-architecture-report.md — notes X-Secret-Key server auth; OTLP/Langfuse; OAuth helper and permission model; config paths; sessions.
- Code references (goose v1.12 mirror):
  - crates/goose/src/oauth/mod.rs — PKCE OAuth flow via localhost callback and web browser; persisted credentials.
  - crates/goose/src/logging.rs — helper for local log directories.
  - crates/goose/src/tracing/{otlp_layer.rs, observation_layer.rs, langfuse_layer.rs} — observability layers.
  - ui/desktop/src/{config.ts, utils/configUtils.ts, types/config.ts} — UI API/config helpers and labels.
- Official docs:
  - Guides → Logging System: https://block.github.io/goose/docs/guides/logs
  - Guides → Configuration File: https://block.github.io/goose/docs/guides/config-file
  - Guides → Extension Allowlist: https://block.github.io/goose/docs/guides/allowlist
  - Guides → Environment Variables: https://block.github.io/goose/docs/guides/environment-variables
- YouTube presentation (Meet goose, an Open Source AI Agent): https://www.youtube.com/watch?v=fYhBbo900HA
  - Enterprise onboarding quote (approx. 18:56–19:36): "...if you download goose as a Block employee and you just click a button, you immediately get redirected to this like automatic login flow and you do your single sign on thing, and you have access now to that whole family of models." (Transcript segment 1136–1176s)
- Local plans & ADRs:
  - Technical Project Plan (master): Technical Project Plan/master-technical-project-plan.md (Phases on Identity, Audit/Observability)
  - ADR 0004: Identity and Auth (OIDC SSO in MVP)
  - ADR 0006: Identity/Auth Bridge (OIDC → JWT → goosed)
  - ADR 0005: Data Retention and Storage
  - ADR 0008: Audit Schema and Redaction Maps
  - ADR 0011: Signed Profile Bundles and Policy Evaluate API
  - ADR 0012: Storage and Metadata Model

---

Notes: If upstream goose later ships first-class OIDC for goosed, reevaluate ADR 0006 to potentially drop the gateway and standardize on Bearer JWT. Until then, the gateway approach prevents double work while preserving compatibility with OSS goose.
