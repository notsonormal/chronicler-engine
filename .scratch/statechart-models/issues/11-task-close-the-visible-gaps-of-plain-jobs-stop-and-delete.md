# Close the gaps of run 1 that a client can see: plain Jobs, stop, delete and the config write

Type: task
Status: open
Blocked by: —

## Question

Add a spec scenario and a tier-1 test for each gap below, and let the TLA+ model reach the writes on an idle Game. Ticket 09 (Q5) decided the list: a gap gets a scenario and a test only when a client can see a different result.

## The gaps (numbers as in [the trial log](../assets/trial-log.md))

1. **Spec 1.** A plain Job whose accept cannot read the story writes the failure line and makes no holder (`TheAcceptCannotRead`). Only the Autoplay form has a scenario (autoplay.md 41.23).
2. **Spec 4 and test 2.** A stop on an idle Game that shows a failure line clears the line. Only `game_flow.md` says it today. The test also forces the mutant `stop-keeps-failure-line`.
3. **Spec 5.** A Narration save, with no Options run after it, clears the Options failure in the dock (`TheTextIsSaved`, `empties_the_dock`). Maybe one line in options.md 24.16.
4. **Spec 6.** A delete of a Game that is not active, while a plain Job holds it. autoplay.md 41.16 covers only a run.
5. **Spec 8.** A config write during a plain Job answers `200`. actions.md 6.4 makes the write happen, but no line checks its answer.
6. **Model 1.** In every config, TLC tries `YouEditAMessage`, `YouSwitchOrDeleteASwipe` and `YouChangeTheConfig` only on a busy Game, where they are refused. Add a config, or change one, so that they reach an idle Game. One effect to check: `clear_failure_and_put_dock` clears the failure line and the Options failure.

## Context

- **Alignment.** Where a rule of the TLA+ model covers a new scenario, the rule quotes it. The five comparisons of ticket 04 then stay true.
- **Ticket 15 needs these scenarios.** The rewrite of `game_flow.md` drops sentences such as "A stop removes the holder, clears the failure line and clears the Options failure", because a spec scenario holds them.
- Do not run this ticket in parallel with ticket 10 or ticket 12. All three edit the gate model.

## Checks

- `python .agents/skills/verify-gate/check.py` passes, the mutants included.
- `python build.py` passes. `validate_feature_spec.py` pairs each new scenario with its test.
- Do not commit.
