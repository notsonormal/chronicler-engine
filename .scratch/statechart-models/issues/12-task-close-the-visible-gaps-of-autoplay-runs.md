# Close the gaps of run 1 that a client can see in an Autoplay run

Type: task
Status: open
Blocked by: —

## Question

Add the tests and the spec lines below for the Autoplay run, and fix the sentence about the span of a run's hold. Ticket 09 (Q5) decided the list.

## The gaps

1. **Test 3 of [the trial log](../assets/trial-log.md).** Run 1 waits at `pause_between_two_jobs`. A stop comes, and run 2 starts. Run 1 must start no Job. The test forces the mutant `accept-without-run-check`.
2. **Ticket 03, sketch G1.** An Action after a stop between two Jobs of a run. Section 11 of [the ticket 03 asset](../assets/03-gate-spec-only-model.md).
3. **Ticket 03, sketch G3.** A run on a Game with an old failure line plays its whole budget. The intro of `docs/specs/autoplay.md` says "a Job of the run" where it means the run's own Job.
4. **Doc 1.** The intro of `docs/specs/autoplay.md` says: "The run holds its Game from its start to its end, also between two Jobs". The code removes the holder inside the accept that ends the run (`accept_for_autoplay`), before the run ends. Write the sentence so that it matches the code. The rule `RunHoldsItsGame` quotes this sentence: update the quote. `game_flow.md` keeps the old sentence until ticket 15 rewrites it.

## Context

- Each new scenario goes with a tier-1 test (`validate_feature_spec.py`).
- Do not run this ticket in parallel with ticket 10 or ticket 11. All three edit the gate model.

## Checks

- `python .agents/skills/verify-gate/check.py` passes.
- `python build.py` passes.
- Do not commit.
