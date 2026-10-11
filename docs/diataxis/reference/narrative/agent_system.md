---
diataxis: reference
title: Agent System
---

## Overview

An **agent** is any type implementing the `Agent` trait. Agents are loaded from `AppSettings` at startup and registered in `AgentRegistry`. The engine works with **zero agents** — all agent execution is optional.

Two agents exist: `QuantifierAgent` detects the NPCs present in the narration and any movement destination, and `OptionsAgent` generates the offered options.

## Execution Phases

`ExecutionPhase` declares where in the pipeline the agent runs. Each phase has its own dispatch site:

- **`PreGeneration`** — no dispatcher reads it.
- **`PostGeneration`** — runs after the main LLM call returns, and its results merge into one scene result. `QuantifierAgent` runs here.
- **`OptionsGeneration`** — runs only at the two gated Options sites (turn end and on demand). `OptionsAgent` runs here.

## Per-Agent Backends

The wiring binds each agent to its recorder at startup. That binding, and not the agent's `backend_selector()`, decides the agent's connection.

## NPC Events

The engine derives NPC enter/leave events by comparing the previous quantifier result with the current one.

## Document References

- [`../../explanation/agent_system_design.md`](../../explanation/agent_system_design.md) — why the agent abstraction is shaped this way.
