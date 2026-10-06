# Decide the fate of the invariant test the recovery doc claims

Type: grilling (HITL)
Status: open
Blocked by: —

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
