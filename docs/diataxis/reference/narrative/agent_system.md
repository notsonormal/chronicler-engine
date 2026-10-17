---
diataxis: reference
title: Agent System
---

## Overview

An **agent** is any type implementing one of the hook traits: `AnalysisAgent` or
`OptionsAgent`. Agents are loaded from `AppSettings` at startup and registered in
`AgentRunner`. The engine works with **zero agents** — all agent execution is
optional.

Two agents exist. `QuantifierAgent` detects the Characters present in the
narration and any movement destination. The Options agent
(`options::agent::OptionsGeneratorAgent`) generates the offered options.

## Hooks

A **hook** is a fixed point in a generation where the Job runs the Agents
registered for it. Each hook has one output type. `Hook` declares which one an
Agent serves, and each hook has its own dispatch site:

- **`Analysis`**: runs after the narration was saved. `AgentRunner::analysis`
  merges the `AnalysisOutput` of each Agent into the result that the game rules
  apply. `QuantifierAgent` runs here.
- **`Options`**: runs once per Job, at the two gated Options sites (turn end and
  on demand). The Options hook holds at most one Agent. The Options agent runs
  here and answers with an `OfferedSet`.

## What an Agent gets

An Agent reads values, never storage: a `HookInput` and an `AgentSettings`. The
step that calls the hook owns every read and write, so an Agent writes no game
record.

## Per-Agent Backends

The wiring binds each agent to its recorder at startup, from the Agent's type.
That binding decides the agent's connection.

## NPC Events

The engine derives NPC enter/leave events by comparing the previous quantifier result with the current one.

## Document References

- [`../../explanation/agent_system_design.md`](../../explanation/agent_system_design.md) — why the agent abstraction is shaped this way.
