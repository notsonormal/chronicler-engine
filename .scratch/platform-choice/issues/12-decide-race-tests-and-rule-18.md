# Decide the race tests and guardrail rule 18

Type: grilling
Status: resolved
Blocked by: 06

## Question

Does the build plan take the recommendation of [research 06](../assets/research-06-race-tests.md): no scheduler tool, five new scenarios with tier-1 tests (I1, I3 to I6), and guardrail rule 18 (one `hold` per writer)? If not, which tool, and which exception to ruling R9 of turn-state-model [ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) does it get?

## Context

- [Research: race tests for the guard and the job id check](06-research-race-tests-for-guard.md) and its asset: §2 (why the held fake reaches every class of order, and the one-hold condition), §3 (the six interleavings), §4 (the tool facts and costs), §5 (rule 18 and when to look again).
- [Decide the shape of the Job and the busy state](05-decide-job-and-busy-state-shape.md): the busy-state shape, gap G3 (the stop matrix, which is I2), and rule 17 (the precedent for one walker row).
- Sketch 25 §6.2: rule 6 (the guard is a private `MutexGuard`), rule 8 (the call-count walker that rule 18 reuses).
- Research 05 §3 (turn-state-model): today's nine writer pairs.

## Done when

The owner has ruled on the tool, on rule 18, and on the five scenarios. The answer states the changes to sketch 25 §6.2 (rule 18) and the scenario list for the build plan.

## Comments

**2026-10-16, from [Decide the build path](09-decide-build-path.md) (resolved).** Each guardrail rule lands in the slice that creates the code it guards (ticket 09 Q10), and each doc change lands in the slice that makes it true (Q12). The slices are P0 the test rule, P1 `ActiveGame`/`CurrentGame`, P2 delete the arrival opening, P3 `SwipeState` with `apply_world_rules`, P4 the Agent edge on values, P5 the storage `Tx` mechanism, then the core step. So the answer here should name, for each rule or doc it keeps, the slice where it lands. [Write the build plan](14-write-the-build-plan.md) waits on this ticket.

## Answer

Resolved 2026-10-17 by the owner, in a grilling (Q1 to Q9). A first session asked Q1 to Q7 and got the owner's answers, then stopped before it wrote anything; the owner confirmed those answers in a second session, which added Q8 and Q9 after a check of the code and sketch 25. Labels: **known**, **inferred**, **guessed**.

### Rulings

| Q | Ruling | Slice |
|---|---|---|
| Q1 | **No scheduler tool** (no `loom`, no `shuttle`). Deterministic tier-1 tests with the holdable model fake, plus rule 18. No exception to ruling R9 or R2 clause 3 of turn-state-model ticket 26. No second cargo configuration in the build slot | — |
| Q2 | **Accept rule 18**, at compiler level plus one row on the call-count walker of rule 8. The re-look triggers of [research 06](../assets/research-06-race-tests.md) §5 item 4 go into the rule's comment | Core step (with `gate.rs`) |
| Q3 | **Five new scenarios** in `docs/specs/`, each with one tier-1 test: I1 (Action and `/options`), I3, I4, I5, I6. I2 stays gap G3 of ticket 05. The tests of 12.1 and 15.1 are rewritten to hold a real Job through the route (ticket 26 R7 condition 2 forbids today's direct `try_claim` arrange) | See Q9 |
| Q4 | **The hold capability of the model-port fake** (ticket 26 R6) lands in P4, with the model-port fake and the seam contracts | P4 |
| Q5 | **Two fast clicks** is one more assertion inside I1 (two accepts at the same moment, one Input saved). No own scenario | With I1 |
| Q6 | **One test file**, `tests/http/races.rs`, registered in `tests/http/mod.rs` and in the generated `tests/AGENTS.md`. Its module header names the cases. The scenarios stay in the `docs/specs/` file of their route | First slice that adds a case to it (P4) |
| Q7 | **Rule 18's text lives in this answer**, as ticket 05 did for rule 17. Sketch 25 §6.2 is not edited. The build plan carries the full rule list | — |
| Q8 | **Rule 18 counts indirect holds.** A call to a `gate.rs` function that calls `hold` counts as a hold | Core step |
| Q9 | **P4 rewrites 12.1 and 15.1 and writes I1's Action and `/options` cases**, on today's pipeline, with the holdable fake. They stay green through the core step. I3 to I6 and the I1 cases for edit, delete and swipe switch (I6) are new behaviour and land in the core step | P4, core step |

### Rule 18 (addition to sketch 25 §6.2)

With rule 16 (sketch 24) and rule 17 (ticket 05), the list now has 18 rules.

| # | Rule | Reason | Level | How |
|---|---|---|---|---|
| 18 | Each writer takes the guard once: every function of `application/generation/gate.rs` makes at most one **hold**, where a hold is a call to `hold` or a call to another function of `gate.rs` that makes a hold | A check and a save in two holds is today's race class (research 05 §3, pairs 6 to 9). One hold per writer keeps every write one critical section and keeps the lock order guard → backend mutex → connection mutex (ticket 11 constraint C10). The indirect count closes the gap that no held model call reaches: for example `replace` written as `stop` then `create` lets an Action on the old Game in between, with no LLM call in that gap (Q8) | compiler + syn (call count) | Compiler: the `Mutex` is a private field of one small type in `gate.rs` whose only method is `hold(game, \|entries, tx\| ..)`; `gate.rs` has no child modules, so Rust privacy keeps `hold` inside `gate.rs`. Syn: one row on the call-count walker of rule 8. The row first collects the functions of `gate.rs` that call `hold`, then allows each function at most one call to `hold` or to a collected function (name match inside one file). Cost: about 20 more walker lines than the direct count (guessed) |

Rule comment, the re-look triggers (research 06 §5 item 4): a second lock (for example one guard per Game); an atomic with weak ordering in the write path; a background Agent save that checks "this Swipe is still active" outside `hold`. Any of these breaks research 06 §2.1, and then `loom` earns a seam contract on the new primitive.

Rule 18 extends rule 6 (the guard is a private `MutexGuard` that no function returns). Ticket 13 owns the call-count walker's tool and cost; this ticket adds one row to it.

### The scenario and test list for the build plan

| Case | Scenario | Test | Slice | Green today? |
|---|---|---|---|---|
| 12.1 Retry while busy | exists (`docs/specs/swipe_new.md`) | rewritten: hold a real Job through the route; replaces `try_claim` at `tests/http/swipe_new.rs:1264` | P4 | yes (known) |
| 15.1 Retrigger while busy | exists (`docs/specs/retrigger.md`) | rewritten, same way; replaces `try_claim` at `tests/http/retrigger.rs:306` | P4 | yes (known) |
| I1 Action while busy, with the two-fast-clicks assertion | new | new, `races.rs` | P4 | yes (known: `actions.rs:40` maps `ConcurrentGeneration` to "Still thinking...") |
| I1 `/options` while busy | new | new, `races.rs` | P4 | yes (inferred: `process_options` goes through the same `claim_and_spawn`, `options.rs:32`). If it is red when P4 writes it, it moves to the core step |
| I2 stop at each stop point | gap G3 (ticket 05) | the generated stop test | Core step | no |
| I3 stop, second Job, the late save of the first | new | new, `races.rs` | Core step | no |
| I4 reset while held | new | new, `races.rs` | Core step | no |
| I5 Job held on Game A, act on Game B, switch back | new | new, `races.rs` | Core step | no (today a switch discards the Job) |
| I6 edit, delete, swipe switch while held | new | new, `races.rs` | Core step | no (today edit and delete check nothing; swipe switch answers `503`, inferred from `message_service.rs:164` and `http/error.rs:15`) |

Cost: about 150 to 250 test lines (guessed, research 06). The 12.1, 15.1 and I1 tests in P4 are the first users of the holdable fake. They pass through the core step unchanged, so the core step has a busy-refusal net that it did not write itself.

### Notes for the build plan (ticket 14)

- P4 also changes `wait_idle` (`tests/http/support/http_requests.rs:109`) only if P4 needs it. The core step must change it, because it reads `generation_gate.is_busy`, which goes (inferred).
- `src/adapters/driving/http/action/handlers/actions_tests.rs:289-300` arranges busy with `try_claim` too. It is a unit test on the legacy list (ticket 08 R4); the core step breaks it, so it goes or moves under the one rule (ticket 08 R5).

**2026-10-17, amended by [Decide the four gaps that the build plan found](16-decide-the-build-plan-gaps.md) Q3.** P4 also makes the model-port fake strict and runs the real Quantifier at tier 1, not only the hold.
