# The trial TLA+ model of the gate, built from the spec only

Labels: wayfinder:prototype. For ticket [03](../issues/03-prototype-spec-only-gate-model.md).
Date: 2026-10-29; verdicts revised 2026-10-30 after the owner's judgement (section 11). Method: one TLA+ model written from the docs and the spec, checked with TLC through the `statechart` skill; then a read of `src/` and six replay sketches, run once against the code and removed.

**Labels.** **known** = I read the file, the test body or the TLC report. **inferred** = I reached it from a read, not from a run. **guessed** = I did not confirm it.

## 1. Result in brief

- The TLA+ model found three rules that `game_flow.md` does not state (**known**). The three rules are one discipline: the end of a Job, the end of an Autoplay run and the accept of a run's loop each act only for their own id. The code holds all three rules. `game_flow.md` need not state them: they are event rules and rules of one part, and `docs/AGENTS.md` puts those in the specs and the code (owner, 2026-10-30). The spec states G2 (autoplay.md 41.6, 41.7 and its intro) and G1 for runs (41.17). G3 is not clear in autoplay.md ("a Job" or "a Job of the run").
- `game_flow.md` holds event rules that the specs already hold, and mechanics that the code holds. Section 12 lists them.
- Run 2 breaks six rules when the job id check is a constant (**known**). So the rules see what the check protects.
- No replay found a code defect: the six sketches pass on the code of today (**known**, one run: `6 passed` in 0.56 s).
- The yield for the tests: six candidate tests. Each forces an order that no test forces today, or reads a record that no test reads after a late answer (**known** from the test bodies).
- Five of the fifteen rules cannot break in this TLA+ model (**known**, run "all off"). The TLA+ model writes a refusal as a condition on an action, so those rules restate the docs and add no evidence.

## 2. The files

All files are in [03-gate-spec-only/](03-gate-spec-only/).

| File | What it is |
|---|---|
| [GateSpecOnly.tla](03-gate-spec-only/GateSpecOnly.tla) | The TLA+ model, version 2. Each rule quotes its sentence. |
| [GateSpecOnly.view.mjs](03-gate-spec-only/GateSpecOnly.view.mjs) | The view: the labels in the reader's words |
| [configs/](03-gate-spec-only/configs/) | One TLC config for each run of section 4 |
| [runs/](03-gate-spec-only/runs/) | The `report.md` of each run, and `chart.mmd` and `chart.svg` where TLC wrote a chart |
| [v1/](03-gate-spec-only/v1/) | Version 1 of the TLA+ model and its runs. Version 1 is the TLA+ model of the first run, written before any read of `src/`. |

**Version 2.** The full check of version 1 with the gaps closed did not end: TLC reached 28.4 million states in 24 minutes and found no break. Version 2 makes the state space smaller: an ended Job or loop keeps no field that no later step and no rule reads. The actions and the rules are those of version 1. Version 2 breaks the same rules as version 1 in run 1, in run 2 and in each gap run (**known**, the reports in `v1/runs/` and `runs/`).

**The chart of run 1** ([SVG](03-gate-spec-only/runs/run1-docs/chart.svg), [Mermaid](03-gate-spec-only/runs/run1-docs/chart.mmd), [PNG](03-gate-spec-only/runs/run1-docs/chart.png)) draws the bug paths only, because TLC reached more than 500 states. In the chart of run 2, the late saves of a stopped Job are loops on one state ("Idle; a stopped Job still runs"), so read the bug paths of run 2 in its report.

**Make the files again.** The runner needs `JAVA` and `TLA2TOOLS` (see the map Notes). Each `--out` folder must be new.

```sh
export JAVA=/tmp/statechart-tools/jdk/bin/java TLA2TOOLS=/tmp/statechart-tools/tla2tools.jar
cd .scratch/statechart-models/assets/03-gate-spec-only
../../../../.agents/skills/statechart/scripts/statechart GateSpecOnly.tla \
  --config configs/GateSpecOnly.cfg --view GateSpecOnly.view.mjs --out /tmp/gate-run1
```

## 3. How the TLA+ model was built

**The discipline.** I wrote version 1 from `docs/diataxis/reference/game_flow.md` and the gate scenarios of `docs/specs/` (dashboard 39.*, autoplay 41.*, reset 7.*, actions 3.3 to 3.5 and 6.4, screen 45.3 to 45.5, story_log 8.7, swipe_switch 36.*, games 18.4 and 19.*, changes 43.11 to 43.14, options 24.21). I read no file of `src/` before the first run (**known**). One file of `src/` is in the git status of this checkout (`gate.rs`: a swipe switch on a Message that is not the last now answers `LastMessageChanged`); that change is outside the TLA+ model.

**The scope.** One Game. The holder: none, a plain Job (a free-text Action), or an Autoplay run with or without its Job (an Impersonate Job, then a Continue Job). The parts of a Job: the Input save, the step names, the narrator call, the text save, the analysis call, the state update, the Options call and its save, the failure line, and the end. Stop, reset, delete (of a Game that is not active), a switch of the active Game, a config write, a story write (edit, delete, swipe switch), and a restart of the engine. Each save of a Job is one step of TLC ("One hold of the registry covers one check and its transactions"). Each model call is a step of its own that reads no holder ("No hold of the registry spans an LLM call").

**Left out.** Retry, Retrigger, `/options`, event Narrations, the after-narration hook, a second Game with a Job of its own, what an edit, a delete or a switch does to a Swipe state, and the order of the checks inside one refusal.

**The three gaps as constants.** Where the docs say nothing, a constant names the reading. `FALSE` is the literal reading of the docs; `TRUE` closes the gap.

| Constant | Gap | What the docs say | What they leave open |
|---|---|---|---|
| `ReleaseChecksId` | G1 | "A stop, a reset and a delete of the Game remove the holder." | What the end of a Job or of a run does to the holder |
| `LoopNeedsOwnHolder` | G2 | "While a holder exists, the busy rule refuses every story write as busy. For an Autoplay run, only the accept of the run's own loop passes." | Whether the accept of a run's loop passes when no holder exists |
| `LoopReadsJobResult` | G3 | "In one hold of the registry, the accept reads the end of the story, counts the cycle that the last Job ended, and either finishes the run or accepts the next Job." | Where the accept learns that the last Job failed |

**The guard.** `IdCheck = FALSE` removes the job id check of each save and of each step name ("Each save method of a `JobWriter` checks, in one hold of the registry, that the job entry of the Game's holder still holds the Job's id").

## 4. The runs

"Breaks" names the rules that TLC found broken. After the first break, the runner stops each TLC run at 100,000 distinct states, so a run with a break checks its other rules only in part; the "rest" runs check them in full. "States" is the count of distinct states of TLC on the config as written; for a run with a break, it is the count at the first break.

| Run | Config | Jobs, runs, restarts | Gaps | Guard | States | Time | Result |
|---|---|---|---|---|---|---|---|
| 1, the docs as written | `GateSpecOnly.cfg` | 3, 2, 1 | open | on | 55 | 28 s | R02, R03, R04, R05, R10 break |
| 1, the rest (version 1) | `run1-rest.cfg` | 3, 2, 1 | open | on | 425,829 | 66 s | R07 breaks; the rest not checked in full |
| 1 (b), the rest | `run1-b-rest.cfg` | 2, 2, 0 | open | on | 494,842 | 30 s | the nine other rules hold |
| 1 (c), the rest | `run1-c-rest.cfg` | 2, 1, 1 | open | on | 152,135 | 14 s | the nine other rules hold |
| 1, plain Jobs only | `run1-plain-jobs.cfg` | 3, 0, 1 | open | on | 116 | 16 s | R02 breaks |
| G1 open alone | `gap-g1-only.cfg` | 3, 2, 1 | G1 | on | 137 | 20 s | R02, R03 break |
| G2 open alone | `gap-g2-only.cfg` | 3, 2, 1 | G2 | on | 55 | 22 s | R04, R05 break |
| G3 open alone | `gap-g3-only.cfg` | 3, 2, 1 | G3 | on | 3,608 | 20 s | R10 breaks |
| 1b, the gaps closed | `run1b-gaps-closed.cfg` | 3, 2, 1 | closed | on | 15,798,130 | 1,156 s | all 15 rules hold |
| 1b (a) | `run1b-a-one-run.cfg` | 3, 1, 1 | closed | on | 3,172,458 | 264 s | all 15 rules hold |
| 1b (b) | `run1b-b-two-jobs.cfg` | 2, 2, 0 | closed | on | 196,199 | 27 s | all 15 rules hold |
| 1b (c) | `run1b-c-restart.cfg` | 2, 1, 1 | closed | on | 95,286 | 11 s | all 15 rules hold |
| 2, the guard off | `run2-guard-off.cfg` | 3, 2, 1 | closed | off | 38 | 40 s | R01, R05, R06, R07, R08, R09 break |
| 2 (b) | `run2-b-guard-off.cfg` | 2, 2, 0 | closed | off | 28 | 54 s | the same six break |
| 2 (b), the rest | `run2-b-rest.cfg` | 2, 2, 0 | closed | off | 2,728,171 | 151 s | R02, R03, R04, R10 hold |
| All off | `all-off.cfg` | 2, 1, 1 | open | off | 2,271,206 | 108 s | R11 to R15 hold |

Run 1b at the bounds of run 1 is the one full check at those bounds. It ran on version 2; on version 1 the same check reached 28.4 million states in 24 minutes and did not end.

Every run has `Budget = 1`. A run of one cycle has two Jobs: an Impersonate Job and a Continue Job.

## 5. The rules

"Run 1" is the docs as written, "run 1b" the gaps closed, "run 2" the guard off. "Holds" in run 1 means it holds in full at the small bounds of runs 1 (b) and 1 (c). The ticket 01 column names the row of [the ticket 01 table](01-gate-rules-and-their-tests.md) and its status there.

| Id | Rule sentence (quoted) | Row of ticket 01 | Run 1 | Run 1b | Run 2 |
|---|---|---|---|---|---|
| R01 | "A stop, a reset and a delete of the Game remove the holder. A late save of their Job lands nowhere." (game_flow.md, The Writer Rule) | The same sentence, partly pinned; and "Each save method of a `JobWriter` checks ...", partly pinned | holds | holds | breaks, 3 steps |
| R02 | "While a Job runs, its Game is busy." (game_flow.md, Overview), with 39.5: "the story log holds the Input and the Narration of the second Job" | "A Game with no holder is idle, and a Game has at most one holder.", pinned. No row for the end of a Job: game_flow.md has no sentence for it | breaks (G1), 4 steps | holds | holds |
| R03 | "An Autoplay run. It lasts from the start of the run to its end, also between two Jobs of the run." (game_flow.md, The Busy State and Its Holder) | The same sentence, pinned | breaks (G1), 4 steps | holds | holds |
| R04 | "For an Autoplay run, only the accept of the run's own loop passes." (game_flow.md), with 41.6 and 41.7: "no further Job of the run starts" | "While a holder exists, the busy rule refuses every story write as busy. ...", pinned | breaks (G2), 3 steps | holds | holds |
| R05 | "A **stop** removes the holder, clears the failure line ... The Game is idle at once." (game_flow.md, Stop and Reset) | "A stop removes the holder, clears the failure line and clears the Options failure ...", partly pinned; "The Game is idle at once.", pinned | breaks (G2), 3 steps | holds | breaks, 6 steps |
| R06 | "A stop or a restart between the two writes leaves the Swipe with its pre-analysis state." (game_flow.md, The State of a Game) | The same sentence, partly pinned | holds | holds | breaks, 9 steps |
| R07 | "A **stop** ... clears the Options failure of the dock record." (game_flow.md, Stop and Reset) | "A stop removes the holder, clears the failure line and clears the Options failure ...", partly pinned | breaks (G2 path), 13 steps | holds | breaks, 12 steps |
| R08 | "The accept clears it in storage, because a restart during the new Job would otherwise show the old failure again." (game_flow.md, Status) | No row: the Status section was outside the scope of ticket 01 | holds | holds | breaks, 7 steps |
| R09 | "A plain Job. Its **job entry** holds the Job's id and the current `Step`." and "Each step sets its name in the job entry of the Job before it runs." (game_flow.md) | "A plain Job. Its job entry holds the Job's id and the current `Step`.", partly pinned | holds | holds | breaks, 5 steps |
| R10 | "A run ends for one of these reasons: a stop, a reset or a delete of the Game, the budget, a Job that ends with a failure line, or an accept that cannot read a record of the Game." (autoplay.md, intro) | No row: ticket 01 read the spec as tags only | breaks (G3), 8 steps | holds | holds |
| R11 | "the number of the cycle in progress and the budget" (game_flow.md, Status) | No row | holds by construction | holds | holds |
| R12 | "While a holder exists, the busy rule refuses every story write as busy." (game_flow.md) | The same sentence, pinned | holds by construction | holds | holds |
| R13 | "While an Autoplay run holds the Game, a write of the Game's config columns ... is refused as busy." (game_flow.md) | The same sentence, pinned | holds by construction | holds | holds |
| R14 | "Only the Job that makes a Swipe writes its Swipe state, at most twice" (game_flow.md) | The same sentence, partly pinned | holds by construction | holds | holds |
| R15 | "A restart loses every holder, so every Game is idle after a start." (game_flow.md) | The same sentence, not pinned | holds by construction | holds | holds |

"By construction": the rule holds also with the guard off and every gap open (run "all off"), so no change of the constants can break it.

**The two kinds of check.** The save check protects R01 and R05 to R09: run 2 breaks them, and run 1b holds them. The "own id" checks of the end of a Job, the end of a run and the loop's accept protect R02, R03, R04 and R10: run 1 breaks them, and run 2 (b) holds them without the save check (**known**).

## 6. The breaks of run 1, and their replays

### G1: the end of a Job or of a run removes only its own entry (R02, R03)

**Paths.**

- R02, run 1, 4 steps: Idle → you start Autoplay → you press Stop → you send an Action → the run finds another holder and ends: "Idle; a Job runs with no entry".
- R02, plain Jobs only, 5 steps: Idle → you send an Action → you press Stop → you send an Action → a save of the stopped Job is refused → the stopped Job ends: "Idle; a Job runs with no entry".
- R03, 4 steps: Idle → you start Autoplay → you press Stop → you start Autoplay → the run finds another holder and ends: "Idle", while the second run's loop still runs.

**The code (read after run 1).** `JobWriter::drop` removes the entry only "while it still holds this Job's id". `GameWriter::end_autoplay` removes the holder only when `is_autoplay_run(run)`. `GameWriter::accept_for_autoplay` answers `AutoplayAccept::Ended` and writes nothing when the holder is gone or names another run (**known**).

**The tests.** 41.17 pins the order of R03: it stops run A during its Job, starts run B, and waits with `wait_for_jobs_running_at_most(1)` until run A ends while run B is held (**known**). For plain Jobs, 39.5 has the order, but it does not force the end of the stopped Job before the text save of the next Job: it releases both holds and then waits for both Jobs (**known**). So 39.5 pins the check of `JobWriter::drop` only when the stopped Job ends first (**inferred**). No test starts an Action after a stop between two Jobs of a run while the run's loop still waits (**known**: in 41.7 the Action comes after `wait_for_run`).

**Sketch G1.** Passes on the code.

```rust
// G1, R02: stop between two Jobs of a run, then an Action, then the run comes to its accept.
#[tokio::test]
async fn sketch_g1_an_action_after_a_stop_keeps_the_game_when_the_stopped_run_comes_to_its_accept() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT).turn(FIRST_NARRATION));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    let pause = pause_between_two_jobs(&app, &state, game).await;
    stop(&app).await;
    let action_hold = model.hold(Role::Narrator);
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(action_hold.wait_started(1000), "the Action's Job reaches its narrator call");
    pause.release();
    assert!(
        wait_for_jobs_running_at_most(&state, 1, 2000).await,
        "the stopped run's loop ends while the Action's Job stays held"
    );
    assert_status(&app, "Generating narration...", "the Action's Job still holds the Game").await;
    action_hold.release();
    assert!(wait_settled(&state, 2000).await, "the Action's Job settles");
    let story = story_after_opening(&state, game);
    assert_eq!(
        story.last(),
        Some(&(MessageType::Narration, FIRST_NARRATION.to_string())),
        "the Action's Job saves its Narration: {story:?}"
    );
}
```

**Verdict (revised 2026-10-30).** The spec holds the form for runs (41.17). The form for plain Jobs is a rule of one part (`JobWriter::drop`), so its home is the code and its tests, not `game_flow.md`. The code is right. The TLA+ model broke only through the literal reading of a doc that need not speak. Candidate tests: sketch G1, and a form of 39.5 that forces the order (sketches R08 and R09 below force it).

### G2: the accept of a run's loop passes only while its run holds the Game (R04, R05, and R07 on the same path)

**Path.** R04 and R05, 3 steps: Idle → you start Autoplay → you press Stop → the run starts its next Job: "A run holds the Game, its Job starts". R07 breaks 10 steps later on the same path, when the Options call of that Job fails.

**The code.** In `GameWriter::accept_for_autoplay`, a holder that is gone gives `None => return AutoplayAccept::Ended` (**known**).

**The spec and the tests.** autoplay.md: "A run ends for one of these reasons: a stop, ...". 41.7 pins this path: it stops the run between two Jobs, releases the pause, and reads one narrator call only (**known**).

**Verdict (revised 2026-10-30).** The spec holds the rule (41.6, 41.7 and the intro of autoplay.md), and 41.7 pins it. The code agrees. Nothing to do.

### G3: the loop ends on a failure line only when its own Job wrote it (R10)

**Path.** 8 steps: Idle → you send an Action → the Job saves its Input → the Job starts its next step → the narrator call of the Job fails → the Job saves a failure line → the engine restarts (the end of the Job gives the same path) → you start Autoplay → the run ends: "Idle with a failure line". A run on a Game that holds an old failure line ends at once and plays no cycle.

**The code.** `AutoplayRun::run_jobs` ends the run on `JobEnd::Failed` of its own Job. `AutoplayRun::decide` reads only the end of the story. The accept clears the failure line in its transaction (`plan_and_start` calls `accept_in_tx`) (**known**).

**The tests.** 41.4 fails a Job inside the run. No test starts a run on a Game that holds a failure line (**known**: `failed(Role::Narrator)` occurs once in `tests/http/autoplay/`, in 41.4).

**Sketch G3.** Passes on the code.

```rust
// G3, R10: a run on a Game that holds a failure line from an earlier Job.
#[tokio::test]
async fn sketch_g3_a_run_on_a_game_with_a_failure_line_plays_its_budget() {
    let model = Arc::new(ScriptedModel::new().failed(Role::Narrator).turn(FIRST_NARRATION));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_for_jobs(&state, 2000).await, "the failing Job ends");
    let failed = fetch_status_label(&app).await;
    assert_ne!(failed, "Ready", "the failed Job leaves a failure line");
    assert!(start_autoplay(&app, "cycles=1").await.status().is_success());
    wait_for_run(&state, game).await;
    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, "look".to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
        ],
        "the run plays its budget: a Continue answers the Input"
    );
    assert_status(&app, "Ready", "the run's accept cleared the old failure line").await;
}
```

**Verdict (revised 2026-10-30).** The spec is not clear: its intro says "a Job that ends with a failure line", not "a Job of the run", but its cycle sentence says "a Job of the run". The code is right. The test is missing. Candidates: the words "a Job of the run" in the intro of autoplay.md, and sketch G3 as the test of that scenario. The sketch also pins that the accept of a run clears an old failure line.

## 7. The breaks of run 2 (the guard off), and their replays

Run 2 breaks, so the rules miss nothing that the check protects (**known**). Each path below starts with a stop of a Job and then a late step of that Job. With the guard on, the code refuses each late step through `JobWriter::hold` (**known**).

| Rule | Path of run 2 | A test that forces this order today | Sketch |
|---|---|---|---|
| R01 | 3 steps: Action, Stop, the stopped Job saves its Input | None. The Input save comes before the first model call, and the LLM fake holds model calls only (**known** from `Job::generate`: room check, Input save, narrator call). The later saves: 39.4 and 39.5 | None: a tier-1 test needs a new hold before the Input save |
| R06 | 9 steps: the stopped Job saves the new state | 39.4, the row "Narration, the analysis call": no room on the Narration after the stop (**known**) | None needed |
| R05 | 6 steps: the narrator call of the stopped Job fails, and it saves a failure line | None. 39.4 reads the status before the late answer only, and its late answers are texts (**known**) | R05 |
| R07 | 12 steps: the Options call of the stopped Job fails, and it saves an Options failure | None. The Options row of 39.4 reaches the check of `save_options`, but it reads the story log only, not the dock (**known**) | R07 |
| R08 | 7 steps: stop, a next Job, the late failure line of the stopped Job | None. 39.5 has no failure (**known**) | R08 |
| R09 | 5 steps: stop, a next Job, the stopped Job starts its next step | None reads the step of the next Job after the stopped Job ends (**known** for 39.5) | R09 |

The four sketches pass on the code.

```rust
// Guard off, R05: a stopped Job whose late narrator call fails.
#[tokio::test]
async fn sketch_r05_a_stopped_job_whose_late_call_fails_leaves_the_game_idle() {
    let model = Arc::new(ScriptedModel::new().failed(Role::Narrator));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(hold.wait_started(1000));
    stop(&app).await;
    hold.release();
    assert!(hold.wait_taken(1000));
    assert!(wait_for_jobs(&state, 2000).await);
    assert_status(&app, "Ready", "the late failure of the stopped Job writes no failure line").await;
}

// Guard off, R08: the late failure of a stopped Job while the next Job runs, and after it.
#[tokio::test]
async fn sketch_r08_a_stopped_jobs_late_failure_does_not_show_for_the_next_job() {
    let model = Arc::new(ScriptedModel::new().failed(Role::Narrator).turn(FIRST_NARRATION));
    let first = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(first.wait_started(1000));
    stop(&app).await;
    let second = model.hold(Role::Narrator);
    assert!(post_action(&app, "listen").await.status().is_success());
    assert!(second.wait_started(1000));
    first.release();
    assert!(first.wait_taken(1000));
    assert!(wait_for_jobs_running_at_most(&state, 1, 2000).await);
    assert_status(&app, "Generating narration...", "the next Job's status stays").await;
    second.release();
    assert!(wait_settled(&state, 2000).await);
    assert_status(&app, "Ready", "no failure line of the stopped Job shows").await;
}

// Guard off, R09: a stopped Job's next step does not change the status of the next Job.
#[tokio::test]
async fn sketch_r09_a_stopped_jobs_next_step_keeps_the_status_of_the_next_job() {
    let model = Arc::new(
        ScriptedModel::new()
            .narration("first")
            .after_narration(analysis_report(&[], None))
            .narration("second")
            .after_narration(analysis_report(&[], None)),
    );
    let analysis = model.hold(Role::Quantifier);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(analysis.wait_started(1000));
    assert_status(&app, "Quantifying...", "the first Job is at its analysis").await;
    stop(&app).await;
    let second = model.hold(Role::Narrator);
    assert!(post_action(&app, "listen").await.status().is_success());
    assert!(second.wait_started(1000));
    analysis.release();
    assert!(analysis.wait_taken(1000));
    assert!(wait_for_jobs_running_at_most(&state, 1, 2000).await);
    assert_status(&app, "Generating narration...", "the stopped Job's next step is refused").await;
    second.release();
    assert!(wait_settled(&state, 2000).await);
}

// Guard off, R07: a stopped Job whose late Options call fails.
#[tokio::test]
async fn sketch_r07_a_stopped_jobs_late_options_failure_leaves_the_dock_clear() {
    const UNPARSEABLE: &str = "no parseable options in this response";
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_NARRATION)
            .options(UNPARSEABLE)
            .options(UNPARSEABLE),
    );
    let options = model.hold(Role::Options);
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    storage
        .seed_world(
            &WorldCard {
                key: "always_on_world".to_string(),
                name: "Always On World".to_string(),
                description: "A World with options always on.".to_string(),
                options_always_on: true,
                ..Default::default()
            },
            &TestMap::single_room("start"),
        )
        .expect("the always-on World seeds");
    let created = post_form(&app, "/games", "world_key=always_on_world&persona_key=test_player").await;
    assert!(created.status().is_success());
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(options.wait_started(2000), "the Job reaches its Options call");
    stop(&app).await;
    options.release();
    assert!(options.wait_taken(1000));
    assert!(wait_for_jobs(&state, 2000).await);
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("Options generation failed"),
        "the late Options failure of the stopped Job lands nowhere: {dock}"
    );
    assert_status(&app, "Ready", "the stopped Game stays idle").await;
}
```

**How the sketches ran.** I put the six sketches in a temporary file `tests/http/autoplay/zz_ticket03_replay.rs` with one `mod` line in `tests/http/autoplay/mod.rs`, ran `python build.py test-pattern "sketch_" --test-target http` once (`6 tests run: 6 passed, 439 skipped`, log `logs/build_20261029_232607.log`), and removed the file and the line. `git status` of `tests/http/autoplay/` is clean (**known**). I did not run a sketch against a changed gate, so I do not know that each sketch fails when its check goes. The R07 sketch uses the `Unavailable` branch of `save_options`; a sketch of the late offered set would read the dock for the set of the stopped Job.

**Verdict for run 2.** The code is right on each path. R01 at the Input save is the order that ticket 01 named as untested; the TLA+ model finds it again, but no tier-1 tool reaches it. The Options row of 39.4 is a case that ticket 02 named: a test executes the check and asserts nothing on its result.

## 8. What the TLA+ model could not do

- **Five rules hold by construction (R11 to R15).** The TLA+ model writes a refusal as a condition on an action, so the ghost variable of the rule can never turn on. For R15, the restart sets the holder to none; for R14, a Job passes the text save and the state update once each. These rules restate their ticket 01 rows and add no evidence. "A restart loses every holder" stays not pinned, as ticket 01 said: only a restart test or a storage read pins it.
- **The atomic steps are assumptions.** One hold for one check and its transactions, the accept's read, count and decision in one hold, the reset's two writes in one hold: each is one step of TLC, so TLC cannot check them. Guardrail rule 18 pins the one-hold shape (ticket 01).
- **The delete case of R01 cannot break.** A save of the TLA+ model also needs the Game to exist, so run 2 cannot show what the id check does after a delete. Ticket 01's gap (a delete of the Game of a plain Job, then its late save) stays. I do not know what a transaction does on a deleted Game without the id check.
- **Two kinds of claim are not stated.** "A config write passes while a plain Job runs" says that an action is possible; the TLA+ model allows it but states no rule for it. "The refusal comes before every other check" is about the order inside one request, which the TLA+ model does not have.

## 9. Cost

| Item | Count |
|---|---|
| Claim to the first TLC run (reading the docs and the spec, version 1 of the TLA+ model and its view) | about 12 minutes (23:06 to 23:18) |
| First TLC run | 36 s (version 1); 28 s (version 2) |
| Read of `src/`, six sketches, one test run | about 8 minutes (23:18 to 23:26) |
| Pause (session limit); the full check of version 1 ran and did not end | about 18 minutes (23:26 to 23:44) |
| Version 2, the small full checks, the SVGs | about 20 minutes (23:44 to 00:04) |
| The write-up | about 10 minutes (00:04 to 00:14) |
| Work in all, without the pause | about 50 minutes |
| TLC wall time of the runs of section 4, without the full run 1b | about 15 minutes |
| TLC wall time of the full run 1b | 19 minutes (version 2); version 1 stopped by hand after 24 minutes |
| States: largest full check that ended | 15,798,130 (run 1b) |
| States: a break | from 28 to 3,608 states |

The toolchain of the map Notes was still in `/tmp`, so the cost has no install.

## 10. For ticket 04

- A spec-only TLA+ model is a one-time audit. It found no place where the spec and the code disagree, and no code defect. Its findings go to their homes: one spec sentence that is not clear (G3), orders that no test forces, and event rules in `game_flow.md` that a spec already holds (section 12).
- It found test gaps of one kind: orders that no test forces (the end of a stopped Job or run against the next holder, a late failure, a late step name), and records that no test reads after a late answer (the dock, the status). Six sketches, all green on the code today.
- Run 2 is a mechanical check of one guard, like a mutation of the id check. It shows which rules depend on the check, and names the order that a test must force for each. It does not show that a test fails when the check goes; only a run against a changed gate shows that.
- A third of the rules (5 of 15) restate the docs by construction. A rule earns its place in a TLA+ model when it is about an interleaving: a stop, a late step, a second holder.
- The value came from the first runs: every break showed in fewer than 4,000 states, and the full check of 15,798,130 states found nothing more (**known**).
- The base rate after this trial: four TLA+ models, no code defect. The three TLA+ models written from the code found no gap; the TLA+ model written from the docs and the spec found the three "own id" rules and the orders that no test forces (**known**).

## 11. The owner's judgement (2026-10-30)

- **The verdicts of G1, G2 and G3: judged.** `game_flow.md` need not cover every use case: the specs hold the event rules, and the docs give an understanding. So the three gaps are not doc gaps, and the first draft of three doc sentences is withdrawn. Sections 1, 6 and 10 carry the revised verdicts.
- **The use of the TLA+ model: a leaning, for ticket 04.** The TLA+ model is a one-time audit that picks up gaps. A TLA+ model in `tmp/` is written again for each audit. Ticket 04 decides the role.
- **Not judged.** The quoted rule sentences of R01 to R10, the six candidate tests (G1, G3, R05, R07, R08, R09), the G3 words in autoplay.md, and a spec scenario for the delete of a Game during a plain Job. They stay candidates for the effort that takes them up. The map rules them out of its scope.

## 12. Event rules and mechanics in `game_flow.md`

The start list for a trim of `game_flow.md`, by the rule of `docs/AGENTS.md` ("Where a fact lives"). The map rules the trim out of its scope. The "where" column is **inferred** where it rests on ticket 01, and **known** where I read the spec scenario.

| Sentence of `game_flow.md` | Kind | Where it already lives |
|---|---|---|
| "`Job::start` accepts the request and spawns the Job. An Autoplay run accepts each of its Jobs through `GameWriter::accept_for_autoplay` and runs it through `Job::run_accepted`." | Function names for the steps of the code | The code |
| "`accept` reads the end of the story and asks `JobPlan::for_request` for the plan. It then checks that the records the plan names exist ..." | Implementation steps | The code; the outcomes in failure_display 38.13 and 38.15 and autoplay 41.23 |
| "A stop, a reset and a delete of the Game remove the holder." (two times) and "A late save of their Job lands nowhere." | Event rule | dashboard 39.4 and 39.5, reset 7.3, autoplay 41.6, 41.7, 41.15 and 41.16. No scenario has the delete of a Game during a plain Job |
| "Each save method of a `JobWriter` checks, in one hold of the registry, that the job entry ... still holds the Job's id. ..." | How one part works | The code (`JobWriter::hold`); the outcome in 39.4 and 39.5 |
| "An edit, a delete and a swipe switch are refused while the Game is busy. The refusal comes before every other check." | Event rule, and the order of the checks inside one part | story_log 8.7 and swipe_switch 36.3; the order only in the code comment of `GameWriter::require_idle` |
| The Stop paragraph after "removes the holder": "The Game is idle at once. The Job starts no further step, ... the id check refuses its answer." | Event rules | 39.4 and 39.5 |
| "A Job that ran on the removed Game writes nothing more." | Event rule | reset 7.3, autoplay 41.15 |
| "The accept clears it in storage, because a restart during the new Job would otherwise show the old failure again." | A reason | Its home is the Explanation doc |
| "While an Autoplay run holds the Game, a write of the Game's config columns ... is refused as busy. A config write passes while a plain Job runs." | Not clear: an event rule, or the invariant "a run freezes the config" | autoplay 41.12 and the intro of 41.18; actions 6.4 |

**What stays.** The holder and its two kinds, and one holder for each Game. No saved record holds the holder, so a restart makes every Game idle. `GameWriter` is the only code that opens a transaction. One hold covers one check and its transactions, and no hold spans an LLM call. A Job keeps the Game that its request named. The section on the state of a Game.

**One invariant for several sentences.** "A Job or an Autoplay run changes its Game only while it is the holder." It covers the save check, G1 and G2. G3 is about what the loop reads, so its home is the spec.
