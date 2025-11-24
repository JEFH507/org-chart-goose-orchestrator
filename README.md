# Goose Org-Chart Orchestrator

**Open Source Enterprise-Ready Multi-Agent AI Orchestration with Privacy-First Design**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Phase](https://img.shields.io/badge/Phase-6%20(95%25%20Complete)-green)]()
[![Docker](https://img.shields.io/badge/Docker-Compose%20Ready-blue)]()

---

Who I am(my big disclaimer): 
## Goal Summary
> #skein:
> *a flock of geese, or the like, in V flight formation.*


![[geese_formation.png]]
### The Challenge
Enterprises struggle to turn AI into measurable productivity without risking data privacy, compliance, and governance. Vendor lock-in, choosing one LLM AI provider, and privacy concerns, are just the tip of the iceberg among some the one-size-fits-all  solutions that don't fit complex organizational structures, access rules, and departmental workflows. 

### Why This Matters

The barriers to enterprise AI adoption are real and quantifiable:

- **67% of enterprises** cite data privacy as the #1 barrier to AI adoption
- **GDPR fines** up to €20M for PII leakage create significant liability risk
- **Manual coordination** between departments (email threads, meetings) wastes an estimated 15+ hours per employee per week
- **Vendor lock-in** limits flexibility and increases costs through per-seat pricing and token limits
- **One-size-fits-all copilots** don't respect organizational hierarchies, access rules, or departmental workflows
- **Lack of audit trails** blocks compliance (SOC2, HIPAA, PCI-DSS) and creates governance gaps

Traditional LLM providers offer zero data sovereignty, forcing enterprises to choose between AI productivity and regulatory compliance.

### The Solution
Open frameworks. No vendor lock in. User Empowerment. Free to deploy on your own.

Leverage modern open source frameworks to deliver a **hierarchical, org-chart-aware AI orchestration framework** that gives every employee and team access to pre-configured goose instances, accessible by enterprise log in and tailored to their role and policies. Scales from individual desktop agents to organization-wide orchestrated agents with strong privacy, governance, and auditability.

### Key Outcomes
- **Faster Execution**: Cross-department coordination (e.g., Finance ↔ Legal ↔ Manager) via structured task routing and strong cross agent authentication
- **Standardized, Repeatable, Shareable Processes**: Role-based recipes, MCP extensions, .goosehints and .gooseignore files, for common workflows (e.g., "Monthly close" for Finance, "Campaign reporting" for Marketing)
- **Safer AI**: Data minimization pipeline with local PII detection/masking before reaching large cloud LLM processing/providers trough rules and pre -trained local AI Models.
- **Enterprise Adoption**: Respects organizational structure and compliance requirements while enabling AI productivity

### Product Vision
Transform how enterprises deploy AI by mapping intelligent agents to the organizational chart—enabling role-specific automation and configuration, cross-team collaboration, and privacy-preserving coordination at scale. Allowing easy scalability with minimal vendor lock-in trough open source technology.

**Target Users**: CIO/CTO, CISO/Compliance, Department Leaders (e.g., Marketing/Finance/Engineering/Support/Legal), IT Ops/Platform Teams, Individual Contributors

---

## How are we building it?

A **privacy-first, org-chart-aware AI orchestration framework** that coordinates role-based agents across departments. Built on [Goose](https://github.com/block/goose) (by Block) with enterprise-grade security, database-driven configuration, and local PII protection.

**Key Innovation**: Privacy Guard runs on user's CPU - sensitive data never leaves local environment, while coordination happens via secure HTTP APIs.

**Privacy Guard Data Flow:**
```
User Input: "My SSN is 123-45-6789"
    ↓
Privacy Guard Proxy (HTTP interceptor)
    ↓
Privacy Guard Service (PII detection/masking)
    ↓ [Optional] Ollama (NER model: qwen3:0.6b)
    ↓
Masked Text: "My SSN is [SSN1]"
    ↓
LLM API (cloud LLM sees only masked)
    ↓
Response: "I see you provided [SSN1]"
    ↓
Privacy Guard Service (unmask response)
    ↓
User sees: "I see you provided 123-45-6789"
```

### Current Testing System at a Glance

- **17 Docker containers** working together (microservices architecture)
- **50 users, 8 role profiles** (Finance, Legal, Manager, HR, Analyst, Developer, Marketing, Support)
- **3 Privacy Guard modes**: Rules-only (<10ms), Hybrid (<100ms), AI-only (~15s* model is not optimized yet*)
- **4 Agent Mesh MCP tools**: send_task, notify, request_approval, fetch_status (A2A framework can likely replace this)
- **26 PII detection patterns**: EMAIL, SSN, CREDIT_CARD, PHONE, IP_ADDRESS, etc.
- **Complete audit trail**: Every action logged on your own infrastructure, every PII detection tracked

## 🎯 Quick Start (Docker Environment)

**Follow the steps here**: [[Container_Management_Playbook]]

**Comprehensive Demo Guide**: [Demo/COMPREHENSIVE_DEMO_GUIDE.md](COMPREHENSIVE_DEMO_GUIDE.md)

## Architecture Overview

```
┌───────────────────────────────────────────────────────────────────────────┐
│                         ADMIN INTERFACE                                   │
│                  http://localhost:8088/admin                              │
│                                                                           │
│  ┌──────────────┬──────────────┬──────────────┬─────────────────────────┐ │
│  │ CSV Upload   │ User Mgmt    │ Profile Edit │ Config Push  │ Live Logs│ │
│  │ (50 users)   │ (Assign)     │ (8 profiles) │ (3 instances)│(Auto-ref)│ │
│  └──────────────┴──────────────┴──────────────┴─────────────────────────┘ │
└─────────────────────────────┬─────────────────────────────────────────────┘
                              │ JWT Auth (10-hour tokens)
                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         KEYCLOAK (IAM)                                      │
│                  http://localhost:8080                                      │
│  Realm: dev  │  Client: goose-controller  │  Grant: client_credentials      │
└─────────────────────────────┬───────────────────────────────────────────────┘
                              │ JWT Tokens
                              ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                      CONTROLLER SERVICE                                    │
│                  http://localhost:8088                                     │
│                                                                            │
│  ┌────────────────────┬──────────────────┬──────────────────────────────┐  │
│  │ Profile Manager    │ Agent Mesh Router│ Session Manager              │  │
│  │ (DB-driven config) │ (/tasks/route)   │ (FSM lifecycle)              │  │
│  └────────────────────┴──────────────────┴──────────────────────────────┘  │
└──┬──────────┬──────────┬──────────┬──────────────────────────────────────┬─┘
   │          │          │          │                                      │
   │ Vault    │ Redis    │ Postgres │ Privacy Guard Proxies                │
   ▼          ▼          ▼          ▼                                      │
┌──────┐ ┌────────┐ ┌──────────┐ ┌─────────────────────────────────────┐   │
│Vault │ │ Redis  │ │PostgreSQL│ │    Privacy Guard Proxy (3 instances)│   │
│:8200 │ │ :6379  │ │ :5432    │ │                                     │   │
│:8201 │ │        │ │orchestr. │ │ Finance │ Manager │ Legal           │   │
│      │ │        │ │50 users  │ │ :8096   │ :8097   │ :8098           │   │
│Unseal│ │LRU-256M│ │8 profiles│ │ (Rules) │ (Hybrid)│ (AI-only)       │   │
│3-of-5│ │        │ │Migration │ │         │         │                 │   │
│      │ │        │ │0001-0009 │ │         │         │                 │   │
└──────┘ └────────┘ └──────────┘ └───────┬─────────┬─────────┬─────────┘   │
   │                      │              │         │         │             │
   │Profile Signatures    │Org Users     │         │         │             │
   │Transit HMAC          │Tasks Table   │         │         │             │
   │AppRole Auth          │Sessions Table│         │         │             │
   │                      │              │         │         │             │
   └──────────────────────┴──────────────┴─────────┴─────────┴─────────────┘
                          │              │        │         │
                          ▼              ▼        ▼        �▼
┌────────────────────────────────────────────────────────────────────────────┐
│              PRIVACY GUARD SERVICES (3 instances)                          │
│                                                                            │
│  ┌──────────────────────┬──────────────────────┬──────────────────────┐    │
│  │ Finance Service      │ Manager Service      │ Legal Service        │    │
│  │ :8093                │ :8094                │ :8095                │    │
│  │ GUARD_MODEL_ENABLED= │ GUARD_MODEL_ENABLED= │ GUARD_MODEL_ENABLED= │    │
│  │ false (rules-only)   │ true (hybrid)        │ true (AI-only)       │    │
│  │ <10ms latency        │ <100ms typical       │ ~15s latency         │    │
│  └──────────────────────┴──────────────────────┴──────────────────────┘    │
└──────────┬───────────────┬───────────────┬─────────────────────────────────┘
           │               │               │
           ▼               ▼               ▼
┌──────────────────────┬──────────────────────┬──────────────────────┐
│ Ollama Finance       │ Ollama Manager       │ Ollama Legal         │
│ :11435               │ :11436               │ :11437               │
│ qwen3:0.6b NER       │ qwen3:0.6b NER       │ qwen3:0.6b NER       │
│ Volume: ollama_fin.  │ Volume: ollama_mgr.  │ Volume: ollama_leg.  │
│ Isolated CPU queue   │ Isolated CPU queue   │ Isolated CPU queue   │
└──────────────────────┴──────────────────────┴──────────────────────┘
           │               │               │
           └───────────────┴───────────────┴────> No blocking between instances!
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                     GOOSE TESTINg INSTANCES (3 containers)                 │
│                                                                            │
│  ┌──────────────────────┬──────────────────────┬──────────────────────┐    │
│  │ Finance (ce_goose_   │ Manager (ce_goose_   │ Legal (ce_goose_     │    │
│  │ finance)             │ manager)             │ legal)               │    │
│  │ Image: goose-test:   │ Image: goose-test:   │ Image: goose-test:   │    │
│  │ 0.5.3                │ 0.5.3                │ 0.5.3                │    │
│  │                      │                      │                      │    │
│  │ Profile: finance     │ Profile: manager     │ Profile: legal       │    │
│  │ (from DB at startup) │ (from DB at startup) │ (from DB at startup) │    │
│  │                      │                      │                      │    │
│  │ Agent Mesh: ✅       │ Agent Mesh: ✅       │ Agent Mesh: ✅       │    │
│  │ 4 tools available    │ 4 tools available    │ 4 tools available    │    │
│  │                      │                      │                      │    │
│  │ Workspace: isolated  │ Workspace: isolated  │ Workspace: isolated  │    │
│  └──────────────────────┴──────────────────────┴──────────────────────┘    │
└────────────────────────────────────────────────────────────────────────────┘
```

### System Components (17 Containers)

```
┌─────────────────────────────────────────────────────────────┐
│                    INFRASTRUCTURE (4)                       │
├──────────────┬──────────────┬──────────────┬────────────────┤
│ PostgreSQL   │ Keycloak     │ Vault        │ Redis          │
│ (users,      │ (OIDC/JWT,   │ (Transit     │ (caching,      │
│  profiles,   │  10hr tokens)│  signing)    │  idempotency)  │
│  tasks)      │              │              │                │
└──────┬───────┴──────┬───────┴──────┬───────┴───────┬────────┘
       │              │              │               │
       ▼              ▼              ▼               ▼
┌─────────────────────────────────────────────────────────────┐
│                    CONTROLLER (1)                           │
│  Port 8088: REST API + Admin Dashboard UI                   │
│  - Profile distribution                                     │
│  - Agent Mesh task routing                                  │
│  - User management                                          │
│  - Configuration push                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
       ┌───────────────┼───────────────┐
       │               │               │
       ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ PRIVACY      │ │ PRIVACY      │ │ PRIVACY      │
│ GUARD(3)     │ │ GUARD (3)    │ │ GUARD(3)     │
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
│ GOOSE 1      │ │ GOOSE 2      │ │ GOOSE  3     │
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
- **50 testing users** from CSV import (organizational hierarchy)
- **8 testing role profiles** (Analyst, Developer, Finance, HR, Legal, Manager, Marketing, Support)
- **Profile auto-fetch** on Goose container startup
- **Signature verification** via Vault Transit engine

## Important Project Structure
This repository still needs much clean up work, but here is a basic guide to the documentation.

```
.
├── Demo/                           # Demo guides and validation
│   ├── COMPREHENSIVE_DEMO_GUIDE.md # Main demo script
│   ├── Container_Management_Playbook.md
│   └── System_Analysis_Report.md
|
├── Technical Project Plan/         # Master plan + phase tracking
│   ├── master-technical-project-plan.md
│   └── PM Phases/
│       ├── Phase-0/ ... Phase-6/   # Phase completion docs
│       └── Phase-6/Phase-6-Agent-State.json  # Current state
├── docs/                          # WIP Documentation
│   ├── product/productdescription.md
│   ├── architecture/PHASE5-ARCHITECTURE.md
│   ├── grants/                     # Possible grant proposal materials
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
├── docker/goose/                   # Docker File & Script for Multi-Goose test  
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

## Project Progress & Phases

**Status**: Phase 6 (95% Complete) - Ready for concept demo  
**Last Updated**: 2025-11-17  
**Next Milestone**: Phase 7-8 (UI, Testing, Hardening, & Production Readiness)

### ✅ Current Status (Phase 6)

For completed and future phase: [[master-technical-project-plan]]

**What We Built (Phases 0-6 weeks, concept validation)**:
- ✅ Privacy Guard Service (local PII masking with 3 modes)
- ✅ Privacy Guard Proxy (Routes user prompts before  they reach LLM Provider)
- ✅ Agent Mesh (org-aware multi-agent coordination)
- ✅ Database-driven profiles (8 roles, extensible)
- ✅ Enterprise security (Keycloak, Vault, JWT)
- ✅ Admin dashboard (CSV upload, profile management)
- ✅ Complete demo system (17 containers, fully working)

**12-Month Roadmap**:

**Q1 (Current)**: Proof Concept & Architecture Foundation
- Privacy Guard Service (PR to Upstream Goose, or Add-on module)
- Proxy-Goose (Routing Messages to local Privacy Guard before it reaches Cloud LLM)
- Orchestrator-Controller (API orchestrator between services and modules)
- Agent Mesh coordination (Multi agent and role collaboration)
- Admin dashboard (Easy UI for IT teams and end user)
- Goose-Containers Testing Environment (Designed to test the E2E infrastructure)
- Database integration (PostgreSQL)

**Q2 (Months 4-6)**: Testing & Polish
- Testing E2E
	- SSO sign-in
	- Enterprise security (Keycloak, Vault, JWT)
	- Database sync accross infrastcture
	- Controller orchestration (Goose config fecth by end user at sign, and data base updates)
	- UI
- Security hardening
- A2A framework vs Agentmesh 
- Production deployment guides
- UI improvements

**Q3 (Months 7-9)**: Scale & Features
- 10-20 Open source roles profiles library
- Community edition fully operational
- Kubernetes deployment
- Performance optimization
- Test with some industries a potential commercial application as an Open Source SAAS Model, where we provide the Database and Controller as a service.

**Q4 (Months 10-12)**: Community & Upstream
- Advanced features (based on feedback)
- Community engagement (blog posts, talks)
- **Upstream contributions (5 PRs to Goose core)**:
  1. Privacy Guard API/MCP/or UI Extension (standalone PII detection/masking)
  2. OIDC/JWT Middleware (enterprise authentication)
  3. Agent Mesh Protocol Spec (multi-agent communication standard)
  4. Session Persistence Module (PostgreSQL backend for sessions)
  5. Role Profiles Spec & Validator (JSON schema + 8 reference templates)
- Business validation (2 paid pilots)

## Success Criteria (12-Month Targets)

### Technical Metrics
- **Test Coverage**: >90% on critical paths
- **API Performance**: P50 latency <5s (current: <0.5s ✅)
- **Privacy Guard Latency**: 
  - Rules-only: <10ms ✅
  - Hybrid: <100ms ✅
  - AI-only: <5s (current: ~15s, target: optimization pending)
- **System Availability**: >99.5% uptime
- **Open Critical Bugs**: <10 at any time

### Community Metrics
- **GitHub Stars**: 200+ (demonstrates interest)
- **Discord/Community**: 50+ active members
- **Production Deployments**: 20+ organizations
- **Contributors**: 15+ (beyond core maintainer)

### Impact Metrics
- **Upstream PRs**: 5 merged to Goose core
- **Conference Presence**: 2+ accepted talks
- **Blog Posts**: 5+ published (technical deep-dives)
- **Business Validation**: 2 paid pilots ($50K+ ARR validated)

## Sustainability Model

### Open Source Core (Apache 2.0)
**Always Free, Self-Hosted:**
- Privacy Guard (all 3 detection modes)
- Agent Mesh (cross-agent coordination)
- Controller API (orchestration)
- Profile system (role-based configuration)
- Community Edition: All features unlocked

**Philosophy**: Core components remain open source forever. Privacy Guard always runs locally (trust model).

### Future Commercial Offerings (Optional)

**Business Edition** (Managed SaaS - Estimated $x/user/month):
- Managed Controller + Admin UI (cloud-hosted)
- Privacy Guard stays local (data sovereignty preserved)
- Enterprise support (SLA, dedicated Slack)
- Advanced features (SCIM provisioning, SSO, multi-tenancy)
- Compliance reports (SOC2, HIPAA, PCI-DSS)
- **Target**: 10 customers by Month 18

### Path to Sustainability
- **Year 1 (Months 1-12)**: Grant-funded R&D, community building, upstream contributions, pilot program launch, Business Edition MVP, break-even target
- **Year 2 (Months 13-24)**:  Enterprise tier, profitability ($600K ARR target)

**Note**: Open source core is irrevocable (Apache 2.0). Commercial offerings are value-adds, not gatekeepers.

## Documentation

### Essential Docs

- **Quick Start**: [Demo/COMPREHENSIVE_DEMO_GUIDE.md](COMPREHENSIVE_DEMO_GUIDE.md)
- **Product Vision**: [docs/product/productdescription.md](docs/product/productdescription.md)
- **Master Plan**: [Technical Project Plan/master-technical-project-plan.md](Technical%20Project%20Plan/master-technical-project-plan.md)
- **Architecture**: [docs/architecture/PHASE5-ARCHITECTURE.md](docs/architecture/PHASE5-ARCHITECTURE.md)
- **Privacy Guard Reference**: [Demo/Privacy-Guard-Pattern-Reference.md](Demo/Privacy-Guard-Pattern-Reference.md)
- **Container Management**: [Demo/Container_Management_Playbook.md](Demo/Container_Management_Playbook.md)
- **Grant Proposal**: [docs/grants/GRANT_PROPOSAL.md](docs/grants/GRANT_PROPOSAL.md)

### Phase-Specific Docs

- Phase 0-6 completion docs: `Technical Project Plan/PM Phases/Phase-{0-6}/`
- Current phase state: `Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json`
- Progress logs: `Technical Project Plan/PM Phases/Phase-{0-6}/phase{1-6}-progress.md`

### API Documentation

- **OpenAPI Spec**: http://localhost:8088/docs (when running)
- **Controller API**: 15 REST endpoints
- **Admin API**: 9 endpoints (CSV, users, profiles, logs)
- **Privacy Guard API**: 6 endpoints (settings, status, audit)

## Known Limitations (Pre-Production)

**Privacy Guard**:
- UI settings don't persist across restarts (manual re-configuration required) - [Issue #32](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/32)
- Hybrid/AI detection modes need validation (accuracy benchmarking pending) - [Issue #33](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/33)
- Employee ID pattern refinement needed - [Issue #36](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/36)

**Agent Mesh**:
- Employee ID validation bug (false positives on certain formats) - [Issue #34](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/34)
- Push configuration button not fully implemented - [Issue #35](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/35)

**Security**:
- Terminal escape sequences not sanitized (potential injection risk) - [Issue #37](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/37)
- Vault running in dev mode with root token (NOT FOR PRODUCTION)

**Documentation**:
- Repository cleanup needed (WIP docs, orphaned files)
- Integration test coverage incomplete (81+ tests planned, 51 passing)

**See all issues**: https://github.com/JEFH507/org-chart-goose-orchestrator/issues

**System Maturity**:
- ✅ 85-90% complete (demo-ready, concept validated)
- ✅ Realistic scope (no overpromising, gaps documented)
- ✅ Proven foundation (working demo, 17 containers operational)
- ⚠️ Pre-production (needs testing, security hardening, documentation)

## Development

### Prerequisites

- Docker & Docker Compose
- Bash (for scripts)
- 8GB disk space
- 4GB RAM minimum

### Development Workflow

[[Container_Management_Playbook]]

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

See [LICENSE for full text](https://www.apache.org/licenses/LICENSE-2.0).

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
  -  [pgAdmin 4](https://www.pgadmin.org/) - PostgreSQL administration UI
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
- **Cargo** & **pip** - Package managers for Rust and Python

### Standards & Protocols
- **[Model Context Protocol (MCP)](https://modelcontextprotocol.io/)** - Tool/extension standard (Goose native)
- **[OIDC/OAuth2](https://openid.net/developers/how-connect-works/)** - Authentication via Keycloak
- **[OpenAPI/Swagger](https://swagger.io/specification/)** - API documentation (Controller REST API)
- **[OpenTelemetry (OTEL)](https://opentelemetry.io/)** - Observability (planned Phase 7)

---

## Future Possible Integration: Agent-to-Agent (A2A) Protocol

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

**Phase X (Proposed Q2 2025): A2A Compatibility Layer**
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
