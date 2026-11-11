---
name: verify-gate
description: "Model-check the generation gate and generate its diagrams in docs/diataxis/reference/diagrams/. It covers the busy registry, the Job id check, the Autoplay holder, stop, reset, delete, the active pointer and the config write. Use after a change to src/application/generation/gate.rs, src/application/pipeline/job.rs, src/application/autoplay/driver.rs, src/application/games/catalogue.rs or src/application/games/active.rs, or to a gate sentence of docs/specs/ or docs/diataxis/reference/game_flow.md."
---

# Verify the gate

The gate lets one holder at a time write the records of a Game. A holder is a Job or an Autoplay run. This skill checks a TLA+ model of the gate with TLC, in every order of its steps, up to small bounds.

**The property.** A stop, a reset or a delete ends the work of the Game at once. No later write of that work lands, and no later end of that work releases the work that came after it.

**The harm when it breaks.** A stopped Job writes a Narration that the person threw away, or the late end of an old Job or run frees the Game while the new one runs. Then two writers race on one story.

## When to use it

- A change touches a function that `coverage.toml` lists, or adds a function that reads or writes the busy state of a Game (`BusyRegistry`, `Holder`, `JobEntry`).
- A change alters a gate scenario in `docs/specs/` (`dashboard.md` 39.x, `autoplay.md` 41.x, `actions.md` 3.x, `reset.md` 7.3, `games.md` 18.4 and 19.x, `options.md` 24.5, `swipe_switch.md` 36.7).
- The end of a `build.py` run shows `WARNING verify-gate: the TLA+ model may not match the code`. It names each listed function that differs from its hash in `coverage.toml`, has no hash, or is missing. The warning reads the files on disk, so uncommitted work counts. `python scripts/verify_coverage.py check` prints it alone. The agent that changed a listed function runs this skill, or `/maintain-formal-verification-skill`, before it ends its task.
- The end of a `build.py` run shows `WARNING verify-gate: N of M diagrams are older than their inputs`, or `the skill has no generated diagram`. Run `check.py`: it checks every config, then writes the diagrams (*The diagrams*). The warning `diagrams.toml cannot be read` names the error in the manifest: fix the manifest first.

## Run it

`check.py` checks the model, never the code. After a change to covered code, a pass proves nothing until you have brought the model in line with the code (*Update the model after a code change*). Do that first, then run the check.

From the repo root:

```bash
python3 .agents/skills/verify-gate/check.py             # every config (about 35 s), then the diagrams when an input changed (about 90 s)
python3 .agents/skills/verify-gate/check.py --mutants   # every mutant, about 25 s
python3 .agents/skills/verify-gate/check.py --diagrams  # only the diagrams, with no config check (about 90 s)
```

`check.py` gets TLC and the JDK through `mise`, with the pins of `mise.toml`. Do not run it while a cargo build runs: one TLC run takes up to 2 GiB of memory. `pgrep -a cargo` shows a running build, and `free -g` shows the free memory. The TLC logs go to `tmp/verify-gate/`.

Each config prints one line, and a new break adds the line of its path. The exit code is 0 when every config gives its expected result:

| Line | Meaning |
|---|---|
| `jobs: holds (...)` | Every rule holds in every reachable state of the config. |
| `<config>: breaks: RULE (...)  (known)` | A known finding (see *Known findings*). It still breaks, as expected. |
| `<- not known: RULE` | A new break. Read it as the next section says. |
| `<- known, but holds now: RULE` | A known finding went away. Find the change that fixed it, and take it out of `known_breaks` in `coverage.toml`. |
| `error` | TLC did not finish: a parse error, a deadlock or an exception. The log in `tmp/verify-gate/` names it. |
| `  path to the break: tmp/verify-gate/paths/<config>/sequence-RULE.mmd` | The sequence diagram of the shortest path to a new break. |
| `  path to the break, as steps: tmp/verify-gate/paths/<config>/report.md` | A PROPERTY broke. `/statechart` draws a sequence diagram for an INVARIANT only, so `report.md` tells the path as steps. |
| `  no path to the break: see ...` | `/statechart` could not draw the path. The log that the line names gives the cause. Read the break from `tmp/verify-gate/<config>.out`. |
| `diagrams: current` or `diagrams: wrote N pages` | The last line of a run of every config. See *The diagrams*. |
| `diagrams: not written ...` | No page changed. After a config that breaks, this is expected. Otherwise, do step 6 of *Update the model after a code change*. |

Each mutant prints `caught`, or `holds` for a fix mutant. Read the mutants only when every config gives its expected result. While a config breaks, a mutant on that config can show `MISSED` because the same break comes first. When the configs pass, `MISSED` means that a rule cannot fail any more. Then the model or the rule is wrong.

## Read a break

For each new break, `check.py` runs the config through the `/statechart` skill with `model/VerifyGate.view.mjs` as the view, into `tmp/verify-gate/paths/<config>/`. `sequence-RULE.mmd` draws the shortest path to the break: each request, each hold of `GameWriter` and each model call, with the note `rule RULE breaks here` at the end. `report.md` tells the same path as plain steps. The path diagram goes with the report: into the trial log or a bug ticket, never into `docs/`. Then replay each step against the real code:

- **A step cannot happen in the code.** The model is wrong. For example, the model splits a step that the code does in one `BusyRegistry::hold`, or it allows a request that a route refuses. Fix the model and say what was wrong.
- **Every step can happen.** It is a bug in the code. Do not weaken the rule or the model until it passes, and do not change product code unless the person asks. Keep the model edit that shows the bug out of the skill folder: save it as a patch under `tmp/`, restore the model, and give the patch with the report. The model moves to the new code only when the person accepts the code change. Report the steps in the words of the code, the harm in one sentence, and the commit that brought it. A candidate tier-1 test is worth it only where a real route can force that order of steps.

`VerifyGate.view.mjs` merges TLC states into chart states. `/statechart` draws a config when TLC reaches at most 200,000 states and the view gives 500 chart states or fewer. Above that, it draws only the path to the break.

## Update the model after a code change

Write the update from the code, before you run the check. Do not shape it by what makes the check pass.

1. Read `git diff <sync_commit>.. -- <the paths in coverage.toml>` and each commit message. Look also for covered functions that moved, and for new functions that read or write `Holder`.
2. For each change, find its action in the table below. One action is one `BusyRegistry::hold` of the code, or one step outside every hold (a model call, a read). Exceptions: where no model call sits between two holds of one Job, one action covers both. The comment of the action names them.
3. Change a rule only when the spec promise changed on purpose. Quote the new spec sentence in the comment of the rule, and keep the old wording in the trial log.
4. Run `check.py`. A new break after the update is either a bug in the new code or an error of the update: judge it as *Read a break* says. You cannot tell an intended change from a regression before that judgement. A `diagrams: not written` line belongs to step 6.
5. Run `check.py --mutants`. A new action or a new check of the code needs a mutant in `model/mutants.toml` that removes it and breaks a rule. When the update changes the text that a mutant replaces, write the mutant again, so that it still removes exactly one check of the new code.
6. Update the diagrams. A new action needs its words in `diagrams/sequence.view.mjs` (`message()`) and in each chart view whose axis it moves (`step()`, and `state()` when the action adds a value that the view reads). Run `check.py` again: it writes the diagrams when the configs pass. A scenario of `diagrams/scenarios.mjs` that the new model does not allow stops the diagrams. The log that the `diagrams: not written` line names gives the step. Write the scenario again from the new code.
7. Update `coverage.toml`: add, rename or remove the listed functions. Then run `python scripts/verify_coverage.py record`. It writes the hash of each listed function as it is on disk, so record only after steps 4 and 5 pass and the model describes every listed function. Then `build.py` shows no warning. Move `sync_commit` and `sync_date` only to a commit that holds the change. The warning does not read them: `/maintain-formal-verification-skill` reads `git diff <sync_commit>` to see the old code.

For the upkeep after changes that skipped this skill (merges, many commits), use the `/maintain-formal-verification-skill` skill.

## The model

`model/VerifyGate.tla` holds the actions and the rules. Each `.cfg` file is one bounded scope of it.

| Action | Code |
|---|---|
| `YouStartAJob` | `GameWriter::accept` (from `Job::start`): busy check, story read, plan, `accept_in_tx`, job entry |
| `TheAcceptCannotRead` | `GameWriter::accept`, the branch of `not_accepted_with_failure` |
| `YouStartAutoplay` | `GameWriter::reserve_for_autoplay` (from `AutoplayRun::start`) |
| `YouPressStop` | `GameWriter::stop`: the dock read, the holder removal, then one transaction that clears the failure line and the Options failure. It does the same on an idle Game |
| `YouResetTheGame`, `TheResetMovesThePointer` | `GameCatalogue::reset`: `ActiveGame::lock`, the hold of `GameWriter::replace`, then `PointerHold::set_new` |
| `YouCreateAGame`, `TheCreateMovesThePointer` | `GameCatalogue::create_game`: `ActiveGame::lock`, the hold of `GameWriter::create`, then `PointerHold::set_new` |
| `YouDeleteAGame`, `TheDeleteRemovesTheGame` | `GameCatalogue::delete_game`: `ActiveGame::lock` and the check against the pointer, then `GameWriter::delete` |
| `YouSwitchTheGame`, `TheSwitchSetsThePointer` | `GameCatalogue::switch_game`: `ActiveGame::lock` and the Game read, then `PointerHold::set` (the touch and the move) |
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

The mutex of `ActiveGame` is the outer lock of a switch, a delete, a create and a reset (`pointerBy`, `TakePointer`, `DropPointer`). Each of these requests is two actions, so that a mutant can take the lock away and show the order of steps that the lock prevents. The lock order of the code is the `ActiveGame` mutex, then the guard, then the backend mutex, then the connection mutex.

Every write of a Job goes through `Own(j)`, the job id check of `JobWriter::hold`. The ghost variables (`cut`, `late`, `storyBusy` and the others) record what the rules read. No code reads them.

| Rule | Spec sentence | Mutant that breaks it |
|---|---|---|
| `OneJob` | actions.md 3.5: "one Action is accepted and the other is refused with 409" | `accept-without-busy-check` |
| `BusyRefusesStory` | screen.md 45.17: "Then each response status is 409" | `edit-without-busy-check` |
| `StopLeavesIdle` | dashboard.md 39.4: "Then the stop answers "reset" And the Game is idle". dashboard.md 39.17: "And the status display answers Ready". options.md 24.5: "Then the dock renders set A". | `stop-keeps-the-run`, `idle-stop-keeps-failure-line` |
| `StopNoFailureLine` | dashboard.md 39.4: "And the status display answers with no failure line" | `stop-keeps-failure-line` |
| `StopClearsOptionsFailure` | options.md 24.5: "When the client POST /status/reset-busy Then the dock renders set A" | `stop-keeps-options-failure` |
| `SwipeClearsFailures` | swipe_switch.md 36.7: "Then the status display answers Ready And the dock renders no Options failure" | `swipe-switch-keeps-failure-line`, `swipe-switch-keeps-options-failure` |
| `StoppedJobWritesNothing` | dashboard.md 39.5: "the story log holds no Narration of the stopped Job"; reset.md 7.3 | `job-writer-without-id-check`, `reset-keeps-the-holder` |
| `NextJobKeepsHolder` | dashboard.md 39.5: "the story log holds the Input and the Narration of the second Job" | `job-drop-without-id-check` |
| `RunHoldsItsGame` | autoplay.md: "The run holds its Game from its start until it ends, also between two Jobs."; 41.17; 41.27 | `job-drop-removes-the-run`, `run-end-without-run-check` |
| `StoppedRunStartsNoJob` | autoplay.md 41.6: "And no further Job of the run starts"; 41.26 | `accept-without-run-check` |
| `RunRefusesConfig` | autoplay.md 41.12: "the config writes are refused and the config stays" | `config-write-during-a-run` |
| `RunEndReasons` | autoplay.md: "A run ends for one of these reasons: ..." | `moved-job-ends-as-failure` |
| `BudgetBound` | autoplay.md: "The **budget** is the number of cycles" | `budget-checked-too-late` |
| `RunFailureLineStays` | autoplay.md 41.4 and 41.23: "the status display answers with the failure line" | `not-accepted-without-failure-line` |
| `RunEndsIdle` | autoplay.md 41.1: "And the Game is idle" | `run-end-keeps-the-holder` |
| `DeleteNotActive` | games.md 19.2: "Deleting the active game returns 400" | `delete-without-the-pointer-lock`, `switch-without-the-pointer-lock` |
| `ActiveExists` | storage.md: "The active Game always names a Game that exists."; games.md 18.2 and 19.2 | `reset-without-the-pointer-lock`, `create-without-the-pointer-lock`, `switch-without-the-pointer-lock` |

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
| `delete-race` | 2 Games and no Job: delete, switch, create, reset, each under the outer lock |

A larger bound costs much more: `run-and-jobs` with 3 Jobs has 3.1 million states and takes 2.5 minutes.

The model leaves out:

- Storage faults. A refused write of the failure line leaves the run ended with no failure line. That is a storage fault, not a gate order.
- The stop whose dock read fails (dashboard.md 39.8). `GameWriter::stop` reads the dock before it removes the holder, so the failed branch changes nothing. No rule checks that order.
- That a Job writes only to its own Game (games.md 18.4). Each Job keeps its Game id from its accept, so the model holds this by construction.
- That a Job always ends (actions.md I.1). TLC checks that no state is stuck, but no rule checks that each Job reaches its end.
- The story content. The model keeps only the type of the last Message, which is what `AutoplayRun::decide` and the plan read. So no rule counts the writes of one Swipe state.
- The config. The model keeps no config variable, so `YouChangeTheConfig` changes no variable. It is a step that changes nothing, and TLC finds no new state for it. The same holds for `YouEditAMessage`, which writes only the story content. Both actions stay in the model for their guard: the busy check (`BusyRefusesStory`) and the refusal during a run (`RunRefusesConfig`). The one write of a story action that the model keeps is the clear of the failure line and of the Options failure in `YouSwitchOrDeleteASwipe` (`SwipeClearsFailures`).
- A restart of the server. The busy registry lives in memory only, so a restart loses every holder.

The model has the branch of `accept_for_autoplay` that answers `NotAccepted::StillThinking`, where the run's own Job is in flight. TLC never reaches it, as the code comment says.

## The diagrams

`check.py` generates the diagrams of `docs/diataxis/reference/diagrams/` from the model, one page each. `diagrams/diagrams.toml` lists them:

- **A state chart for each `[[chart]]`.** Each one is a config and a view of one axis: the busy state of a Game, its failure line and Options failure, the life of one Job, the life of one Autoplay run. Give each chart one axis: a chart of two axes is hard to read. The first line of the config, `\* Scope: ...`, goes on the page.
- **A sequence diagram for each scenario of `diagrams/scenarios.mjs`.** A scenario is a named order of actions. TLC takes exactly those steps and checks every INVARIANT of `diagrams/scenarios.cfg` in each state, so a diagram cannot show an order that the model does not have. Each scenario names the rule that it shows (`rule`), states that rule in words for the page (`says`), and names its spec scenario (`spec`). A page names no spec scenario number: a Diátaxis doc cites none. TLC checks `rule` along the order only when it is an INVARIANT. The page says so when it is not.

`check.py` writes the pages only when every config gives its expected result, and only when an input changed. The inputs are `model/*.tla`, every file in `diagrams/`, `check.py` and the scripts of `/statechart`. Each page carries the hash of the inputs in its marker line. `scripts/verify_coverage.py` warns at the end of every `build.py` run when a page carries an older hash. `--diagrams` checks no config: use it only when the model did not change. To change a page, change its view, its config or `diagrams/scenarios.mjs`, then run `check.py`. A page edited by hand goes at the next run.

To add a diagram: add a `[[chart]]` with its `page`, `config` and `view`, or a scenario with its `title`, `says`, `steps`, `constants`, `rule` and `spec`. Then run `check.py`. Each config of `diagrams/` is a scope for a diagram, not a check: the checks of the rules are the configs of `model/`.

## Known findings

No config breaks at the code of today, so `known_breaks` in `coverage.toml` is empty.

The trial log (`work/wayfinder-maps/statechart-models/assets/trial-log.md`) keeps the past findings and their fixes.
