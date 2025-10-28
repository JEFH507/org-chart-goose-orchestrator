# Architecture One‑Pager

OSS‑first, local‑first orchestrator view — HTTP‑only MVP, OIDC SSO, Vault+KMS, agent pre/post Privacy Guard, metadata-only on controller.

```mermaid
flowchart LR
  subgraph Orchestrator
    DIR["Org Directory & Policy"]
    ROUTER["Task Router & Skills Graph"]
    CTX["Cross-Agent Session Broker"]
    AUD["Audit & Observability"]
    GBL_POL["Global Policies"]
  end

  subgraph Agents["Agent Instances (per role/user/dept)"]
    CS["C-Suite Twin"]
    MKT["Marketing Twin"]
    FIN["Finance Twin"]
    ENG["Engineering Twin"]
    MGR["Manager Twin"]
    IC["IC Twin"]
  end

  subgraph Agent_Internal["Each Agent"]
    UI["UI/CLI/API"]
    CORE["Agent Brain (Goose-based)"]
    PRIV["Privacy Guard (Local LLM + masking)"]
    EXT["MCP Extensions"]
    MEM["Session/Memory"]
    LOC_PROF["Local: Recipes / Memory / Policies / Hints / Ignore"]
    GBL_PROF["Global: Recipes / Memory / Policies / Hints / Ignore"]
  end

  subgraph Models
    LOCALLLM["Local LLM"]
    CLOUDLLM["Cloud LLM"]
  end

  subgraph Data["Data / Integrations"]
    FS["Local FS / Repos"]
    SAAS["SaaS via MCP"]
    DESKTOP["Desktop Software (VS Code, Adobe, etc.)"]
    DB["Databases / Data Lake"]
  end

  DIR --> ROUTER --> CTX --> AUD
  GBL_POL --> ROUTER

  CS <--> DIR
  MKT <--> DIR
  FIN <--> DIR
  ENG <--> DIR
  MGR <--> DIR
  IC <--> DIR

  UI --> CORE --> PRIV --> LOCALLLM
  CORE --> CLOUDLLM
  CORE --> EXT --> SAAS
  EXT --> FS
  EXT --> DB
  EXT --> DESKTOP
  CORE --> MEM
  CORE --> LOC_PROF
  CORE --> GBL_PROF

  CS <--> MKT
  CS <--> FIN
  CS <--> ENG
  ENG <--> MGR
  MGR <--> IC
  CORE <--> CTX
  CORE <--> ROUTER
```

Quick nav: [Docs Home](../README.md) • [Executive Overview](./overview.md)
