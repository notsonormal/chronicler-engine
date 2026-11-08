---
name: verify-gate
description: "Model-check the generation gate: the busy registry, the Job id check, the Autoplay holder, stop, reset, delete and the config write. Use after a change to src/application/generation/gate.rs, src/application/pipeline/job.rs, src/application/autoplay/driver.rs or src/application/games/catalogue.rs, or to a gate sentence of docs/specs/ or docs/diataxis/reference/game_flow.md."
---

# Verify the gate

The gate lets one holder at a time write the records of a Game. A holder is a Job or an Autoplay run. This skill checks a TLA+ model of the gate with TLC, in every order of its steps, up to small bounds.

**The property.** A stop, a reset or a delete ends the work of the Game at once. No later write of that work lands, and no later end of that work releases the work that came after it.

**The harm when it breaks.** A stopped Job writes a Narration that the person threw away, or the late end of an old Job or run frees the Game while the new one runs. Then two writers race on one story.

## When to use it

- A change touches a function that `coverage.toml` lists, or adds a function that reads or writes the busy state of a Game (`BusyRegistry`, `Holder`, `JobEntry`).
- A change alters a gate scenario in `docs/specs/` (`dashboard.md` 39.x, `autoplay.md` 41.x, `actions.md` 3.x, `reset.md` 7.3, `games.md` 18.4 and 19.x, `options.md` 24.5).
- `git diff <sync_commit> -- <the paths in coverage.toml>` is not empty. It shows commits and uncommitted work. Then the model may not describe the code.

## Run it

`check.py` checks the model, never the code. After a change to covered code, a pass proves nothing until you have brought the model in line with the code (*Update the model after a code change*). Do that first, then run the check.

From the repo root:

```bash
python3 .agents/skills/verify-gate/check.py            # every config, about 35 s
python3 .agents/skills/verify-gate/check.py --mutants  # every mutant, about 25 s
```

`check.py` gets TLC and the JDK through `mise`, with the pins of `mise.toml`. Do not run it while a cargo build runs: one TLC run takes up to 2 GiB of memory. `pgrep -a cargo` shows a running build, and `free -g` shows the free memory. The TLC logs go to `tmp/verify-gate/`.

Each config prints one line. The exit code is 0 when every config gives its expected result:

| Line | Meaning |
|---|---|
| `jobs: holds (...)` | Every rule holds in every reachable state of the config. |
| `delete-race: breaks: DeleteNotActive (...)  (known)` | A known finding (see *Known findings*). It still breaks, as expected. |
| `<- not known: RULE` | A new break. Read it as the next section says. |
| `<- known, but holds now: RULE` | A known finding went away. Find the change that fixed it, and take it out of `known_breaks` in `coverage.toml`. |
| `error` | TLC did not finish: a parse error, a deadlock or an exception. The log in `tmp/verify-gate/` names it. |

Each mutant prints `caught`, or `holds` for a fix mutant. Read the mutants only when every config gives its expected result. While a config breaks, a mutant on that config can show `MISSED` because the same break comes first. When the configs pass, `MISSED` means that a rule cannot fail any more. Then the model or the rule is wrong.

## Read a break

Run the model and the config through the `/statechart` skill (`.agents/skills/statechart/SKILL.md`, section *Run*), with `model/VerifyGate.view.mjs` as the view. Write its output under `tmp/`. It tells the shortest path to the break as plain steps. Then replay each step against the real code:

- **A step cannot happen in the code.** The model is wrong. For example, the model splits a step that the code does in one `BusyRegistry::hold`, or it allows a request that a route refuses. Fix the model and say what was wrong.
- **Every step can happen.** It is a bug in the code. Do not weaken the rule or the model until it passes, and do not change product code unless the person asks. Keep the model edit that shows the bug out of the skill folder: save it as a patch under `tmp/`, restore the model, and give the patch with the report. The model moves to the new code only when the person accepts the code change. Report the steps in the words of the code, the harm in one sentence, and the commit that brought it. A candidate tier-1 test is worth it only where a real route can force that order of steps.

When a config holds, `/statechart` draws a chart only when the config has 500 states or fewer. The `runs` config is near that size.

## Update the model after a code change

Write the update from the code, before you run the check. Do not shape it by what makes the check pass.

1. Read `git diff <sync_commit>.. -- <the paths in coverage.toml>` and each commit message. Look also for covered functions that moved, and for new functions that read or write `Holder`.
2. For each change, find its action in the table below. One action is one `BusyRegistry::hold` of the code, or one step outside every hold (a model call, a read). Exceptions: where no model call sits between two holds of one Job, one action covers both. The comment of the action names them.
3. Change a rule only when the spec promise changed on purpose. Quote the new spec sentence in the comment of the rule, and keep the old wording in the trial log.
4. Run `check.py`. A new break after the update is either a bug in the new code or an error of the update: judge it as *Read a break* says. You cannot tell an intended change from a regression before that judgement.
5. Run `check.py --mutants`. A new action or a new check of the code needs a mutant in `model/mutants.toml` that removes it and breaks a rule. When the update changes the text that a mutant replaces, write the mutant again, so that it still removes exactly one check of the new code.
6. Update `coverage.toml`: the functions, and `sync_commit` and `sync_date`. Move `sync_commit` only after you confirmed that the model describes every listed function at that commit.

For the upkeep after changes that skipped this skill (merges, many commits), use the `/maintain-formal-verification-skill` skill.

## The model

`model/VerifyGate.tla` holds the actions and the rules. Each `.cfg` file is one bounded scope of it.

| Action | Code |
|---|---|
| `YouStartAJob` | `GameWriter::accept` (from `Job::start`): busy check, story read, plan, `accept_in_tx`, job entry |
| `TheAcceptCannotRead` | `GameWriter::accept`, the branch of `not_accepted_with_failure` |
| `YouStartAutoplay` | `GameWriter::reserve_for_autoplay` (from `AutoplayRun::start`) |
| `YouPressStop` | `GameWriter::stop`: the dock read, the holder removal, then one transaction that clears the failure line and the Options failure |
| `YouResetTheGame`, `TheResetMovesThePointer` | `GameCatalogue::reset`: the hold of `GameWriter::replace`, then `ActiveGame::set_new` |
| `YouDeleteAGame`, `TheDeleteRemovesTheGame` | `GameCatalogue::delete_game`: the check against `ActiveGame` outside the hold, then `GameWriter::delete` |
| `YouSwitchTheGame` | `GameCatalogue::switch_game` |
| `YouEditAMessage` | `GameWriter::edit` |
| `YouSwitchOrDeleteASwipe` | `GameWriter::switch_swipe`, `GameWriter::delete_last`, with `clear_failure_and_put_dock` |
| `YouChangeTheConfig` | `GameWriter::save_config` (from `GameCatalogue::save_config`) |
| `AJobStartsAGeneration` | `Job::run_step` and `JobWriter::step` before the narrator call; for the first generation of an Action, also `JobWriter::save_input` |
| `TheNarratorAnswers`, `TheNarratorFails` | the narrator call of `Job::generate`, outside every hold |
| `TheTextIsSaved` | `JobWriter::save_text` (`write_new_message`, `write_new_swipe`) |
| `TheAnalysisStarts`, `TheAnalysisAnswers` | `run_step(Step::Quantifying)`, then the Quantifier call |
| `TheStateIsSaved` | `Job::update_state` and `JobWriter::save_state_update`; a fired Trigger orders the event generation |
| `TheOptionsRunFollows`, `TheJobSkipsTheOptionsRun` | `OptionsRun::follows` in `Job::run` |
| `TheOptionsRunStarts`, `TheOptionsCallEnds`, `TheOptionsAreSaved` | `Job::options_step`, the Options call, `JobWriter::save_options` |
| `TheFailureLineIsSaved` | `Job::end` and `JobWriter::save_failure` |
| `TheJobEnds` | `Drop for JobWriter` |
| `TheRunAccepts` | `GameWriter::accept_for_autoplay` with `AutoplayRun::decide` and `plan_and_start` |
| `TheRunSeesItsJobEnd` | the `JobEnd` that `Job::run_accepted` gives to `AutoplayRun::run_jobs` |
| `TheRunEnds` | `Drop for AutoplayRun` and `GameWriter::end_autoplay` |

Every write of a Job goes through `Own(j)`, the job id check of `JobWriter::hold`. The ghost variables (`cut`, `late`, `storyBusy` and the others) record what the rules read. No code reads them.

| Rule | Spec sentence | Mutant that breaks it |
|---|---|---|
| `OneJob` | actions.md 3.5: "one Action is accepted and the other is refused with 409" | `accept-without-busy-check` |
| `BusyRefusesStory` | screen.md 45.17: "Then each response status is 409" | `edit-without-busy-check` |
| `StopLeavesIdle` | dashboard.md 39.4: "Then the stop answers "reset" And the Game is idle" | `stop-keeps-the-run` |
| `StopNoFailureLine` | dashboard.md 39.4: "And the status display answers with no failure line" | `stop-keeps-failure-line` |
| `StopClearsOptionsFailure` | options.md 24.5: "When the client POST /status/reset-busy Then the dock renders set A" | `stop-keeps-options-failure` |
| `StoppedJobWritesNothing` | dashboard.md 39.5: "the story log holds no Narration of the stopped Job"; reset.md 7.3 | `job-writer-without-id-check`, `reset-keeps-the-holder` |
| `NextJobKeepsHolder` | dashboard.md 39.5: "the story log holds the Input and the Narration of the second Job" | `job-drop-without-id-check` |
| `RunHoldsItsGame` | autoplay.md: "The run holds its Game from its start to its end, also between two Jobs."; 41.17 | `job-drop-removes-the-run`, `run-end-without-run-check` |
| `StoppedRunStartsNoJob` | autoplay.md 41.6: "And no further Job of the run starts" | `accept-without-run-check` |
| `RunRefusesConfig` | autoplay.md 41.12: "the config writes are refused and the config stays" | `config-write-during-a-run` |
| `RunEndReasons` | autoplay.md: "A run ends for one of these reasons: ..." | `moved-job-ends-as-failure` |
| `BudgetBound` | autoplay.md: "The **budget** is the number of cycles" | `budget-checked-too-late` |
| `RunFailureLineStays` | autoplay.md 41.4 and 41.23: "the status display answers with the failure line" | `not-accepted-without-failure-line` |
| `RunEndsIdle` | autoplay.md 41.1: "And the Game is idle" | `run-end-keeps-the-holder` |
| `DeleteNotActive` | games.md 19.2: "Deleting the active game returns 400" | breaks today (fix mutant: `delete-check-inside-the-hold`) |

TLC stops at the first break, so each mutant names the rules that can break first. A rule that breaks only behind another rule uses `without` in `mutants.toml`.

## Scope

| Config | Scope |
|---|---|
| `jobs` | 1 Game, 2 plain Jobs (Action or Options), stop, reset, the story writes, the config write |
| `runs` | 1 Game, 1 run with a budget of 2 cycles, stop, the story writes, the config write |
| `two-runs` | 1 Game, 2 runs with a budget of 1 cycle, stop |
| `run-and-jobs` | 1 Game, 1 run and plain Jobs, 2 Jobs in all, stop, the story writes, the config write |
| `games` | 2 Games, 1 plain Job, reset, delete, a switch of the active Game |
| `games-run` | 2 Games, 1 run, reset, delete, a switch of the active Game |
| `delete-race` | 2 Games and no Job: delete, switch, reset |

A larger bound costs much more: `run-and-jobs` with 3 Jobs has 3.1 million states and takes 2.5 minutes.

The model leaves out:

- Storage faults. A refused write of the failure line leaves the run ended with no failure line. That is a storage fault, not a gate order.
- The stop whose dock read fails (dashboard.md 39.8). `GameWriter::stop` reads the dock before it removes the holder, so the failed branch changes nothing. No rule checks that order.
- That a Job writes only to its own Game (games.md 18.4). Each Job keeps its Game id from its accept, so the model holds this by construction.
- That a Job always ends (actions.md I.1). TLC checks that no state is stuck, but no rule checks that each Job reaches its end.
- The story content. The model keeps only the type of the last Message, which is what `AutoplayRun::decide` and the plan read. So no rule counts the writes of one Swipe state.
- A restart of the server. The busy registry lives in memory only, so a restart loses every holder.

The model has the branch of `accept_for_autoplay` that answers `NotAccepted::StillThinking`, where the run's own Job is in flight. TLC never reaches it, as the code comment says.

## Known findings

`delete-race` breaks `DeleteNotActive`: `GameCatalogue::delete_game` checks the active Game outside the hold of `GameWriter::delete`. The steps: you delete Game 2 while Game 1 is active, and the check passes. Before the delete takes its hold, a second request switches the active Game to Game 2. The delete then removes the active Game. The fix mutant `delete-check-inside-the-hold` closes this order only.

The model has a gap here. `YouSwitchTheGame` is one step, but `GameCatalogue::switch_game` reads the Game and then calls `ActiveGame::set`, outside every hold, and `ActiveGame` has its own mutex. With a switch in two steps and the rule `ActiveExists`, TLC finds two more orders that leave the pointer on a deleted Game: a switch, then a delete; and a reset, then a delete of the new Game before `ActiveGame::set_new`. The fix mutant does not close them. The ticket *Fix the active pointer that can name a deleted Game* (`.scratch/statechart-models/issues/10-*.md`) fixes the model and the code.
