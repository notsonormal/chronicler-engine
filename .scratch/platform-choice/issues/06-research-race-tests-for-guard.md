# Research: race tests for the guard and the job id check

Type: research
Status: resolved
Blocked by: 05

## Question

Do the guard and the job id check need tests that explore thread interleavings? If yes, which Rust tool fits (`loom`, `shuttle`, or another), what does it cost to adopt here (async with tokio, build time, the build slot), and which interleavings must the tests cover?

## Context

- [Research 01](01-research-state-machines-and-reducers.md) §6 item 4: a sequential model test does not test interleavings around the guard. The tools are guessed, not researched.
- The busy-state shape comes from [Decide the shape of the Job and the busy state](05-decide-job-and-busy-state-shape.md).
- [Target design](../../turn-state-model/assets/target-design.md) §3.3 (one writer at a time, the guard, the job id check), §5 (stop, restart, switch, reset). [Sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §3.2.
- Test placement: turn-state-model [ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) (seam contracts, the fake that can hold a call).
- Problem B: one heavy cargo step at a time on a 3-CPU, 8 GiB host (`ENVIRONMENT.md`). A tool that needs a separate build profile adds to it.

## Done when

An asset `../assets/research-06-race-tests.md` with a recommendation (no race tests, or one tool), the interleavings to cover, and the cost. Primary sources only.

## Answer

Resolved 2026-10-16 (AFK research). Asset: [research 06](../assets/research-06-race-tests.md).

- **Recommendation: no scheduler tool** (no `loom`, no `shuttle`). Every write is one hold of one lock, so a run equals some order of the holds (inferred). The holdable fake of ruling R6 reaches every class of order; the gaps that it cannot reach give the same records as a gap that it can reach (asset §2.2, inferred).
- **The condition:** each writer takes the guard once for its check and its save. Today's pairs 6 to 9 (research 05 §3) break exactly this. Proposed **guardrail rule 18**: the `Mutex` sits in one small type whose only method is `hold`, and the call-count walker of rule 8 allows one `hold` per function of `gate.rs` (asset §5).
- **Six interleavings** (asset §3): I1 busy refusal of each Job request, I2 the stop matrix (already gap G3), I3 stop then a second Job then the late answer of the first, I4 reset while held, I5 switch and a second Game while held, I6 edit, delete and swipe switch while held. Five need a scenario and a tier-1 test. Cost guessed 150 to 250 test lines.
- **The cost of a tool** (known): both tools need a unit test of `gate.rs`, which ruling R9 does not allow, and a type swap under a cfg or a feature. `loom` has no tokio model, and its documented global `--cfg loom` reaches every dependency (tokio 1.51.1 asserts its IO driver off under it). `shuttle` is randomised and not sound; its tokio wrapper replaces the `tokio` package. Each adds one heavy cargo step to the build slot.
- **When to look again:** a second lock, an atomic with weak ordering in the write path, or a background Agent save outside `hold` (asset §5 item 4).

The owner has not ruled on rule 18 or on "no tool". [Decide the race tests and guardrail rule 18](12-decide-race-tests-and-rule-18.md) holds that decision.
