# Decide the fate of the invariant test the recovery doc claims

Type: grilling (HITL)
Status: resolved
Blocked by: —
Claimed by: pi session (wayfinder run)

## Question

`docs/diataxis/explanation/two-state-channels.md` states that after any mutation the registry
state agrees with the persisted status, and that a contract test "fails the test suite rather
than shipping". No such test exists in `src/` or `tests/`.

- **Add the test.** Assert that after a mutation the registry's slot state and the persisted
  status agree. Which mutations must it cover, and at which tier — a unit invariant over
  `GenerationGate` and the pipeline, or an HTTP-driven check?
- **Remove the claim.** Correct the doc to say the invariant is untested, and state what
  protects it instead.

One caution before either: the agreement is not exact by design. The persisted `Generating` is
written before `try_claim` claims the slot, and the boot heal exists precisely because the two
can disagree. "Agree after any mutation" needs a precise definition before it can become a test.

## Context

- Deferred by [Recover a dashboard stuck on Generating with no live generation](09-recover-stuck-generating.md)
  (Q7=A). The two wording repairs ride in [Recover a stale Generating from the live registry](11-recover-stale-generating.md);
  this claim did not.
- Found by the 2026-10-06 scout, which verified the test is absent and that the only atomic in
  `gate.rs` is the generation-id counter, not the registry the doc calls "the atomic".
- The test's *implementation* depends on [Recover a stale Generating from the live registry](11-recover-stale-generating.md),
  because that ticket sets the heal coverage the invariant would assert over.

## Done when

- The doc either gains the test or loses the claim, and the choice is recorded.

## Answer

Resolved 2026-10-06. **The claim is removed.** No invariant test exists, and none is added:
the biconditional the doc asserted does not hold, so a test for it would fail, and a test for
the narrow true part would assert the implementation back to itself.

### The claim was false in three ways

Re-checked in the tree, not from the sweep:

- **The biconditional fails in both directions.** On the normal completion path
  `PipelineRun::phase_finalize` writes and persists `Idle` before `GenerationGuard::Drop`
  releases the slot, so a live slot can sit beside a persisted `Idle`. After a panic or a
  `SIGKILL` the slot is released or gone and the persisted status stays `Generating`, which is
  the case `GenerationGate::heal_stale` and the boot heal exist for.
- **The lock claim is false.** `GenerationGate::try_claim` inserts the registry slot under the
  registry write lock, releases that lock, and only then writes the persisted `Generating`.
- **The writer claim is false.** The persisted status has writers beyond the action path:
  `try_claim`, `heal_stale`, `ActionPipeline::reset_persisted_status`, `persist_generation_error`,
  the pipeline's finalize, cancellation and error paths, and `ArrivalTaskContext::run_inner`.

No test in `src/` or `tests/` asserts the agreement. `gate_tests.rs` and `guard_tests.rs`
exercise the pieces (`heal_stale` with and without a slot), which is why the claim looked plausible.

### What changed

`docs/diataxis/explanation/two-state-channels.md`:

- "The single-writer rule" becomes "Who writes each channel". It states the several writers,
  the two disagreement windows, and that the agreement is a convention checked by review.
- The intro's "maintains their consistency with an explicit invariant" is corrected.
- The section links the target contract to
  [Give the input buffer named generation transitions](../../architecture-deepening/issues/09-input-buffer-generation-transitions.md),
  where the module shape that would own the invariant is decided.

The doc change is the whole fix. No test is added, because the truthful invariant belongs to
the deepened module, and that decision lives on the architecture-deepening map.

### Consequence for a decision already made

`Recover a dashboard stuck on Generating with no live generation` rejected "poll heals and
persists" partly because a GET write would "break the documented single-writer rule". That
constraint does not exist. The rejection still stands on its other cost, which is real: the heal
can race a generation start, because the persisted `Generating` is written before the slot is
claimed.

### Arrival defect, surfaced here and ruled out of scope

`bootstrap/init_game.rs::spawn_arrival_task_if_needed` spawns the opening-scene narration at
boot. Arrival claims no registry slot, so `is_busy` is false for the whole arrival call,
`try_claim` from a player action succeeds, and two generations then write one game state. The
submit button does not stop it: `render_action_area` reads the persisted status, which is `Idle`
during arrival.

This is not a stale-record defect. Arrival sets `Generating` in memory, but its only save runs
after the status is already `Idle` or `Error`, so it can never leave a stale persisted record.
`Recover a dashboard stuck on Generating with no live generation` decision 3 listed arrival
among the writers the heal must cover; that item is vacuous, not unmet. The defect is engine
code that no sweep finding needs, so it is out of scope for this map; see Out of scope.
