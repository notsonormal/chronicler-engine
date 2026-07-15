---
diataxis: reference
title: Agent System
---

> **Diátaxis mode:** Reference. This document describes the agent abstraction as it is: the trait contract, the execution phases where agents run, the result types they return, and how the registry wires per-agent backends at startup. The problem it solves for the reader is *look-up*: what does an `Agent` look like to the pipeline, when does it run, and what can it return. Trait and result definitions live in `src/application/agents/`.

## Overview

The engine has an agent architecture. An **agent** is any type implementing the `Agent` trait. Agents are loaded from `AppSettings` at startup and registered in `AgentRegistry`. The pipeline iterates registered agents at each `ExecutionPhase` and dispatches them in registration order. The engine works with **zero agents** — all agent execution is optional.

The single agent in production today is `QuantifierAgent` (post-generation, scene analysis). It detects NPCs that appeared in the narration and any movement destination, returning a `StatePatch` that `GameService` translates back into a `QuantifierResult` for the action pipeline.

## Trait Contract

Every agent implements four capabilities:

- **Name** — a human-readable identifier used in logs and forensics.
- **Phase** — declares when the agent runs, as an `ExecutionPhase`; dispatched by the registry.
- **Backend selector** — records which LLM backend the agent prefers, as a `BackendSelector`. The selector is recorded at config time but is not consulted by the registry today; agents are bound to a recorder at wiring time.
- **Execute** — runs the agent against an `AgentContext` and returns an `AgentResult` (one of three variants).

Agents are stored as `Box<dyn Agent>` in the registry.

## Execution Phases

`ExecutionPhase` declares where in the pipeline the agent runs:

- **`PreGeneration`** — no dispatcher reads it today; the pipeline iterates only `PostGeneration` agents.
- **`PostGeneration`** — runs after the main LLM call returns. `QuantifierAgent` runs here.

The pipeline shape today:

1. Load state from snapshot.
2. Generate main narration via LLM.
3. Run `PostGeneration` agents — `QuantifierAgent` analyzes the narration and returns detected NPCs + movement.
4. Apply agent results to engine state and save the snapshot.

## Result Types

An agent's `execute` returns one of three `AgentResult` variants:

- **`PromptDirective(String)`** — inject text into a future prompt. Not constructed by any registered agent today.
- **`StatePatch(StatePatch)`** — propose a state mutation. `QuantifierAgent` returns this with the NPCs it detected in the narration and any movement destination. `GameService` translates the patch back into a `QuantifierResult` for the action pipeline.
- **`NoOp`** — the agent ran but has nothing to report.

`StatePatch` carries a `confidence` field rated `High`, `Medium`, or `Low`. This rating reflects how certain the agent's LLM call is about its detected entities.

## Agent Registry

`AgentRegistry` is constructed at startup from `AppSettings.agents` via `AgentRegistry::from_configs(&settings.agents)?`. Each agent receives a recorder bound at wiring time (see "Backend Selection" below); the `agent_type` discriminator selects the implementation, and `enabled` controls registration.

If no agent config exists in `AppSettings`, the registry injects defaults for backward compatibility:

- `quantifier` agent enabled, `PostGeneration`, `UseNamed("quantifier")` backend.

## Per-Agent Backends

Each agent can use a different LLM connection. The wiring lives in `bootstrap::wiring`:

- **Main narrator** uses `narration_connection_id` from settings.
- **Quantifier** uses `quantifier_connection_id` from settings, or a custom connection named via the `UseNamed` selector.

The quantifier recorder is pre-built and passed to `AgentRegistry::from_configs_with_storage`. The agent's declared `backend_selector()` is recorded but not consulted — the registry relies on the wiring-time binding.

The `UseMain` selector exists in the enum; no agent currently selects it.

## Document References

- [ADR-009: Agent Trait and Registry Architecture](../../docs/adr/adr-009-agent-trait-registry.md) — `Agent` trait + `AgentRegistry` + the extension procedure for new agents.
- [ADR-006: Quantifier-Driven Game Systems](../../docs/adr/adr-006-quantifier-systems.md) — quantifier-driven movement + NPC detection; the `QuantifierAgent`'s purpose.
- [ADR-027: Hexagonal Architecture Migration](../../docs/adr/adr-027-hexagonal-architecture-migration.md) — agent constructors carry `Option<Arc<Storage>>` directly under the storage-direct exemption (deferred to G1-B).
- [`../explanation/agent_system_design.md`](../explanation/agent_system_design.md) — why the agent abstraction is shaped this way and which tradeoffs it encodes.
- [`./triggers.md`](./triggers.md) — uses the quantifier's NPC + movement output as the precondition for trigger evaluation.
- [`./action_pipeline.md`](./action_pipeline.md) — pipeline home for the `PostGeneration` dispatch.
