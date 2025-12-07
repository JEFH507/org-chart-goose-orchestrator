# goose Org-Chart Orchestrator

**Open Source Enterprise-Ready Multi-Agent AI Orchestration with Privacy-First Design**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Phase](https://img.shields.io/badge/Phase-6%20(95%25%20Complete)-green)]()
[![Docker](https://img.shields.io/badge/Docker-Compose%20Ready-blue)]()

---

## About This Project

**Who I Am:**  
I'm Javier, a solo industrial engineer (not a trained software developer) building this as my first serious open-source project. I'm leveraging my understanding of systems and AI tools like goose to explore how enterprise AI orchestration could work. Expect rough edges, documented gaps, and honest transparency about what works and what doesn't. If you find bugs or have ideas, please share them—I'm learning in public.

**📖 Want a Deep Dive?**  
For a complete walkthrough with screenshots, architecture explanations, and demo analysis, read the full blog post: **[Building Enterprise-Ready AI Orchestration: Org-Chart-Aware Agents with Privacy-First Design](docs/blog/enterprise-ai-orchestration-privacy-first.md)**

## Goal Summary

> **#skein:** *a flock of geese in V flight formation*  
>  
> Each goose has a role (navigator, followers), they coordinate mid-flight, and the flock moves faster together than alone. That's the vision here—AI agents coordinating like geese in formation, not isolated chatbots working in silos.

![Geese formation - organic AI orchestration metaphor](geese_formation.png)

### The Problem

Enterprises can't deploy AI without risking data leaks. Finance, Legal, HR—each department needs different tools and workflows, but one-size-fits-all AI copilots don't respect organizational structure, access controls, or privacy rules. Meanwhile, employees are copying SSNs and financial data into ChatGPT because there's no sanctioned alternative that actually works for their role. 

### Why This Matters

The barriers to enterprise AI adoption aren't hypothetical—they're what stops most organizations from moving beyond pilot projects:

- **Data privacy concerns dominate executive hesitation** around AI deployment
- **GDPR, HIPAA, SOC2 compliance creates existential liability risks**—one PII leak can trigger multi-million dollar fines
- **Manual coordination between departments** (email threads, meetings for approvals) defeats AI productivity gains
- **Vendor lock-in through proprietary APIs** with unpredictable pricing models
- **One-size-fits-all copilots** don't respect organizational structure—Finance needs different capabilities than Legal, HR needs different workflows than Engineering
- **Lack of audit trails** makes compliance verification impossible

Meanwhile, cloud LLM providers offer zero data sovereignty—your organization's sensitive data leaves your control the moment it reaches their APIs.

### What I Built

An org-chart-aware AI orchestration system where goose instances map to job roles—Finance, Legal, HR, Engineering—each with role-specific tools, policies, and workflows. Privacy Guard intercepts every LLM call and masks PII (SSNs, emails, credit cards) **before** data reaches cloud providers. Built with open standards: self-host it, modify it, or run it on your laptop—your data, your rules.

### What This Enables

- **Cross-department collaboration**: Finance → Manager → Legal task routing with full audit trails
- **Role-specific configuration**: Finance gets spreadsheet tools, Legal gets compliance checks, HR gets recruitment workflows
- **Local PII protection**: Sensitive data never leaves your infrastructure—masking happens on your CPU before cloud LLMs see it
- **Database-driven governance**: Update policies once, all agents fetch new config on next startup

### Who This Is For

IT teams deploying AI across departments without cloud data leaks. If you're a CTO worried about GDPR fines, or a CISO who can't sleep because employees are pasting customer SSNs into ChatGPT—this explores one potential approach.

---

## How It Works

Built on [goose](https://github.com/block/goose) (by Block) with a privacy-first architecture: Privacy Guard runs on users' CPUs, Controller orchestrates via secure HTTP APIs, and PostgreSQL stores all configuration.

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
- **Profile auto-fetch** on goose container startup
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
├── docker/goose/                   # Docker File & Script for Multi-goose test  
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

For completed and future phases: [[master-technical-project-plan]]

**What's Built (Phases 0-6, ~6 weeks of concept validation)**:
- ✅ Privacy Guard Service (local PII masking with 3 modes: rules, hybrid, AI)
- ✅ Privacy Guard Proxy (HTTP interceptor that routes prompts through masking before LLM)
- ✅ Agent Mesh (cross-agent task routing - Finance → Manager → Legal workflows)
- ✅ Database-driven profiles (8 roles: Finance, Legal, HR, Manager, Analyst, Developer, Marketing, Support)
- ✅ Enterprise security foundation (Keycloak, Vault, JWT - dev mode, not production-ready)
- ✅ Admin dashboard (CSV upload, profile management, user assignment)
- ✅ Complete demo environment (17 Docker containers, fully operational)

## What I'm Asking From the goose Community

**Feedback and Collaboration:**

1. **Architecture Review**: Does this approach make sense for enterprise goose orchestration? What am I missing or overcomplicating?

2. **Privacy Guard as Extension?**: Should Privacy Guard be:
   - A standalone MCP extension (packaged separately, users install it)
   - Part of goose core (upstreamed with configuration options)
   - Kept separate as infrastructure (proxy model makes it tool-agnostic)

3. **Agent Mesh vs A2A Protocol**: I built a custom HTTP-based agent coordination system. Should I:
   - Focus on making Agent Mesh work reliably first
   - Pivot to A2A protocol integration (Google's open standard for agent-to-agent communication)
   - Support both with a compatibility layer

4. **Upstreaming Opportunities**: Which components would add value to goose core?
   - OIDC/JWT authentication middleware
   - Session persistence (PostgreSQL backend for goosed sessions)
   - Role-based profile system (JSON schema + validator)
   - Privacy Guard as optional extension

5. **Honest Critique**: What's naive? What won't scale? What's a bad idea I should abandon before investing more time?

I'm not asking you to merge unfinished code—I'm asking for direction before I go further down this path. If this doesn't align with goose's vision, tell me now.

**How to Engage:**
- **GitHub Discussions**: https://github.com/JEFH507/org-chart-goose-orchestrator/discussions
- **Issues**: https://github.com/JEFH507/org-chart-goose-orchestrator/issues (20+ documented gaps)
- **Try It**: Follow the [Container Management Playbook](/Demo/Container_Management_Playbook.md) and break things

**12-Month Roadmap**:

**Q1 (Current)**: Proof Concept & Architecture Foundation
- Privacy Guard Service (PR to Upstream goose, or Add-on module)
- Proxy-goose (Routing Messages to local Privacy Guard before it reaches Cloud LLM)
- Orchestrator-Controller (API orchestrator between services and modules)
- Agent Mesh coordination (Multi agent and role collaboration)
- Admin dashboard (Easy UI for IT teams and end user)
- goose-Containers Testing Environment (Designed to test the E2E infrastructure)
- Database integration (PostgreSQL)

**Q2 (Months 4-6)**: Testing & Polish
- Testing E2E
	- SSO sign-in
	- Enterprise security (Keycloak, Vault, JWT)
	- Database sync accross infrastcture
	- Controller orchestration (goose config fecth by end user at sign, and data base updates)
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
- **Upstream contributions (5 PRs to goose core)**:
  1. Privacy Guard API/MCP/or UI Extension (standalone PII detection/masking)
  2. OIDC/JWT Middleware (enterprise authentication)
  3. Agent Mesh Protocol Spec (multi-agent communication standard)
  4. Session Persistence Module (PostgreSQL backend for sessions)
  5. Role Profiles Spec & Validator (JSON schema + 8 reference templates)
- Business validation (2 paid pilots)

## Open Source First

This project is Apache 2.0 licensed—forever. All components (Privacy Guard, Agent Mesh, Controller, Profile System) are free to self-host, modify, and redistribute. No feature gates, no paid tiers in the core.

**Future exploration:** A managed SaaS version (where I host the Controller for enterprises, while Privacy Guard stays local on their machines) might make sense after proving the open-source version works. No decisions made yet—just exploring sustainability models. See `docs/grants/GRANT_PROPOSAL.md` for the full business/grant thinking.

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

**Agent Mesh (CRITICAL - Phase 7 Priority)**:
- ❌ **agentmesh__notify broken** (validation error) - [Issue #51](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/51) 🔴 **CRITICAL**
  - Impact: 1/4 Agent Mesh tools non-functional (25% failure rate)
  - Evidence: December 5, 2025 demo screenshots (45, 47)
- ⚠️ **fetch_status returns "unknown" status** - [Issue #52](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/52) 🔴 **HIGH**
  - Tool executes but returns incomplete data
  - Cannot filter tasks by role
  - Root Cause Hypothesis: Task ID format mismatch (task: vs session- prefix)
  - Evidence: December 5, 2025 demo screenshots (54, 60)

**Privacy Guard**:
- UI settings don't persist across restarts (manual re-configuration required) - [Issue #32](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/32)
- Hybrid/AI detection modes need validation (accuracy benchmarking pending) - [Issue #33](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/33)
- Employee ID pattern refinement needed - [Issue #36](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/36)
- Employee ID validation bug (false positives on certain formats) - [Issue #34](https://github.com/JEFH507/org-chart-goose-orchestrator/issues/34)

**Admin UI**:
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
- **[goose](https://github.com/block/goose)** - MCP-based AI agent framework by Block (v1.12.00 baseline)
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
- **[Model Context Protocol (MCP)](https://modelcontextprotocol.io/)** - Tool/extension standard (goose native)
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
- *Our use*: goose extensions (Developer, GitHub, Privacy Guard)

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

**Phase X (Proposed Q2 2026): A2A Compatibility Layer**
1. **Agent Card Generation**: Convert YAML profiles → JSON Agent Cards with Vault-signed integrity
2. **A2A JSON-RPC Endpoint**: Implement `POST /a2a/{agent_id}/rpc` with `a2a/createTask`, `a2a/getTaskStatus`
3. **Task Schema Alignment**: Extend PostgreSQL `tasks` table with A2A fields (`a2a_task_id`, `a2a_status`, `a2a_context`)
4. **Dual Protocol Support**: Maintain backward compatibility with custom Agent Mesh during transition
5. **Integration Testing**: Validate interoperability with external A2A-compliant agent systems

**Benefits**:
- **Multi-Vendor Interoperability**: goose agents ↔ Google Gemini agents, Microsoft Autogen agents, etc.
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

- Built on [goose](https://github.com/block/goose) by [Block](https://block.xyz/)
- Privacy Guard uses [Ollama](https://ollama.ai/) for NER ([qwen3:0.6b](https://ollama.com/library/qwen2.5:0.5b) model)
- Infrastructure: [PostgreSQL](https://www.postgresql.org/), [Keycloak](https://www.keycloak.org/), [HashiCorp Vault](https://www.vaultproject.io/), [Redis](https://redis.io/)
- Inspired by [A2A Protocol](https://a2a-protocol.org/) for future multi-agent interoperability

## Contact & Links

- **GitHub**: https://github.com/JEFH507/org-chart-goose-orchestrator
- **Issues**: https://github.com/JEFH507/org-chart-goose-orchestrator/issues
- **Author**: Javier (@JEFH507)
