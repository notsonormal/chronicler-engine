# Trial log of the verify-gate skill

One row for each run of `.agents/skills/verify-gate/` and each maintain pass. The outcome is one of: in sync, updated, bug, blocked. The gap kinds come from the five comparisons of [ticket 07](../issues/07-task-write-the-verify-skill-of-the-gate.md).

| Date | Commit range | Minutes | Outcome | Gaps by kind |
|---|---|---|---|---|
| 2026-11-05 | (none) .. `e6715c60` (first run) | 74 | bug | code 1, model 1, spec 8, test 5, doc 1 |
| 2026-11-08 | `e6715c60` .. uncommitted work (ticket 10, the active pointer fix) | 40 | updated | none new; closes code 1 |

Ticket 10: the mutex of `ActiveGame` is now the outer lock of a switch, a delete, a create and a reset. The TLA+ model has the switch in two steps, a create action, the lock (`pointerBy`) and the rule `ActiveExists`. Every config holds, and `known_breaks` is empty. The fix mutant `delete-check-inside-the-hold` is gone. Four mutants each take the lock away from one request: `switch-`, `delete-`, `reset-` and `create-without-the-pointer-lock`. The reset and the create mutants show orders 3 and 4 of ticket 10. The two stop mutants were written again for the new variable.

The minutes cover the whole first run: the rule list, the TLA+ model, the configs, the mutants, the skill, the proof run and the five comparisons. One later run of the check takes about 35 s, and the mutants take about 25 s.

The detail of the first run is in [`07-verify-gate-first-run/`](07-verify-gate-first-run/):

- `rules-from-spec.md`: the rule list, written from `docs/specs/` before any read of `src/`.
- `break-paths.md`: the shortest break path of each mutant.
- `tests-vs-model.md`: the tier-1 test that forces each path.
- `spec-docs-vs-model.md`: the scenario of each branch, and the sentences of `game_flow.md` against the TLA+ model.
- `verify-gate-model-follows-code.diff`: the model edit of the proof run.

## The gaps of run 1

Each claim is labelled known (TLC or a read showed it) or inferred (a search or a reading found it).

### Code (model and code agree, and the code breaks a rule)

1. **The delete of the active Game** (known: TLC; inferred: harm).
   - Rule: `DeleteNotActive`.
   - Scenario: games.md 19.2, "Deleting the active game returns 400".
   - Path: `GameCatalogue::delete_game` checks Game 2 against `ActiveGame` outside the hold, and the check passes. A second request runs `GameCatalogue::switch_game` to Game 2. `GameWriter::delete` then removes the active Game.
   - Reach: the routes are real, `POST /games/:id/delete` and `POST /games/:id/switch`. No await sits between the check and the delete, so only a thread preemption reaches the order. The harm is low.
   - Fix: the fix mutant `delete-check-inside-the-hold` holds.
   - **Correction (ticket 09, 2026-11-06).** The fix mutant does not close the race. The TLA+ model makes `YouSwitchTheGame` one step, but `GameCatalogue::switch_game` reads the Game and then sets the pointer, outside every hold. With a switch in two steps, TLC finds two more orders that leave the pointer on a deleted Game, also with the fix mutant on: a switch, then a delete; and a reset, then a delete of the new Game before `ActiveGame::set_new`. So this is also a model gap. Details: the Comments of [ticket 09](../issues/09-grilling-decide-the-gaps-and-the-verify-skill.md).

### Model (the TLA+ model does not reach a part of the code)

1. **The story writes and the config write on an idle Game** (known: `-coverage`).
   - Actions: `YouEditAMessage`, `YouSwitchOrDeleteASwipe` and `YouChangeTheConfig`.
   - Gap: in every config, TLC tries these actions only while the Game is busy, where they are refused. So their effect on an idle Game is never checked. One example is the failure line and the Options failure that `clear_failure_and_put_dock` clears.

Two modelling errors were found and fixed before the configs passed. `RunEndsIdle` blamed a run for a holder that a later request had taken. `RunFailureLineStays` read the failure line after a later stop had cleared it.

### Spec (a branch of the model with no covering scenario; inferred from a search of `docs/specs/`)

1. The accept of a plain Job whose story read fails (`TheAcceptCannotRead`). Only the Autoplay version exists: autoplay.md 41.23.
2. A stop after the failure line of the Job landed and before the drop of the `JobWriter` (`YouPressStop`). The table of dashboard.md 39.4 stops only at model calls.
3. A stop after the Options failure landed and before the drop (`YouPressStop`). In options.md 24.5, the stop comes after the Job ended.
4. A stop that clears the failure line of a Job that already ended, on an idle Game (`YouPressStop`). Only `game_flow.md` states this rule.
5. The Narration save that clears the Options failure (`TheTextIsSaved`, `empties_the_dock`). No scenario isolates it from the Options run that follows.
6. A delete of a Game that is not active while a plain Job holds it (`YouDeleteAGame`). autoplay.md 41.16 covers only a run.
7. The window between the hold of `GameWriter::replace` and `ActiveGame::set_new` (`TheResetMovesThePointer`). No scenario acts in it.
8. A config write that passes while a plain Job runs (`YouChangeTheConfig`). actions.md 6.4 makes it happen only as a side effect, and no scenario checks its own answer.

### Test (no tier-1 test forces the break path; from `tests-vs-model.md`)

1. `StoppedJobWritesNothing` with `reset-keeps-the-holder` (inferred). The late write of the Job hits the foreign key of the deleted Game, so reset.md 7.3 still passes. No test reads the holder of the removed Game.
2. `StopNoFailureLine` with `stop-keeps-failure-line` (known). No test stops a Game that shows a failure line.
3. `StoppedRunStartsNoJob` with `accept-without-run-check` (inferred). The seam `pause_between_two_jobs` exists. No test holds run 1 at that pause while a stop and a second run happen.
4. `RunEndReasons` with `moved-job-ends-as-failure` (known). The `RunEnd` reaches only a log line, and no test reads it.
5. `DeleteNotActive` (inferred). No test races a delete with a switch. `ActiveGame` and `GameCatalogue` have no pause seam, so a test cannot force the order today.

The other 13 break paths each have a tier-1 test that forces them.

### Doc (a sentence of `game_flow.md` contradicts the TLA+ model; inferred)

1. **The span of a run's hold.** The sentence: "An Autoplay run. It lasts from the start of the run to its end, also between two Jobs of the run."
   - Model: `TheRunAccepts` removes the holder inside the accept, for the budget end and for the unreadable record. That happens before `TheRunEnds`.
   - Code: `accept_for_autoplay` does the same.
   - So the sentence is loose. In that window, a second run can take the Game before the first run drops. That window is the one the mutant `run-end-without-run-check` uses.

### The skill itself (found by the proof run, fixed in `SKILL.md`)

A fresh agent got a scratch break in `GameWriter::end_autoplay` (it removed the run id check). It caught the break after it updated the model, and judged it a code bug.

Its first `check.py` run, on the old model, was a false green. `SKILL.md` now says that the model must follow the code before a pass means anything. The trigger now uses `git diff <sync_commit>`. `SKILL.md` also now says how to read a `MISSED` mutant while a config breaks, and where a model edit that shows a bug goes.
