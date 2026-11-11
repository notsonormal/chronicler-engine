# Close the gaps of run 1 that a client can see: plain Jobs, stop, delete and the config write

Type: task
Status: resolved
Blocked by: 10 (the delete test of item 4 depends on the new `GameCatalogue::delete_game`)

## Question

Add a spec scenario and a tier-1 test for each gap below, and let the TLA+ model reach the writes on an idle Game. Ticket 09 (Q5) decided the list: a gap gets a scenario and a test only when a client can see a different result.

## The gaps (numbers as in [the trial log](../assets/trial-log.md))

1. **Spec 1.** A plain Job whose accept cannot read the story writes the failure line and makes no holder (`TheAcceptCannotRead`). The plain form of this branch already has two scenarios: actions.md 2.6 (the Persona) and failure_display.md 38.15 (the start state). Only the story read itself (`StorageCall::StoriesTail`) has no plain scenario. **Dropped** (plan review, owner, decision 5, Option A): the client sees the same kind of result as in 2.6 and 38.15, and `tests/STRATEGY.md` gives one behaviour one test. No scenario and no test.
2. **Spec 4 and test 2.** A stop on an idle Game that shows a failure line clears the line. Only `game_flow.md` says it today. Scenario: dashboard.md 39.17. Test: `tests/http/dashboard.rs`. A Job fails (`ScriptedModel::failed(Role::Narrator)`), then `POST /status/reset-busy`, then the status display answers Ready. The test forces the code form of `stop-keeps-failure-line`. No rule of the TLA+ model checks an idle stop today. So extend `StopLeavesIdle` to also check `failLine'[g] = 0 /\ optFail'[g] = 0`, and make it quote 39.17 (plan review, owner, decision 7, Option A). The TLA+ mutant `stop-keeps-failure-line` then breaks on the idle path too.
3. **Spec 5.** A Narration save, with no Options run after it, clears the Options failure in the dock (`TheTextIsSaved`, `empties_the_dock`). Scenario: a second Given block in options.md 24.16 that starts from the Options failure of 24.5. Test: `tests/http/options.rs`, with the script of `test_failed_options_run_keeps_prior_set_http` and then an Action. After the Action, the dock shows no Options failure.
4. **Spec 6.** A delete of a Game that is not active, while a plain Job holds it. autoplay.md 41.16 covers only a run. Scenario: games.md 19.5. Test: `tests/http/games_delete.rs`, with the steps of `test_a_delete_of_the_game_ends_its_run` and `post_action` in place of the run. The client sees three things: the delete answers 200 while the Job holds the Game, the Games list no longer names the Game, and the active Game stays idle. Do this after ticket 10, which changes `GameCatalogue::delete_game`.
5. **Spec 8.** A config write during a plain Job answers `200`. actions.md 6.4 makes the write happen, but no spec line checks its answer. `races.rs` (`test_the_event_narration_call_uses_the_preset_active_when_it_runs`) already asserts `is_success()`. Add the line to 6.4 and change that assertion to `StatusCode::OK`. Do not add a new test.
6. **Model 1.** In every config, TLC tries `YouEditAMessage`, `YouSwitchOrDeleteASwipe` and `YouChangeTheConfig` with no new state. Their guards already allow them only on an idle Game (`VerifyGate.tla:242-270`). The likely cause (inferred): the edit and the config write change no variable, and on an idle Game the swipe action gives the same next state as `YouPressStop`. So a config alone does not close the gap. The decision (plan review, owner, decision 15, Option B): the edit and the config write stay steps that change nothing, because the TLA+ model leaves out the story content. Add one rule for the swipe action on an idle Game: it clears the failure line and the Options failure (`clear_failure_and_put_dock`, `gate.rs:731-734`). The rule quotes the new `swipe_switch.md` scenario of item 7. A mutant that keeps the failure line in the swipe action breaks the rule once. Write in `SKILL.md`, Scope, why the edit and the config write change no variable.
7. **A swipe switch or a delete-last clears the failure line and the Options failure** (`gate.rs:553`, `:590`; `game_flow.md:99`). Found by the plan review (known). A client can see it, and no scenario covers it: 36.2 covers only the offered set. Add a scenario in `docs/specs/swipe_switch.md` and a test in `tests/http/swipe_switch.rs` (plan review, owner, decision 6, Option A). Ticket 15 removes the `game_flow.md` line that states it.

## Context

- **Alignment.** Where a rule of the TLA+ model covers a new scenario, the rule quotes it. The five comparisons of ticket 04 then stay true.
- **Ticket 15 needs these scenarios.** The rewrite of `game_flow.md` drops sentences such as "A stop removes the holder, clears the failure line and clears the Options failure", because a spec scenario holds them.
- Do not run this ticket in parallel with ticket 10 or ticket 12. All three edit the gate model.

## Checks

- `python3 .agents/skills/verify-gate/check.py` and `python3 .agents/skills/verify-gate/check.py --mutants` pass. A mutant whose `old` text the edit touches still matches once, and its `breaks` list names each new rule.
- The rule, config and scope tables of `SKILL.md` show each change.
- `python build.py` passes. `validate_feature_spec.py` pairs each new scenario with its test.
- Do not commit.

## Not in scope

- Spec 2, Spec 3, Test 1 and Test 4 (ticket 09 left them as they are).
- The delete race (ticket 10).
- A new seam for `GameCatalogue`.
- A trial-log row. No code changes, so this is not a maintain pass.

## What already exists

- `ScriptedModel::{failed, hold, options, turn}` (`src/test_support/scripted_model.rs`).
- `fetch_status_label`, `fetch_busy_status`, `fetch_part`, `post_action`, `wait_settled` (`tests/http/support/http_requests.rs`).
- `create_game_from` (`tests/http/support/http_fixtures.rs:45`).
- The steps of 41.16 (`tests/http/autoplay/run.rs:904`) and 24.5 (`tests/http/options.rs:226`).
- The assertion in `tests/http/races.rs:545`.
- The ticket 03 sketches, section 6 of [the ticket 03 asset](../assets/03-gate-spec-only-model.md).

## Failure modes

- A TLA+ model edit near `YouPressStop` changes the `old` text of `stop-keeps-failure-line` or `stop-keeps-options-failure`. Then `check.py` stops with "old text is not in the model once". The Checks cover it.
- A new rule or config breaks, or grows past the time limit. Read it as `SKILL.md`, *Read a break*, says.
- Ticket 10 changes the delete, so the 19.5 test answers differently. Handled by the order (blocked by 10).
- A rule quote goes stale. No check finds it. The Checks ask for a read of the `SKILL.md` tables.

## Answer

I closed items 2 to 7. Item 1 stays dropped, as the owner decided. I did not change `src/`. I did not commit. I did not set the status to resolved.

### Specs and tests (known)

| Item | Scenario | Test |
|---|---|---|
| 2 | `dashboard.md` 39.17: a stop of an idle Game clears its failure line | `tests/http/dashboard.rs`, `test_a_stop_of_an_idle_game_clears_its_failure_line` |
| 3 | `options.md` 24.27 (first written as a second Given block of 24.16): a Narration save clears the Options failure of 24.5 | `tests/http/options.rs`, `test_narration_turn_clears_the_options_failure_http` |
| 4 | `games.md` 19.5: a delete of a saved Game while a Job holds it | `tests/http/games_delete.rs`, `test_a_delete_while_a_job_holds_the_game_answers_success_and_ends_its_writes` |
| 5 | `actions.md` 6.4: new line "Then the response status is "200 OK"" after the preset change | No new test. `tests/http/races.rs` now asserts `StatusCode::OK` in place of `is_success()` |
| 7 | `swipe_switch.md` 36.7: a swipe switch clears the failure line and the Options failure | `tests/http/swipe_switch.rs`, `test_switch_swipe_clears_the_failure_line_and_the_options_failure_http` |

- Test 19.5 follows the steps of 41.16: Game A runs a plain Job held at its narrator call, `create_game_from` makes Game B active, then the delete of A. It checks the answer 200, the Games list without A, the status Ready, and the status Ready after the Job ends.
- Test 36.7 gets its two failures like this: a Retry makes two Swipes, a failed Options run puts the Options failure in the dock, and a failed Retry (`ScriptedModel::failed(Role::Narrator)`) writes the failure line. Both stay after the failed Retry.
- 36.7 and the model rule cover a swipe switch only. A delete-last (`gate.rs`, same `clear_failure_and_put_dock`) has no test. The ticket asked for the swipe switch only (inferred).

### Mutant check of each test (known)

I applied each code form to `src/application/generation/gate.rs` as a scratch edit and ran the narrow test. Each test failed. I restored the file from a saved copy, and `diff` shows it is the same as before my work.

| Test | Code form | Result |
|---|---|---|
| 39.17 | `GameWriter::stop` writes the dock but does not clear the failure line | FAIL |
| 24.27 | `write_new_message` does not put the empty dock | FAIL |
| 19.5 | `GameWriter::delete` refuses while a holder exists | FAIL |
| 36.7 | `switch_swipe` does not clear the failure line | FAIL |
| 36.7 | `switch_swipe` does not clear the Options failure (the dock) | FAIL |

The 19.5 mutant is a refusal of a busy delete. The client cannot see a leaked holder after the delete, so a "delete does not remove the holder" mutant would pass the test (inferred). The ticket 10 model covers that order.

### TLA+ model changes (`.agents/skills/verify-gate/`)

- `StopLeavesIdle` now also checks `failLine'[g] = 0 /\ optFail'[g] = 0`. Its comment quotes 39.17 and 24.5. The old rules `StopNoFailureLine` and `StopClearsOptionsFailure` read only the Jobs that a stop cut, so they never checked an idle stop.
- New rule `SwipeClearsFailures`: the swipe action clears the failure line and the Options failure. It quotes 36.7. It is in the `PROPERTY` list of all 7 configs.
- New mutants: `idle-stop-keeps-failure-line` (breaks only `StopLeavesIdle`, so the new check is not redundant), `swipe-switch-keeps-failure-line` and `swipe-switch-keeps-options-failure`. `StopLeavesIdle` is added to the `breaks` of `stop-keeps-failure-line` and `stop-keeps-options-failure`.
- Comments of `YouEditAMessage` and `YouChangeTheConfig` say they change no variable.
- `SKILL.md`: the rule table (`StopLeavesIdle` row and the new `SwipeClearsFailures` row), the `YouPressStop` row, the "When to use" list (36.7), and a Scope bullet that says why the edit and the config write change no variable (model 1, option B).
- `coverage.toml` is unchanged. No covered function changed, so I did not run `verify_coverage.py record`.

### Checks (known)

- `check.py`: all 7 configs hold, exit 0. State counts did not change.
- `check.py --mutants`: 24 mutants, all caught, exit 0.
- `python build.py` (full gate): passed. Spec-coverage guardrail passed, so each new scenario has its test. Integration: 1660 passed (+4). Browser: 44 passed. The run printed no verify-gate warning.

### Left open

- Delete-last has no scenario or test of its own (see above).
- `game_flow.md` still holds the sentences that these scenarios now state. Ticket 15 removes them.
- The generated `tests/AGENTS.md` index is not updated. The pre-commit hook does it (inferred).

### Coordinator check (2026-11-08)

- `scripts/verify_coverage.py check`: every listed function matches its hash (known). `gate.rs` holds only the doc comment of ticket 10, so the scratch code forms are restored.
- The full gate passes (`logs/build_20261108_230542.log`: integration 1660 passed and 3 skipped, browser 44).
- Accepted as left open: a delete-last has no scenario of its own. It runs the same `clear_failure_and_put_dock` as the swipe switch of 36.7.
