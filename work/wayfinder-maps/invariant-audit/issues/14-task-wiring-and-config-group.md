# Decide and close the wiring and config group (tier 4)

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Give the one verdict for the wiring and config group under the enforcement order of the map Notes, and put it in place. A part of the group can get its own verdict when one mechanism does not fit it.

| Finding | Invariant that nothing enforces |
|---|---|
| llm-08 | Every configured Agent is registered. `wiring.rs:152` replaces a `from_configs` error with `unwrap_or_default()`, so both Agents silently disappear at boot. |
| llm-09 | The agent name that a call records is a `Role` name. The Quantifier passes the literal `"quantifier"`, and the Role health table filters by that name. |
| world-08 | The preset ids in the settings exist and have the type of their slot. No boot check exists, but one exists for the Connection slots. |
| world-09 | The `preset_type` of a preset seed file matches its folder. |

The candidate mechanism, from the reports: boot checks that return their error, as `check_connection_slots` already does on the line before llm-08; and a `Role` argument in place of a string name.

## Context

- Findings in full: llm-08 and llm-09 in [the llm report](../assets/llm.md), world-08 and world-09 in [the world report](../assets/world.md). The coordinator read llm-08 in the code (`src/bootstrap/wiring.rs:147-152`).
- The code-concepts synthesis asks the same question as llm-08 (its Q24). That map is a catalogue and fixes nothing.
- [Decide the runtime World validation](02-grilling-runtime-world-validation.md) decides the load-time validation of the World. If it is resolved, use the same mechanism for the presets.
- A boot check that fails on today's seed data stops the server. That changes what the player sees, so ask the owner first (decide-and-do rule).
- The six end conditions of the map Notes apply.

## Answer
