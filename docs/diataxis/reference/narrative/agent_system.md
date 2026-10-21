---
diataxis: reference
title: Agent System
---

## Overview

An **agent** is any type implementing one of the hook traits: `AfterNarrationAgent` or
`EndOfJobAgent`. The Agent type in `AppSettings` gives its hook. Agents are loaded
from `AppSettings` at startup and registered in `AgentRunner`. The engine works with
**zero agents** — all agent execution is optional.

Two agents exist. `QuantifierAgent` detects the Characters present in the
narration and any movement destination. The Options agent
(`options::agent::OptionsGeneratorAgent`) generates the offered options.

## Hooks

A **hook** is a fixed point in a Job where the Job runs the Agent registered for
it, named by its position. Each Agent gives one kind of result, and each kind of
result has one landing:

- **After narration**: runs after the text save and before the state update.
  `AgentRunner::after_narration` runs the Agent and returns its `QuantifierResult`:
  the Characters present and the destination. The game rules apply it.
  `QuantifierAgent` runs here.
- **End of the Job**: runs once per Job, at the two gated Options sites (after a
  Narration and on demand). `AgentRunner::end_of_job` runs the Agent. The hook holds
  at most one Agent. The Options agent runs here and answers with an `OfferedSet`.

## What an Agent gets

An Agent reads values, never storage: a `HookInput` and an `AgentSettings`. The
step that calls the hook owns every read and write, so an Agent writes no game
record.

## Per-Agent Backends

The wiring binds each agent to its recorder at startup, from the Agent's type.
That binding decides the agent's connection.

## NPC Events

The game rules derive Character enter and leave events by comparing the Characters present before the generation with the `QuantifierResult`.

## Document References

- [`../../explanation/agent_system_design.md`](../../explanation/agent_system_design.md) — why the agent abstraction is shaped this way.
