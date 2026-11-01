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
  `QuantifierAgent` runs here. The runner logs a failed Agent, and the Job continues
  with the Room and the Characters present of the previous state. A runner with no
  after-narration Agent does the same. The Quantifier does the same when all its
  model calls fail.
- **End of the Job**: runs once per Job, at the two gated Options sites (after a
  Narration and on demand). `AgentRunner::end_of_job` runs the Agent. The hook holds
  at most one Agent. The Options agent runs here and answers with an `OfferedSet`.

## What an Agent gets

An Agent reads values, never storage: a `HookInput` and an `AgentSettings`. The
step that calls the hook owns every read and write, so an Agent writes no game
record. The `AgentSettings` give the preset text, the call settings and the Options
count. The calling step fills them from the preset of the Game and the Connection of
the hook's slot.

## Per-Agent Connections

The wiring binds each agent to its recorder at startup, from the Agent's type.
The Quantifier uses the Quantifier slot. The Options agent uses the Options slot.
An empty Options slot uses the Narration Connection.

## Options

Options are pre-written Actions that the engine offers while a Game is idle.
The offered set of the Options agent lives in the Game's dock record, apart from
the story. The always-on toggle is a setting of the Game, inherited from its
World at creation. One agent and one offered set serve both narrator modes.

## Answer shapes

A preset asks the model for one of the shapes below. The parser of each agent reads only the shapes of that agent.

- **Quantifier.** One JSON object. It carries `characters_present`, the list of Character ids, and `movement`, with the fields `type` and `destination`. The `destination` names the Room the player moves to. The parser keeps `type`, and the game rules read `destination` alone.
- **Options.** One of three shapes: a `<suggestion>` tag for each option, a numbered list, or a `Suggestion N:` line for each option.

## Document References

- [`../../explanation/agent_system_design.md`](../../explanation/agent_system_design.md) — why the agent abstraction is shaped this way.
