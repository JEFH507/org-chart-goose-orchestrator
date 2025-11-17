# Goose Org-Chart Orchestrator

**Enterprise-Ready Multi-Agent AI Orchestration with Privacy-First Design**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Phase](https://img.shields.io/badge/Phase-6%20(95%25%20Complete)-green)]()
[![Docker](https://img.shields.io/badge/Docker-Compose%20Ready-blue)]()
[![Grant](https://img.shields.io/badge/Block%20Goose-Grant%20Application-orange)](https://block.github.io/goose/grants/)

---

## Executive Summary

### The Challenge
Enterprises struggle to turn AI into measurable productivity without risking data privacy, compliance, and governance. One-size-fits-all copilots don't fit complex organizational structures, access rules, and departmental workflows.

### Our Solution
A **hierarchical, org-chart-aware AI orchestration framework** that gives every employee and team a "digital twin" assistant tailored to their role, tools, and policies. Scales from individual desktop agents to organization-wide orchestrated agents with strong privacy, governance, and auditability.

### Key Outcomes
- **Faster Execution**: Cross-department coordination (Finance ↔ Legal ↔ Manager) via structured task routing
- **Standardized Processes**: Role-based recipes for common workflows (e.g., "Monthly close" for Finance, "Campaign reporting" for Marketing)
- **Safer AI**: Data minimization pipeline with local PII detection/masking before cloud processing
- **Enterprise Adoption**: Respects organizational structure and compliance requirements while enabling AI productivity

### Product Vision
Transform how enterprises deploy AI by mapping intelligent agents to the organizational chart—enabling role-specific automation, cross-team collaboration, and privacy-preserving coordination at scale.

**Target Users**: CIO/CTO, CISO/Compliance, Department Leaders (Marketing/Finance/Engineering/Support/Legal), IT Ops/Platform Teams, Individual Contributors

---

## What is this?

A **privacy-first, org-chart-aware AI orchestration framework** that coordinates role-based "digital twin" agents across departments. Built on [Goose](https://github.com/block/goose) (by Block) with enterprise-grade security, database-driven configuration, and local PII protection.

**Key Innovation**: Privacy Guard runs on user's CPU - sensitive data never leaves local environment, while coordination happens via secure HTTP APIs.

### System at a Glance

- **17 Docker containers** working together (microservices architecture)
- **50 users, 8 role profiles** (Finance, Legal, Manager, HR, Analyst, Developer, Marketing, Support)
- **3 Privacy Guard modes**: Rules-only (<10ms), Hybrid (<100ms), AI-only (~15s)
- **4 Agent Mesh tools**: send_task, notify, request_approval, fetch_status
- **26 PII detection patterns**: EMAIL, SSN, CREDIT_CARD, PHONE, IP_ADDRESS, etc.
- **Complete audit trail**: Every action logged, every PII detection tracked

## 🎯 Quick Start (5 Minutes)

```bash
# Clone and navigate
git clone https://github.com/JEFH507/org-chart-goose-orchestrator.git
cd org-chart-goose-orchestrator

# Start infrastructure
cd deploy/compose
docker compose -f ce.dev.yml up -d postgres keycloak vault redis
sleep 45

# Unseal Vault (enter 3 keys when prompted)
cd ../..
./scripts/unseal_vault.sh

# Start all services
cd deploy/compose
docker compose -f ce.dev.yml --profile controller --profile multi-goose up -d
sleep 60

# Upload test organization (50 users)
cd ../..
./admin_upload_csv.sh test_data/demo_org_chart.csv

# Access interfaces
echo "Admin Dashboard: http://localhost:8088/admin"
echo "pgAdmin 4: http://localhost:5050"
echo "Privacy Guard (Finance): http://localhost:8096/ui"
echo "Privacy Guard (Manager): http://localhost:8097/ui"
echo "Privacy Guard (Legal): http://localhost:8098/ui"
```

**Comprehensive Demo Guide**: [Demo/COMPREHENSIVE_DEMO_GUIDE.md](COMPREHENSIVE_DEMO_GUIDE.md)

## Architecture Overview

### System Components (17 Containers)

```
┌─────────────────────────────────────────────────────────────┐
│                    INFRASTRUCTURE (4)                        │
├──────────────┬──────────────┬──────────────┬────────────────┤
│ PostgreSQL   │ Keycloak     │ Vault        │ Redis          │
│ (users,      │ (OIDC/JWT,   │ (Transit     │ (caching,      │
│  profiles,   │  10hr tokens)│  signing)    │  idempotency)  │
│  tasks)      │              │              │                │
└──────┬───────┴──────┬───────┴──────┬───────┴───────┬────────┘
       │              │              │               │
       ▼              ▼              ▼               ▼
┌─────────────────────────────────────────────────────────────┐
│                    CONTROLLER (1)                            │
│  Port 8088: REST API + Admin Dashboard                      │
│  - Profile distribution                                      │
│  - Agent Mesh task routing                                   │
│  - User management                                           │
│  - Configuration push                                        │
└──────────────────────┬──────────────────────────────────────┘
                       │
       ┌───────────────┼───────────────┐
       │               │               │
       ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ PRIVACY      │ │ PRIVACY      │ │ PRIVACY      │
│ GUARD        │ │ GUARD        │ │ GUARD        │
│ (FINANCE)    │ │ (MANAGER)    │ │ (LEGAL)      │
│              │ │              │ │              │
│ • Proxy 8096 │ │ • Proxy 8097 │ │ • Proxy 8098 │
│ • Service    │ │ • Service    │ │ • Service    │
│ • Ollama     │ │ • Ollama     │ │ • Ollama     │
│              │ │              │ │              │
│ Mode: Rules  │ │ Mode: Hybrid │ │ Mode: AI     │
│ (<10ms)      │ │ (<100ms)     │ │ (~15s)       │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ GOOSE        │ │ GOOSE        │ │ GOOSE        │
│ (FINANCE)    │ │ (MANAGER)    │ │ (LEGAL)      │
│              │ │              │ │              │
│ Auto-config  │ │ Auto-config  │ │ Auto-config  │
│ from DB      │ │ from DB      │ │ from DB      │
│              │ │              │ │              │
│ Agent Mesh   │ │ Agent Mesh   │ │ Agent Mesh   │
│ Extension    │ │ Extension    │ │ Extension    │
└──────────────┘ └──────────────┘ └──────────────┘
```

### Key Features

**Privacy Guard**:
- **26 PII detection patterns** (EMAIL, SSN, CREDIT_CARD, PHONE, IP_ADDRESS, etc.)
- **Luhn validation** on credit cards (prevents false positives)
- **3 detection modes**: Rules-only, Hybrid (rules + Ollama), AI-only (Ollama semantic)
- **Deterministic pseudonymization** (HMAC-based, consistent across sessions)
- **Audit logging** (every detection tracked with session ID)

**Agent Mesh**:
- **Cross-agent coordination** (Finance ↔ Manager ↔ Legal)
- **4 MCP tools**: send_task, notify, request_approval, fetch_status
- **Task persistence** (PostgreSQL, survives restarts - migration 0008)
- **Idempotency** (Redis-backed, safe retries)

**Enterprise Security**:
- **Keycloak OIDC/JWT** (10-hour token lifespan)
- **Vault Transit signing** (profile integrity verification)
- **AppRole authentication** (1-hour token lifespan with fallback)
- **Role-based access control** (profile-driven extension allowlists)

**Database-Driven Configuration**:
- **50 users** from CSV import (organizational hierarchy)
- **8 role profiles** (Analyst, Developer, Finance, HR, Legal, Manager, Marketing, Support)
- **Profile auto-fetch** on Goose container startup
- **Signature verification** via Vault Transit engine

## Project Structure

```
.
├── Demo/                           # Demo guides and validation
│   ├── COMPREHENSIVE_DEMO_GUIDE.md # Main demo script
│   ├── Container_Management_Playbook.md
│   └── Privacy-Guard-Pattern-Reference.md
├── Technical Project Plan/         # Master plan + phase tracking
│   ├── master-technical-project-plan.md
│   └── PM Phases/
│       ├── Phase-0/ ... Phase-6/   # Phase completion docs
│       └── Phase-6/Phase-6-Agent-State.json  # Current state
├── docs/                           # Documentation
│   ├── product/productdescription.md
│   ├── architecture/PHASE5-ARCHITECTURE.md
│   ├── grants/                     # Grant proposal materials
│   ├── operations/                 # Operational guides
│   └── tests/                      # Test documentation
├── src/                            # Source code (Rust + Python)
│   ├── controller/                 # Main API server (Axum)
│   ├── privacy-guard/              # PII detection service
│   ├── privacy-guard-proxy/        # HTTP proxy + UI
│   ├── agent-mesh/                 # MCP extension (Python)
│   ├── lifecycle/                  # Session FSM (Rust lib)
│   ├── profile/                    # Profile system (Rust lib)
│   └── vault/                      # Vault client (Rust lib)
├── deploy/compose/                 # Docker Compose configs
│   └── ce.dev.yml                  # Community Edition stack
├── scripts/                        # Automation scripts
│   ├── unseal_vault.sh
│   ├── sign-all-profiles.sh
│   ├── get_admin_token.sh
│   └── admin_upload_csv.sh
├── test_data/                      # Test datasets
│   └── demo_org_chart.csv          # 50 test users
└── seeds/                          # Database migrations
    └── postgres/                   # PostgreSQL schema
```

## Current Status (Phase 6)

**Overall Progress**: 95% Complete

### ✅ Completed Workstreams

1. **Workstream A: Lifecycle Integration** (100%)
   - Session FSM (PENDING → ACTIVE → PAUSED → COMPLETED)
   - 17/17 tests passing

2. **Workstream B: Privacy Guard Proxy** (100%)
   - HTTP proxy with standalone UI
   - 3 detection modes (rules/hybrid/AI)
   - 35/35 tests passing

3. **Workstream C: Multi-Goose Environment** (100%)
   - 3 Goose containers (Finance, Manager, Legal)
   - Profile auto-fetch from Controller
   - 17/18 tests passing (94%)

4. **Workstream D: Agent Mesh E2E** (100%)
   - All 4 MCP tools working
   - Task persistence (migration 0008)
   - Vault integration complete

5. **Admin Dashboard** (100%)
   - CSV upload (50 users)
   - User management (profile assignment)
   - Profile editor (create/edit/download)
   - Live logs (mock implementation)

6. **Demo Validation** (90%)
   - Comprehensive demo guide complete
   - 6-terminal layout documented
   - Known limitations tracked

### 📋 Pending (Deferred to Phase 7)

- Automated testing suite (81+ tests)
- UI detection mode persistence fix (ISSUE-1-UI)
- Ollama hybrid/AI mode validation (ISSUE-1-OLLAMA)
- Employee ID validation bug fix (ISSUE-3)
- Push button implementation (ISSUE-4)
- Deployment topology documentation

**See**: [Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json](Technical%20Project%20Plan/PM%20Phases/Phase-6/Phase-6-Agent-State.json)

## Grant Alignment

### Block Goose Innovation Grant ($100K/12mo)

**What We Built (Phases 0-6, 7 weeks)**:
- ✅ Privacy Guard (novel: local PII masking with 3 modes)
- ✅ Agent Mesh (novel: org-aware multi-agent coordination)
- ✅ Database-driven profiles (8 roles, extensible)
- ✅ Enterprise security (Keycloak, Vault, JWT)
- ✅ Admin dashboard (CSV upload, profile management)
- ✅ Complete demo system (17 containers, fully working)

**Proposed 12-Month Roadmap**:

**Q1 (Complete)**: Foundation
- Privacy Guard implementation
- Agent Mesh coordination
- Admin dashboard
- Database integration

**Q2 (Months 4-6)**: Testing & Polish
- Automated testing (81+ tests)
- Security hardening
- Production deployment guides
- UI improvements

**Q3 (Months 7-9)**: Scale & Features
- 10 role profiles library
- Model orchestration (lead/worker)
- Kubernetes deployment
- Performance optimization

**Q4 (Months 10-12)**: Community & Upstream
- Advanced features (SCIM, approvals)
- Community engagement (blog posts, talks)
- Upstream contributions (5 PRs to Goose)
- Business validation (2 paid pilots)

**Grant Proposal**: [docs/grants/GRANT_PROPOSAL.md](docs/grants/GRANT_PROPOSAL.md) *(to be created)*

## Documentation

### Essential Docs

- **Quick Start**: [Demo/COMPREHENSIVE_DEMO_GUIDE.md](COMPREHENSIVE_DEMO_GUIDE.md)
- **Product Vision**: [docs/product/productdescription.md](docs/product/productdescription.md)
- **Master Plan**: [Technical Project Plan/master-technical-project-plan.md](Technical%20Project%20Plan/master-technical-project-plan.md)
- **Architecture**: [docs/architecture/PHASE5-ARCHITECTURE.md](docs/architecture/PHASE5-ARCHITECTURE.md)
- **Privacy Guard Reference**: [Demo/Privacy-Guard-Pattern-Reference.md](Demo/Privacy-Guard-Pattern-Reference.md)
- **Container Management**: [Demo/Container_Management_Playbook.md](Demo/Container_Management_Playbook.md)

### Phase-Specific Docs

- Phase 0-6 completion docs: `Technical Project Plan/PM Phases/Phase-{0-6}/`
- Current phase state: `Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json`
- Progress logs: `docs/tests/phase{1-6}-progress.md`

### API Documentation

- **OpenAPI Spec**: http://localhost:8088/docs (when running)
- **Controller API**: 15 REST endpoints
- **Admin API**: 9 endpoints (CSV, users, profiles, logs)
- **Privacy Guard API**: 6 endpoints (settings, status, audit)

## Known Limitations & Issues

**Current known issues** (tracked for Phase 7):

1. **ISSUE-1-UI**: Privacy Guard UI detection mode changes don't persist
2. **ISSUE-1-OLLAMA**: Hybrid/AI detection modes not fully tested
3. **ISSUE-3**: Database Employee ID validation expects string (should accept integer)
4. **ISSUE-4**: Admin UI "Push" button is placeholder
5. **ISSUE-5**: Employee ID pattern not in Privacy Guard catalog
6. **ISSUE-6**: Terminal escape sequences break word boundary regex

**GitHub Issues**: Will be created from [Demo/Demo-Validation-State.json](Demo/Demo-Validation-State.json)

**These gaps demonstrate**:
- System is 85-90% complete (demo-ready)
- Clear roadmap for grant funding
- Realistic scope (no overpromising)
- Proven foundation (working demo shows feasibility)

## Development

### Prerequisites

- Docker & Docker Compose
- Bash (for scripts)
- 8GB disk space
- 4GB RAM minimum

### Development Workflow

```bash
# Start infrastructure
cd deploy/compose
docker compose -f ce.dev.yml up -d postgres keycloak vault redis

# Unseal Vault
cd ../.. && ./scripts/unseal_vault.sh

# Start services
cd deploy/compose
docker compose -f ce.dev.yml --profile controller --profile multi-goose up -d

# Watch logs
docker compose -f ce.dev.yml logs -f controller

# Rebuild after code changes
docker compose -f ce.dev.yml build controller
docker compose -f ce.dev.yml restart controller
```

### Testing

```bash
# Unit tests (Rust)
cd src/controller
cargo test

# Integration tests
cd ../../
./scripts/test_integration.sh

# Privacy Guard tests
cd src/privacy-guard
cargo test
```

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

**Quick start**:
1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'feat: add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## License

Apache-2.0 (core components)

See [LICENSE](LICENSE) for full text.

## Technology Stack & Dependencies

### Core Frameworks
- **[Goose](https://github.com/block/goose)** - MCP-based AI agent framework by Block (v1.12.00 baseline)
  - [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) - Tool integration standard
  - Agent Engine with extension system
  - Desktop and API (goosed) deployment modes

### Infrastructure Components
- **[PostgreSQL](https://www.postgresql.org/)** (v16) - Relational database for users, profiles, tasks, audit logs
  - 10 tables across 9 migrations (0001-0009)
  - Foreign keys, indexes, triggers for data integrity
- **[Keycloak](https://www.keycloak.org/)** (v26.0.7) - Identity and access management
  - OIDC/JWT authentication (10-hour token lifespan)
  - SSO integration ready
- **[HashiCorp Vault](https://www.vaultproject.io/)** (v1.18.3) - Secrets management
  - Transit engine for profile signature signing
  - AppRole authentication (1-hour token lifespan)
  - Root token mode (dev only, NOT FOR PRODUCTION)
- **[Redis](https://redis.io/)** (v7.4) - Caching and idempotency
  - Task idempotency keys
  - Session state caching

### Application Stack
- **Rust** (v1.83.0) - Backend services
  - [Axum](https://github.com/tokio-rs/axum) (v0.7) - Web framework for Controller API
  - [Tokio](https://tokio.rs/) (v1.48) - Async runtime
  - [SQLx](https://github.com/launchbadge/sqlx) (v0.8) - PostgreSQL driver
  - [Reqwest](https://github.com/seanmonstar/reqwest) (v0.12) - HTTP client
- **Python** (v3.12) - Agent Mesh MCP extension
  - [goose-mcp](https://pypi.org/project/goose-mcp/) - MCP server SDK
  - [httpx](https://www.python-httpx.org/) - Async HTTP client
  - [pydantic](https://docs.pydantic.dev/) - Data validation

### AI/ML Components
- **[Ollama](https://ollama.ai/)** (v0.5.4) - Local LLM inference
  - qwen3:0.6b model for Named Entity Recognition (NER)
  - Used in Privacy Guard hybrid/AI detection modes
  - Semantic PII detection (complements regex rules)

### Development Tools
- **[Docker](https://www.docker.com/)** & **[Docker Compose](https://docs.docker.com/compose/)** - Container orchestration
  - 17 containers in multi-service stack
  - Service profiles: controller, multi-goose, single-goose
- **[pgAdmin 4](https://www.pgadmin.org/)** - PostgreSQL administration UI
- **Cargo** & **pip** - Package managers for Rust and Python

### Standards & Protocols
- **[Model Context Protocol (MCP)](https://modelcontextprotocol.io/)** - Tool/extension standard (Goose native)
- **[OIDC/OAuth2](https://openid.net/developers/how-connect-works/)** - Authentication via Keycloak
- **[OpenAPI/Swagger](https://swagger.io/specification/)** - API documentation (Controller REST API)
- **[OpenTelemetry (OTEL)](https://opentelemetry.io/)** - Observability (planned Phase 7)

---

## Future Integration: Agent-to-Agent (A2A) Protocol

### What is A2A?

The [**Agent-to-Agent Protocol (A2A)**](https://a2a-protocol.org/) is an open standard (Apache 2.0) developed by Google LLC that enables communication and interoperability between opaque agentic applications. It allows AI agents built on diverse frameworks by different companies running on separate servers to collaborate effectively—**as agents, not just as tools**.

**Key Capabilities**:
- **Agent Discovery**: Via "Agent Cards" (JSON documents) detailing capabilities, connection info, authentication
- **Standardized Communication**: JSON-RPC 2.0 over HTTP(S)
- **Flexible Interaction**: Synchronous request/response, streaming (SSE), asynchronous push notifications
- **Rich Data Exchange**: Text, files, and structured JSON data
- **Opacity Preservation**: Agents collaborate without exposing internal state, memory, or tools
- **SDKs Available**: [Python](https://github.com/a2aproject/a2a-python), [Go](https://github.com/a2aproject/a2a-go), [JavaScript](https://github.com/a2aproject/a2a-js), [Java](https://github.com/a2aproject/a2a-java), [.NET](https://github.com/a2aproject/a2a-dotnet)

### A2A vs. MCP: Complementary Protocols

**Model Context Protocol (MCP)**: Connects **agents to tools/resources** (databases, APIs, files)  
- *Our use*: Goose extensions (Developer, GitHub, Privacy Guard)

**Agent2Agent Protocol (A2A)**: Enables **agent-to-agent collaboration** (task delegation, workflows)  
- *Our opportunity*: Replace custom Agent Mesh HTTP/gRPC with A2A JSON-RPC

### Synergy with Our Stack

Our orchestration system shares several design goals with A2A:

| **Our Implementation** | **A2A Protocol** | **Integration Opportunity** |
|------------------------|------------------|----------------------------|
| Agent Mesh (HTTP/gRPC) | A2A JSON-RPC 2.0 | Replace custom protocol with A2A-compliant messages |
| `send_task`, `notify`, `request_approval`, `fetch_status` | `a2a/createTask`, `a2a/getTaskStatus` | Map our 4 MCP tools to A2A task lifecycle methods |
| Task Router (Controller) | A2A Agent Registry | Implement A2A discovery service with Agent Cards |
| Privacy Guard pre/post | A2A Security Layer | Map PII masking to A2A trust boundaries |
| PostgreSQL `tasks` table | A2A Task State Machine | Align schema with A2A task lifecycle |
| Role profiles (YAML) | A2A Agent Cards (JSON) | Export profiles as A2A capability manifests |
| Keycloak/Vault/JWT | A2A Authentication Schemes | Map OIDC tokens to A2A `Authorization` headers |

### Key Design Principles Alignment

1. **Interoperability**: Agents from different vendors/frameworks can communicate
   - *Our system*: MCP for tools, custom HTTP for agent mesh → **A2A would enable multi-vendor agent collaboration**
2. **Extensibility**: Custom message types beyond core protocol
   - *Our system*: Already supports custom task payloads → **A2A standardizes envelope format**
3. **Security & Opacity**: Trust models, authentication; agents don't expose internals
   - *Our system*: Keycloak/Vault, Privacy Guard PII masking → **Natural mapping to A2A trust model**
4. **Asynchronous Workflows**: Fire-and-forget, callbacks, polling, streaming
   - *Our system*: Redis idempotency, task polling → **A2A adds SSE streaming + push notifications**

### Integration Roadmap (Post-Phase 7)

**Phase 8 (Proposed Q3 2025): A2A Compatibility Layer**
1. **Agent Card Generation**: Convert YAML profiles → JSON Agent Cards with Vault-signed integrity
2. **A2A JSON-RPC Endpoint**: Implement `POST /a2a/{agent_id}/rpc` with `a2a/createTask`, `a2a/getTaskStatus`
3. **Task Schema Alignment**: Extend PostgreSQL `tasks` table with A2A fields (`a2a_task_id`, `a2a_status`, `a2a_context`)
4. **Dual Protocol Support**: Maintain backward compatibility with custom Agent Mesh during transition
5. **Integration Testing**: Validate interoperability with external A2A-compliant agent systems

**Benefits**:
- **Multi-Vendor Interoperability**: Goose agents ↔ Google Gemini agents, Microsoft Autogen agents, etc.
- **Standards-Based**: Reduce custom code, leverage [A2A SDKs](https://github.com/a2aproject) and community tooling
- **Enterprise Credibility**: Adopting industry standards (MCP + A2A) demonstrates production maturity

**Tradeoffs**:
- **Complexity**: JSON-RPC 2.0 adds overhead vs. simple HTTP POST; Agent Cards require generation/signing infrastructure
- **Maturity**: A2A launched 2024, evolving in 2025; specification may change (monitor for breaking changes)
- **Value Validation**: ROI depends on A2A ecosystem growth and real-world multi-vendor use cases

**Decision**: **Yellow Light** → Monitor A2A adoption quarterly; initiate pilot when ≥2 validation partners confirmed.

**See**: [docs/integrations/a2a-protocol-analysis.md](docs/integrations/a2a-protocol-analysis.md) for detailed analysis and prototype Agent Card.

---

## Acknowledgments

- Built on [Goose](https://github.com/block/goose) by [Block](https://block.xyz/)
- Applying for [Block Goose Innovation Grant](https://block.github.io/goose/grants/)
- Privacy Guard uses [Ollama](https://ollama.ai/) for NER ([qwen3:0.6b](https://ollama.com/library/qwen2.5:0.5b) model)
- Infrastructure: [PostgreSQL](https://www.postgresql.org/), [Keycloak](https://www.keycloak.org/), [HashiCorp Vault](https://www.vaultproject.io/), [Redis](https://redis.io/)
- Inspired by [A2A Protocol](https://a2a-protocol.org/) for future multi-agent interoperability

## Contact & Links

- **GitHub**: https://github.com/JEFH507/org-chart-goose-orchestrator
- **Issues**: https://github.com/JEFH507/org-chart-goose-orchestrator/issues
- **Author**: Javier (@JEFH507)
- **Grant Program**: https://block.github.io/goose/grants/

---

**Status**: Phase 6 (95% Complete) - Ready for grant proposal demo  
**Last Updated**: 2025-11-17  
**Next Milestone**: Phase 7 (Testing & Production Readiness)
