# The break paths of the model against the tier-1 tests

Labels: **known** = I read the test body and the code the mutant breaks, and traced the break
through the test's steps. **inferred** = I reached it from the test body, the gate code and the
storage or template it reaches. Read-only work: no cargo, no `build.py`. Date: 2026-11-05.

Question: for each row of `break-paths.md`, would the named code break make a tier-1 test fail?

A test counts when the mutant makes it fail. Where the test reaches the same rule by a different
order of steps than the model's shortest path, the note says which order the test arranges.

| Rule | Mutant (code break) | Verdict | Test | Note |
|---|---|---|---|---|
| OneJob | accept-without-busy-check | forced (known) | tests/http/races.rs::test_two_actions_at_the_same_moment_save_one_input_http; also ::test_action_while_job_runs_is_refused_http | Two clicks go through `accept` together; with no `require_idle` both are accepted, so "exactly one click is refused" fails. The single-click test fails on its 409 assertion. |
| BusyRefusesStory | edit-without-busy-check | forced (known) | tests/http/races.rs::test_edit_and_delete_while_job_runs_are_refused | The held narrator call makes the Game busy; the edit must answer 409, and with no busy check `set_swipe_text` answers 200. |
| BusyRefusesStory | swipe-switch-without-busy-check | forced (known) | tests/http/autoplay/run.rs::test_while_a_run_holds_the_game_every_story_write_is_refused_between_two_jobs (41.11); also tests/http/races.rs::test_swipe_switch_while_job_runs_is_refused (36.3) | 41.11 switches to the other Swipe index while the run holds the Game between two Jobs, so the story would change; the tail check and the bounds check both pass on the last Message. |
| StoppedJobWritesNothing | job-writer-without-id-check | forced (known) | tests/http/races.rs::test_a_stop_at_each_model_call_keeps_the_records_of_that_step (39.4: the `Narration` row for the text save, the `Impersonate` and `InputRetry` rows for the Input save); also ::test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text (39.5) | Each row stops at the held call and then releases it; with the id check gone the busy check of the now-idle Game passes, so the late save lands and the `PlanInput` / `NoMessage` assertions fail. |
| StoppedJobWritesNothing | reset-keeps-the-holder | none (inferred) | — | tests/http/races.rs::test_reset_while_job_runs_keeps_only_the_new_game_opening (7.3) would still pass: the stale holder is keyed by the old Game id, so the Job's late write names the deleted Game, the foreign key refuses it, and every read the test makes is of the new Game. The stale holder is invisible to `wait_idle`, which reads the active Game only. |
| StopNoFailureLine | stop-keeps-failure-line | none (known) | — | No test sets the Game's failure line and then stops. Every `POST /status/reset-busy` call site was read: 39.4, 39.5, 39.8, 39.10, 41.6, 41.7, 41.11, 41.12, 41.17, 24.5, 24.18, 43.12, 43.13 and the route-word tables. None of them holds a failure line before the stop. |
| StopClearsOptionsFailure | stop-keeps-options-failure | forced (known) | tests/http/options.rs::test_failed_options_run_keeps_prior_set_http (24.5) | The test fails an Options run, reads the dock notice, then stops and asserts the prior set is back. The dock template renders the failure notice *instead of* the strip, so a kept failure hides "Search the desk". |
| StopLeavesIdle | stop-keeps-the-run | forced (known) | tests/http/autoplay/run.rs::test_a_stop_during_a_job_of_a_run_ends_the_run (41.6); ::test_a_stop_between_two_jobs_of_a_run_ends_the_run (41.7) | Both read `Ready` and `!is_busy` at once after the stop; a kept run holder keeps the Autoplay status. |
| NextJobKeepsHolder | job-drop-without-id-check | forced (known) | tests/http/races.rs::test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text (39.5) | The test releases the stopped call first, so the stopped Job's `Drop` runs after the second Job accepted. With no id check that drop removes the second Job's holder, and the second Job's text save is refused, so `NEXT_NARRATION` is missing. |
| RunHoldsItsGame | job-drop-removes-the-run | forced (known) | tests/http/autoplay/run.rs::test_a_run_on_a_fresh_game_plays_its_cycles_and_the_game_is_then_idle (41.1); also 41.7 | The model's path ends a failed Job; a run of 2 cycles is enough: the holder must survive the first Job to accept the second, so with the mutant the run ends after one Job and calls 3 and 4 never happen. |
| RunHoldsItsGame | run-end-without-run-check | forced (known) | tests/http/autoplay/run.rs::test_the_end_of_a_stopped_run_does_not_release_the_run_that_started_after_it (41.17) | Run A's late end must leave run B's holder: the test then reads run B's Autoplay status and a 409 for an Action. |
| StoppedRunStartsNoJob | accept-without-run-check | none (inferred) | — | The order needs run 1 waiting at its accept while run 2 holds. 41.17 stops run A inside a Job, so run A ends at `JobEnd::EntryMoved` and never accepts; 41.7 stops run 1 at the pause but no holder is left, so the accept answers `Ended`. No test starts a second run while the first waits at the pause. |
| RunRefusesConfig | config-write-during-a-run | forced (known) | tests/http/autoplay/run.rs::test_while_a_run_holds_the_game_the_config_writes_are_refused_and_the_config_stays (41.12) | All three config routes must answer 409 and the config must read back unchanged; with no check the writes land. |
| RunEndReasons | moved-job-ends-as-failure | none (known) | — | `RunEnd` reaches only `tracing::info!(..., end = ?end, "autoplay run ended")`; no test names `RunEnd`, `JobFailed` or that log line, and the reason changes no write. |
| BudgetBound | budget-checked-too-late | forced (known) | tests/http/autoplay/run.rs::test_a_run_on_a_fresh_game_plays_its_cycles_and_the_game_is_then_idle (41.1) | With `d = Budget` not finished, a budget of 2 accepts a third Job: `calls.len() == 4` fails, the four-Message story fails, and the fifth narrator call is unscripted, which `wait_for_run` reports. |
| RunFailureLineStays | not-accepted-without-failure-line | forced (known) | tests/http/autoplay/run.rs::test_a_run_whose_story_cannot_be_read_ends_with_the_failure_line_of_the_story (41.23); also ::test_a_run_whose_start_state_cannot_be_read_ends_with_the_failure_line_of_that_record | The injected `StoriesTail` failure makes the run's own accept fail; the status must hold the disclosure line. |
| RunEndsIdle | run-end-keeps-the-holder | forced (known) | tests/http/autoplay/run.rs 41.1 (`wait_for_run` asserts `!is_busy`, and `assert_status "Ready"`) | A kept holder leaves the Game busy after every Job of the run. |
| DeleteNotActive | none: breaks on the current code | none (inferred) | — | No test races a delete with a switch. tests/http/games_delete.rs 19.2 deletes the active Game with one request and 19.1 deletes an inactive Game with no switch, and neither test uses two concurrent requests. |

Counts: 13 forced, 5 none. No row is "partly": each forced row fails outright when the mutant is made.

## The delete-vs-switch race (row DeleteNotActive)

- **No test covers it** (known: `games_delete.rs` holds 4 tests, `games_switch.rs` 3, and none of
  them combines the two requests or runs two requests at once).
- **A real route pair reaches the order** (known): `POST /games/:id/delete` and
  `POST /games/:id/switch` (`src/adapters/driving/http/builders/router.rs:82-91`), and both are
  plain POSTs with no revision or header check that would stop the pair.
- **The window is real but not forceable by a test today** (inferred). `GameCatalogue::delete_game`
  reads `ActiveGame::get` and then calls `GameWriter::delete` with no await between them, so only a
  thread preemption on the multi-thread runtime reaches the order. Neither `ActiveGame` nor
  `GameCatalogue` has a test pause seam like `AutoplayRun`'s `pause::point`, so a tier-1 test can
  reach the order only by luck.

## The test gaps

1. **reset-keeps-the-holder**: no test makes the held Job's late write observable after a reset. It
   names the deleted Game, so the foreign key refuses it and the assertion set of 7.3 cannot fail.
   A test needs a read of the removed Game's busy state, or of the `busy` change number.
2. **stop-keeps-failure-line**: no test stops a Game that holds a failure line. Every stop site
   starts from a Game with no failure line, so the clear has no test.
3. **accept-without-run-check**: no test has run 1 at its pause point while a stop and a second run
   happen. The pause seam exists, so this order is forceable with `pause_between_two_jobs` and
   `pause_before_first_job`.
4. **moved-job-ends-as-failure**: the run's end reason is a log line only, so no HTTP test can fail
   on it. A test would need a log assertion, or the reason would need a reader.
5. **DeleteNotActive**: no delete-vs-switch test, and no seam to force the window.
