# Close the gaps of run 1 that a client can see in an Autoplay run

Type: task
Status: resolved
Blocked by: —

## Question

Add the tests and the spec lines below for the Autoplay run, and fix the sentence about the span of a run's hold. Ticket 09 (Q5) decided the list.

## The gaps

1. **Test 3 of [the trial log](../assets/trial-log.md).** Run 1 waits at `pause_between_two_jobs`. A stop comes, and run 2 starts. Before you start run 2, arm a second `state.job_deps.pause.hold(game)`, so that run 2 waits at its pause before its first accept (check `wait_paused`). Then release run 1. Run 1 must start no Job: its task ends (`wait_for_jobs_running_at_most(&state, 1, ..)`), the narrator calls and the story stay as they were, and the status display answers the progress of run 2. Then release run 2. The order forces `accept-without-run-check` only while run 2 has no Job in flight (`gate.rs:380-389`). Scenario: autoplay.md 41.26. Test: `tests/http/autoplay/run.rs`.
2. **Ticket 03, sketch G1.** An Action after a stop between two Jobs of a run. The Action's Job keeps the Game when the stopped run comes to its accept. Scenario 41.27. The test forces the code form of `run-end-without-run-check`. The sketch is in section 6 of [the ticket 03 asset](../assets/03-gate-spec-only-model.md).
3. **Ticket 03, sketch G3.** A run on a Game with an old failure line plays its whole budget. The run's accept also clears the line. Scenario 41.28 (sketch: section 6). In the list of reasons in the intro of `docs/specs/autoplay.md`, change "A Job that ends with a failure line." (line 19) to "A Job of the run that ends with a failure line." No mutant reads this order.
4. **Doc 1.** The intro of `docs/specs/autoplay.md` says: "The run holds its Game from its start to its end, also between two Jobs". The code removes the holder inside the accept that ends the run (`accept_for_autoplay`), before the run ends. Write the sentence so that it matches the code. The new sentence (plan review, owner, decision 8, Option A): "The run holds its Game from its start until it ends, also between two Jobs. When the budget or an unreadable record ends the run, its last accept lets go of the Game." The rule `RunHoldsItsGame` quotes this sentence: update the quote in `VerifyGate.tla:574` and in `SKILL.md:107`. `check.py` does not compare quotes. For a Job that ends with a failure line, the holder goes in `end_autoplay` when the run ends (`driver.rs:102`, `:137-140`), so the new sentence must also hold for that case. `CONTEXT.md:88` has the same sentence. Change it in this ticket too, so that the glossary keeps no claim that the spec drops (plan review, owner, decision 8). `game_flow.md` keeps the old sentence until ticket 15 rewrites it.

## Context

- Each new scenario goes with a tier-1 test (`validate_feature_spec.py`).
- Do not run this ticket in parallel with ticket 10 or ticket 11. All three edit the gate model.

## Checks

- `python .agents/skills/verify-gate/check.py` passes.
- `python build.py` passes.
- Do not commit.

## Not in scope

- The rewrite of `game_flow.md` (ticket 15).
- `moved-job-ends-as-failure`.
- Any code change.

## What already exists

- `pause_between_two_jobs`, `pause_before_first_job`, `stop`, `assert_status`, `story_after_opening`, `narrator_calls`, `wait_for_run` (`tests/http/autoplay/mod.rs`).
- `wait_for_jobs_running_at_most` (`tests/http/support/http_requests.rs:266`).
- The 41.17 test as a pattern (`tests/http/autoplay/run.rs:942`).
- Sketches G1 and G3: section 6 of [the ticket 03 asset](../assets/03-gate-spec-only-model.md).

## Failure modes

- The test arms a pause hold before the earlier hold has fired. Then `pause.rs:56` panics. Arm the new hold only after `wait_paused`.
- Run 2 has a Job in flight when run 1 accepts. The test then passes with the mutant on (a weak test). Gap 1 says how to avoid it.
- A waited step never comes. The `wait_*` call returns false, and the assert fails. A dropped `PauseHandle` releases the loop (`pause.rs:90`).
- A model call that the script does not have. `wait_for_run` reports it.
- A stale quote of the spec in the TLA+ model or in `SKILL.md`. No check finds it. Gap 4 names both places.

## Answer

Status: all four gaps closed. Nothing is committed. The ticket is not set to resolved.

### What I did

- **Gap 1 (test 3 of the trial log).** Scenario 41.26 and its test. Run A waits between two Jobs. A stop comes. Run B starts and waits at a second `pause.hold(game)` before its first accept (checked with `wait_paused`). Then run A is released. Run A starts no Job: `wait_for_jobs_running_at_most(&state, 1, ..)` passes, `narrator_calls` stays at 1, and the story keeps only the first Input. The status display answers `Autoplay: cycle 1/1` (run B; run A had `1/2`). Then run B is released and plays its budget.
- **Gap 2 (sketch G1).** Scenario 41.27 and its test. Sketch G1 of the ticket 03 asset, with the names of this ticket. An Action after a stop keeps the Game when the stopped run comes to its accept.
- **Gap 3 (sketch G3).** Scenario 41.28 and its test. A run with `cycles=1` on a Game with an old failure line plays its budget, and the run's accept clears the line. I checked the line with `status-error-popover` in the status body, as 41.23 does. The intro of `docs/specs/autoplay.md` now says "A Job of the run that ends with a failure line."
- **Gap 4 (doc 1).** The new sentence of decision 8 is in the intro of `docs/specs/autoplay.md` and in `CONTEXT.md:88`: "The run holds its Game from its start until it ends, also between two Jobs. When the budget or an unreadable record ends the run, its last accept lets go of the Game." I updated the quote of `RunHoldsItsGame` in `VerifyGate.tla` (about line 627) and in `SKILL.md` (line 110). The line numbers differ from the ticket because ticket 10 moved them. `game_flow.md` is untouched.

### Files

- `docs/specs/autoplay.md` (intro, reasons list, scenarios 41.26 to 41.28)
- `CONTEXT.md`
- `tests/http/autoplay/run.rs` (three tests; I also fixed the scenario range in its doc header)
- `.agents/skills/verify-gate/model/VerifyGate.tla` (comments only: the quote, plus 41.26 and 41.27 added as sources of `RunHoldsItsGame` and `StoppedRunStartsNoJob`)
- `.agents/skills/verify-gate/SKILL.md` (the same two rule rows)

No file in `src/` changed. I did not touch `.agents/skills/statechart/`, the map, the trial log or the other tickets.

### Checks

- **Known:** `check.py` passes (all 7 configs hold). `check.py --mutants` passes (all mutants caught).
- **Known:** `python build.py` passes (log `logs/build_20261108_225242.log`): guardrails 309, integration 1656 passed with 3 skipped, browser 44. The first run failed on Vale (the 41.27 heading had more than 25 words). I shortened the heading, and the second run passed.
- **Known:** The verify-gate staleness warning did not show in the full run.
- **Known (scratch mutants, restored; `diff` shows `gate.rs` is equal to the copy I saved):**
  - `accept-without-run-check` (the `is_autoplay_run` filter removed in `accept_for_autoplay`): the 41.26 test fails. The 41.27 test passes, as expected.
  - `run-end-without-run-check` (`end_autoplay` removes the holder with no run check): the 41.27 test fails. The 41.26 test also fails.
  - The 41.28 test has no mutant, as the ticket says.

### Left open

- **Inferred:** `tests/AGENTS.md` does not show the new header range of `run.rs`. The pre-commit hook regenerates it.
- **Inferred:** the old quote "from its start to its end" still stands in `game_flow.md`, in the ticket 03 asset and in the ticket 07 asset. The ticket says `game_flow.md` waits for ticket 15. I left the assets as history.
- **Guessed:** the verify-gate maintain pass needs no `record`, because no covered function changed.

### Coordinator check (2026-11-08)

- `scripts/verify_coverage.py check`: every listed function matches its hash (known). `gate.rs` holds only the doc comment of ticket 10, so the scratch mutants are restored.
- The full gate passes (`logs/build_20261108_225242.log`: integration 1656 passed and 3 skipped, browser 44).
