# ADR 0013: Lead/Worker and Cost-Aware Model Orchestration

Status: Accepted (MVP)
Date: 2025-10-27

## Context
We need predictable cost and policy-aligned model selection across tasks.

## Decision
- Implement guard-first flow and select lead/worker per task class, with downshift to cheaper models for summarization. Record token usage in audit events.

## Consequences
- Predictable costs and policy-compliant routing; added selection complexity.

## Alternatives
- Single-model strategy; manual operator choice; less flexible and less cost-efficient.
