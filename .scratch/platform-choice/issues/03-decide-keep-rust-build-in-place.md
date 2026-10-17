# Decide: keep Rust and build the new design in place

Type: grilling
Status: resolved
Blocked by: —

## Question

Keep the Chronicler Engine in Rust or rewrite it in another language? If it stays in Rust, what does this map do next?

## Context

- The map's first Destination: a decision to keep Rust or rewrite, with the language, the test oracle and the migration path for a rewrite.
- [Research 01](01-research-state-machines-and-reducers.md) and [Prototype 02](02-prototype-job-as-pure-state-machine.md): neither made the language choice clearer.
- The handoff from the turn-state-model map: [target design](../../turn-state-model/assets/target-design.md) version 2, [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md), [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md), test placement in [ticket 26](../../turn-state-model/issues/26-decide-test-placement.md).

## Answer

Resolved 2026-10-16, in the charting grilling (three rounds).

**Keep Rust. Build target design version 2 in the current crate, in place** (owner, round 2 Q1 = A). The owner's reason: the design and the plan are solid now. Supporting facts: the design addresses problem D (design churn and defects) in any language (ticket 17, research 01; inferred). A port costs 49,000 lines, a new black-box oracle that runs against two servers, four missing specs, and the compiler-level guardrail rules (seed §3; known).

**The Destination is redrawn** (round 3 Q1 = A). The map is now "the Rust rebuild". It ends with a build plan that leaves nothing to decide. The folder path stays, because the turn-state-model map links to it. The map takes over:

1. **The full design** (round 2 Q1, round 3 Q2 = A). The follow-ups of tickets 01 and 02 become tickets here, not later work: the Job and busy-state shape, gaps G1 to G3, fork 1, the race tests. The edges (the inside of the Agents, prompt assembly, storage, Triggers, the HTTP views, the LLM transport) stay as fog until the sort of `architecture-deepening` shows their shape.
2. **Problems A and B** (round 3 Q3 = B). Without a rewrite, the edit-to-test loop (A) and the build slot and memory limits for parallel agents (B) stay. One measured ticket decides the crate split and the dev-build loop (Cranelift), after the module shape.
3. **The test rework** (round 2 Q3). Redoing the tests is part of the change. Ticket 26 R1 to R11 is the start. The rule outside the designed scope, which ticket 26 R1 left open, is a ticket here.
4. **`architecture-deepening`** (round 2 Q5, round 3 Q5 = A). This map supersedes it, because the new design changes the architecture that map deepens. One AFK ticket sorts its ten open tickets: settled by the design, a ticket here, or moved to a later effort. Then that map closes, with a pointer here.

**The fog, row by row** (round 3 Q4 = A):

| Fog item | Result |
|---|---|
| Language and platform candidates | Out of scope |
| What an AI-native project needs | Split: the language part is out of scope; module shape and docs for agents stay as fog |
| A measured feedback loop | Ticket: [Prototype: decide the crate split and the dev-build loop](07-prototype-crate-split-and-dev-build-loop.md) |
| The test oracle across two servers | Out of scope. The test rework stays in scope: [Decide the test rule outside the designed scope](08-decide-test-rule-outside-designed-scope.md) |
| The migration path | Ticket, with a new meaning: [Decide the build path](09-decide-build-path.md) |
| The parts tied to Rust | Out of scope: they stay |
| State-machine follow-ups | Tickets: [Decide the shape of the Job and the busy state](05-decide-job-and-busy-state-shape.md) and [Research: race tests for the guard and the job id check](06-research-race-tests-for-guard.md) |
| The design beyond scope B | Stays as fog. [Research: sort the open architecture-deepening tickets against the new design](04-research-sort-architecture-deepening.md) sorts part of it |

Problems A and B, restated for the record: **A** is the edit-to-test loop (one example: 53.2 s for the first targeted run after a Rust edit, then 2.6 s; known). **B** is parallel agents that wait for the one build slot or run out of memory on the 3-CPU, 8 GiB host (known).
