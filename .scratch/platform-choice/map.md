# Map: platform choice

Labels: wayfinder:map

## Destination

A decision: keep the Chronicler Engine in Rust or rewrite it. For a rewrite, the decision also names the language and platform, the test oracle (black-box tests from the specs), and the migration path. It uses the clean design from the [turn state model](../turn-state-model/map.md) map and measured costs of the feedback loop.

## Notes

- **Plan, don't do.** Tickets produce decisions. A rewrite, if one is chosen, goes to a plan written after the map.
- **Origin:** [Decide whether and how to run a clean-room design of the generation model](../turn-state-model/issues/17-decide-clean-room-run.md), in the turn state model map. The owner raised the language question during that grilling. Read its Answer.
- **Seed:** [build, test and port facts from the ticket-17 grilling](assets/seed-build-and-port-facts.md). Read it before charting.
- **Not charted yet.** This file is a skeleton: Destination, Notes and fog. A separate session charts the frontier and creates the tickets (wayfinder "Chart the map", steps 2 and 4).
- **The problems that a language move must solve** (owner, 2026-10-12):
  - A: the edit-to-test loop after a change;
  - B: parallel agents that wait for the build slot or run out of memory;
  - D: design churn and defects.

  The full gate time is not one of them. About 80 % of the gate is the browser tier, and a new language keeps most of that (inferred).
- **The owner's position.** Performance does not matter much for this application. The owner is not sure that TypeScript is the right language. The owner expects the answer to be clearer after the clean design. The owner also asked how much a clean-room design gives, compared with "starting fresh with a clean plan, stronger models and a stronger understanding of how to build an AI-native project".
- **Dependency.** The final decision waited for the turn-state-model map. That map is resolved (2026-10-14): see the handoff below.
- **From the turn-state-model map (2026-10-12).** That map now ends with a verdict: [Decide where the new design of the game-state lifecycle makes sense](../turn-state-model/issues/24-decide-fix-or-rewrite.md). The verdict waits for [Sketch the new design next to today's code](../turn-state-model/issues/25-sketch-new-design-next-to-code.md). The owner gave this map two questions in that grilling (Q4, Q7): how and where the new design is built, and whether the edges (the Agents, the world rules and Triggers, prompt assembly, storage) and the untouched parts get a clean design of their own.
- **Handoff from the turn-state-model map (2026-10-14).** [Decide where the new design of the game-state lifecycle makes sense](../turn-state-model/issues/24-decide-fix-or-rewrite.md) resolved that map. This map takes:
  1. **The verdict.** The new design for all six areas of the designed scope: the records and their storage, the writer rule and the guard, the Job and its steps, the reads of the views, create and reset, and the dock. The design is [`target-design.md`](../turn-state-model/assets/target-design.md) version 2, with its code shape in [sketch 25](../turn-state-model/assets/sketch-25-new-design.md) and, for the Agents, [sketch 24](../turn-state-model/assets/sketch-24-agent-hooks.md).
  2. **The cost.** 3,577 non-test lines replaced or deleted (known). About 1,765 new lines, plus about 80 for the Agent hooks (guessed). About 7,400 unit-test lines go with the replaced modules. The tier-1 scenarios, the storage tier and the browser tier stay ([ticket 26](../turn-state-model/issues/26-decide-test-placement.md) R10).
  3. **Wishes that must stay cheap.** Side by side in two browser tabs (only `CurrentGame` reads the active Game). Background Agents (sketch 24 §5). Many Agents at fixed hooks, as in Marinara.
  4. **What depends on Rust.** The guardrail rules at compiler level (sketch 25 §6.2 rules 4, 5, 6, 9, 14, 15; sketch 24 rule 16) and one trait per Agent hook. A language with weaker types moves them to walkers or tests.
  5. **What this map decides.** How and where the design is built (in place, as a new module, or in another language). Whether the edges get a clean design of their own: the inside of the Agents, prompt assembly, storage. The test oracle, from ticket 26's placement.
- **`architecture-deepening`** pauses by owner intent until this decision lands. Its map has no marker. Ticket 24 resolved its tickets 04, 05, 07, 08, 09, 10 and 13 (2026-10-14).
- **Skills:** `/grilling` and `/domain-modeling` for decisions; `/research` for outside facts; `/prototype` for a measured slice.

## Decisions so far

## Not yet specified

- **Language and platform candidates.** TypeScript is the owner's first idea, not a decision. Other languages, and building on an existing platform (SillyTavern and Marinara are JavaScript and TypeScript), are open.
- **What an AI-native project needs.** Fast feedback loops, black-box tests from the specs, how well the models write each language, how much a compiler catches before a test runs, and the docs and module shape that agents need. A strict compiler such as Rust's catches many agent errors. A language with a faster loop may give some of that up (inferred).
- **A measured feedback loop.** Time the same edit-to-test loop on a thin slice in each candidate language, under the host limits (3 CPUs, 8 GiB, shared). Also measure Rust options not tried yet (seed §4).
- **The test oracle.** The HTTP tests call the router in-process and the browser tests use `playwright-rs`, so no test can check a server in another language. A rewrite needs black-box tests over HTTP and the browser, made from the specs, that run against both servers.
- **The migration path.** Replace the parts one at a time behind the same HTTP interface, or rewrite everything at once. This also covers what happens to new features during the move.
- **The parts tied to Rust.** `harper-core` (text check), Askama templates, the `arch-lint` and `syn` guardrails, the `build.py` tooling, `CODING_STANDARDS.md`, and the Rust skills.
- **The design beyond scope B.** The turn-state-model map designs only the game-state lifecycle. A rewrite also needs designs for the Agents, Triggers, prompt assembly, storage, the HTTP views and the LLM transport. Four of those subsystems have no spec.

## Out of scope
