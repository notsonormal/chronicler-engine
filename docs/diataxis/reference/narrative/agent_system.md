---
diataxis: reference
title: Agent System
---

## Overview

An **agent** is any type implementing one of the hook traits: `BeforeNarrationAgent`,
`AfterNarrationAgent` or `EndOfJobAgent`. The Agent type in `AppSettings` gives the
hook of an optional Agent. Optional Agents are loaded from `AppSettings` at startup
and registered in `AgentRunner`. The engine works with **zero optional agents**.

Three agents exist. `QuantifierAgent` detects the Characters present in the
narration and any movement destination. The Options agent
(`options::agent::OptionsGeneratorAgent`) generates the offered options. The story
memory writer (`memory::StoryMemoryWriter`) rewrites the story memory. It is always
on: the wiring builds it next to the Narrator, outside `AppSettings`, and the Job
holds it apart from `AgentRunner`.

## Hooks

A **hook** is a fixed point in a Job where its Agents run, named by
its position. Each Agent gives one kind of result, and each kind of
result has one landing:

- **Before narration**: runs before the narrator call, only when Messages that the
  story memory does not cover are outside the history space of that call. The story memory
  writer runs here and answers with the new story memory. The Job shows it in the
  narrator call and saves it in the new Swipe state. One writer call gets the oldest
  of those Messages up to one history space, measured as the history cut measures
  them. The rest wait for the next cut.
- **After narration**: runs after the text save and before the state update.
  `AgentRunner::after_narration` runs the Agent and returns its `QuantifierResult`:
  the Characters present and the destination. The game rules apply it.
  `QuantifierAgent` runs here. A runner with no after-narration Agent leaves the Room
  and the Characters present of the previous state.
- **End of the Job**: runs once per Job, at the two gated Options sites (after a
  Narration and on demand). `AgentRunner::end_of_job` runs the Agent. The hook holds
  at most one Agent. The Options agent runs here and answers with an `OfferedSet`.

The Quantifier and the story memory writer are state writers. When a state writer fails, the Job keeps its part of the previous Swipe state and continues.

## What an Agent gets

An Agent reads values, never storage: the input of its hook and the settings of its call. The
step that calls the hook owns every read and write, so an Agent writes no game
record. The `AgentSettings` give the preset text, the call settings and the Options
count. The calling step fills them from the preset of the Game and the Connection of
the hook's slot. The instructions of the story memory writer are code, so it gets no preset.

## Per-Agent Connections

The wiring binds each agent to its recorder at startup, from the Agent's type.
The Quantifier uses the Quantifier slot. The Options agent uses the Options slot.
An empty Options slot uses the Narration Connection. The story memory writer uses the
Narration slot, and records its calls under its own Role.

## Options

Options are pre-written Actions that the engine offers while a Game is idle.
The offered set of the Options agent lives in the Game's dock record, apart from
the story. The always-on toggle is a setting of the Game, inherited from its
World at creation. One agent and one offered set serve both narrator modes.

## Answer shapes

The prompt of each Agent asks the model for one of the shapes below.

- **Quantifier.** One JSON object. It carries `characters_present`, the list of Character ids, and `movement`, with the fields `type` and `destination`. The `destination` names the Room the player moves to. The `type` does not change the move.
- **Options.** One of three shapes: a `<suggestion>` tag for each option, a numbered list, or a `Suggestion N:` line for each option.
- **Story memory.** Plain text. A saved story memory is always one whole answer of the writer. The word count that the prompt asks for follows the response length of the call.

## Document References

- [`../../explanation/agent_system_design.md`](../../explanation/agent_system_design.md) — why the agent abstraction is shaped this way.
