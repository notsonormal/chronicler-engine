# Seed: the state-machine idea (Matt Pocock post)

Given by the owner on 2026-10-15, for [Research: what explicit state machines and reducers give the new design, in Rust and in other languages](../issues/01-research-state-machines-and-reducers.md).

Labels: **known** = read in a file. **inferred** = follows from known facts. **guessed** = not supported by a source.

## 1. The source

- Post: <https://x.com/mattpocockuk/status/2110818314499072493>. A direct fetch returned HTTP 403 on 2026-10-15. The owner gave the text (known):

  > I think I'm addicted to getting agents to build state machines
  >
  > Every time I see an agent has built a state machine (or even just a reducer) I can feel a huge bump in the robustness
  >
  > And I know that future updates will be trivial and testable
  >
  > Would recommend for frontend

- The post is a practice report, not a measurement. It gives no data (known).
- The post recommends the idea for frontend code only (known). This repo's candidate places are mostly backend (inferred).
- An outside chat assistant said that the replies mention his past work on XState, and David Khourshid, the author of XState (guessed, not checked).

## 2. The chat assistant's claims about this repo

The owner pasted a chat answer that applies the idea to Chronicler Engine. The assistant read the public GitHub repo, so it describes today's code, not the [target design](../../turn-state-model/assets/target-design.md). Treat each claim as a guess until a session checks it.

| Claim | Label | Note |
|---|---|---|
| `generation_status.rs` says "live state machine lives in `application/pipeline/`" | known | The module doc says this (tree `4ad85da8`). |
| The generation has `GenerationStatus` (Idle, Generating, Error) and `GenerationPhase` (Narrating, Quantifying, GeneratingEvent, Options), and two state channels | inferred | Matches `docs/diataxis/explanation/two-state-channels.md`. The target design replaces this. |
| Highest value: make the generation an explicit FSM with states such as Idle, Claimed, Narrating, Quantifying, Committing, TriggerContinuation, Error | guessed | The state names are the assistant's. Target design §4.4 already has a states table for one generation and one for the Job. |
| GameState as a reducer: "event → new GameState", which makes Snapshots replayable | guessed | Sketch 25 already makes the world rules a pure function, `apply_world_rules(start, analysis, world, mode)`. |
| The HTMX dashboard can be a small state machine | inferred | The action area already uses one client state machine: [Rework the action area so a text check cannot strand it](../../dashboard-ui-review/issues/42-rework-the-action-area.md). |
| The idea applies "almost everywhere" in the project | guessed | No evidence given. |
