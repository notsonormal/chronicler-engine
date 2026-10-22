# Plan: fix the Autoplay review findings

## 1. Context

The after-plan workflow on the Autoplay build (`old-docs/archived-plans/autoplay-plan.md`) ran a plan check and 4 reviews. The full gate is green (`logs/build_20261021_212733.log`). No finding is a correctness bug. The findings are in `tmp/review/autoplay-findings-{code-review,thermo,antipattern,test-police}.md`. This plan fixes all of them. It also changes the shape of the busy state, after an owner session on 2026-10-21.

Branch: `autoplay-engine`, on top of `be475520` (the after-plan cleanup, committed).

Every decision below comes from the owner session of 2026-10-21 (Q1–Q8). A gap that needs a decision stops the work and goes to the owner. Facts with no label are known (read in the code). **Inferred** facts are labelled; check them before you rely on them.

### 1.1 Decisions

| Id | Decision |
|---|---|
| Q1 | Busy is a property of the Game. The gate keeps **one busy state per Game** with one **holder**: a plain Job, or an Autoplay run. Autoplay is a loop **on top** of Jobs and owns its budget, its cycles and its cycle rule. The Job is unchanged (Q20 of the Autoplay plan holds) |
| Q2 | A test pause point in the Autoplay loop, between two Jobs. It names its Game. The between-Jobs tests run the real loop |
| Q3 | Glossary: add **Busy state**, **Holder**, **Autoplay run**, **Cycle**, **Budget** to `CONTEXT.md` (through `/domain-modeling`); widen **Job entry**. Code: no bare "run" in the gate; `application/autoplay/` keeps "run" |
| Q4 | Keep: the story-log and config controls are view-only only under Autoplay; the dock is disabled under every holder. The spec states the reason |
| Q5 | Keep one `disabled` attribute per control. `view_only` becomes a constructor argument. An Askama macro removes the repeated markup |
| Q6 | In this plan: the `deleteMessage` busy fix, and a rerun of the real-LLM UI pass. To tickets: the Narrator's role health after a Stop |
| Q7 | `GameCatalogue::reset` takes a `GameId` |
| Q8 | No `autoplay_runs` history table in this plan. Ticket for later. Runs stay transient |

### 1.2 Design constraint: per-tab Games later

The owner wants, later, two Games open in two browser tabs at the same time. This plan does not build that, and it must not block it.

- The Job and Autoplay system is keyed by `GameId`. None of it reads `ActiveGame`. The coupling to the one active-Game pointer is in `CurrentGame` and in `GameCatalogue::reset` (fixed here, Q7). The client polls carry no Game id.
- Rules: every new or changed application API takes its `GameId` as an argument. No new read of `ActiveGame` outside `GameCatalogue` and `CurrentGame`. Every per-Game fact (busy, progress, `view_only`) comes from the request's `GameId`. The pause point names its Game. New client code reads the Game's state from the fragment it receives, not from a page-global flag.

## 2. The target design

### 2.1 The busy state (gate)

```text
GameWriter (application/generation/gate.rs)
  BusyRegistry (Mutex)                       # was JobRegistry
    busy: GameId → Holder                    # absent = idle; replaces Holds { jobs, runs }
  enum Holder {
    Job(JobEntry { id, step }),
    Autoplay { run: AutoplayRunId, progress: AutoplayProgress, job: Option<JobEntry> },
  }
```

| Rule | Reads |
|---|---|
| `require_idle(game)` (every plain accept, Edit, Delete, swipe switch, `reserve`) | `busy` has no `game` |
| `accept_for_autoplay(game, run, request)` | `Holder::Autoplay { run, job: None }` with the same run id; sets `job` |
| config write | holder is not `Autoplay` |
| `JobWriter` id check | the `JobEntry` of either holder |
| `JobWriter::drop` | removes `Holder::Job`, or sets `Autoplay.job = None`, only while the id matches |
| `stop` / `replace` / `delete` | remove `busy[game]` |
| `GameViewQuery::status` | one read of the holder: the step, and the progress under `Autoplay` |

The gate applies no cycle rule and no budget rule. **Decision taken in this plan (flag to owner at review):** the progress (`cycles done`, `budget`) is stored *in* `Holder::Autoplay` as data that only the Autoplay loop writes. The loop hands it to the gate inside its accept (§2.2), so there is no separate progress write. A separate autoplay registry would need its own cleanup on stop, reset and delete, and a new run on the same Game could meet the old driver's stale entry while that driver waits for its last model call. Storing the progress with the holder removes it in the same hold as the holder, and the status reads one value.

### 2.2 The Autoplay loop (on top)

```text
AutoplayRun (application/autoplay/driver.rs)
  start(writer, deps, game, budget: Budget)      # Budget parsed first: 0 or >100 is 400 on any Game
    writer.reserve_for_autoplay(game, budget)    # require_idle; makes Holder::Autoplay{job: None}
    spawn_blocking(loop)
  loop:
    pause point (tests only)                     # Q2
    writer.accept_for_autoplay(game, run, decide)  # ONE registry hold, ONE story read:
      decide(tail) -> Decision                     #   pure fn of the loop, runs under the lock,
                                                   #   calls no GameWriter (same rule as accept's `pick`)
        if a Job of this run ended and Tail::awaits_narration is false: done += 1
        done == budget → Decision::Finish(progress)       → gate removes the holder
        else           → Decision::Next(progress, JobRequest::next_in_autoplay(tail))
                                                   → gate stores progress, accepts the Job, sets job
      unreadable story → the gate's existing refusal writes the story's failure line → end Refused
      holder gone / other run id → end Stopped
    Job::run_accepted(accepted, deps) → JobEnd     # Done → loop; Failed → end; EntryMoved → end
  Drop → writer.end_autoplay(game, run)          # removes the holder if the run id matches
```

- The cycle rule is one domain function next to `next_in_autoplay`, `Tail::awaits_narration` in `domain/model/tail.rs` (the inherent-impl guardrail puts `Tail` methods there; `tail.rs` already holds `retrigger_allowed` and `can_delete`), used by both the request rule and the count. It has one unit test for the empty story.
- `Budget` (with `DEFAULT` 10, `MAX` 100, `parse`) lives in `application/autoplay/`. The handler only calls `Budget::parse`.
- One registry hold and one story read per pass: the count, the budget check and the next accept happen together. `finish_cycle`, `CycleEnd`, `RunHeld`, `RunProgress` (domain), `set_autoplay_progress` and the second story read go. `RunRefusal` becomes the accept's result: `Accepted`, `Finished`, `Ended` (holder gone), `Refused(Refusal)`.
- `decide` is a pure function of the tail and the loop's own count. It is the same shape as the `pick` closure that `accept_in_hold` already takes, so the "no `GameWriter` call under the lock" rule of §2.5 holds: the closure gets only the tail.
- **Inferred, check:** the count happens at the start of the next pass, so between the last Job of a cycle and the next pass the status shows the old count for a few milliseconds. No test polls that fast.

### 2.3 The test pause point (Q2)

- `AutoplayPause` in `application/autoplay/`, carried in `JobDeps` next to `JobTasks`. `JobDeps` is built in three places: `bootstrap/wiring.rs` (twice) and `test_support/context.rs` (same precedent: built in production because `testing` is a default feature).
- Production: no pause is armed, so the call returns at once.
- Test API, modelled on `ScriptedModel::hold`: `pause.hold(game) -> PauseHandle`, `handle.wait_paused(ms)`, `handle.release()`. A hold fires once, at the next between-Jobs point of that Game.

### 2.4 One busy answer (owner: minimum scope)

- `BUSY_STATUS` and `THINKING_STATUS` move from `action/handlers/actions.rs` to `adapters/driving/http/utils/response.rs`. The autoplay handler and `error.rs` import them from there.
- A config write under Autoplay returns `Refusal::StillThinking`. `RunHeld` and `ApplicationError::still_thinking` go, so one busy type remains. The posture handler answers it through the existing `Refusal` mapping in `error.rs` (`200` with `BUSY_STATUS`).
- `deleteMessage` in `assets/dashboard-story-log.js` uses the existing `answeredWaitStatus`, as `submitEdit` and `switchSwipe` do (Q6).
- Kept as it is (owner): `answeredWaitStatus` matches the `BUSY_STATUS` markup. Scenario 42.6 guards it: the stub server sends the real constant, so a markup change fails the gate. No new response header, and no posture re-render on a refused config write.

### 2.5 Config write without a closure under the lock

`config_write(game, closure)` goes. `GameCatalogue` reads and validates outside the lock, then calls `GameWriter::save_config(game, &Game) -> Result<(), Refusal>`. That unit checks the holder and writes the config columns through a new `Tx` method (`Tx::set_game_config`, next to `Tx::set_game_failure`, with its own `TxWrite` variant for failure injection) in the same hold. `Storage::update_game_config` goes if it has no other caller. `GameCatalogue` stays the owner of the config rules.

### 2.6 Rendering

- `NarrativeLogTemplate::new(entries, retrigger_allowed, view_only)` and `GamePostureTemplate::from_game(…, view_only)`: no `false` placeholder.
- An Askama macro writes the `disabled` attribute; one macro writes the `run-badge`.
- The story log and the posture read `GameViewQuery::autoplay_holds(game)`. The dock keeps `is_busy`. Spec 41.18 states why (Q4: a plain Job is short and the player keeps control of it; an Autoplay run makes the Game view-only).
- `.games-panel` swaps with `morph:innerHTML`, so the 5 s poll during a run keeps an open rename form and the new-Game form.

### 2.7 Names (Q3)

| Now | After |
|---|---|
| `JobRegistry`, `Holds { jobs, runs }` | `BusyRegistry`, `busy: GameId → Holder` |
| `RunId`, `RunHold`, `start_run`, `end_run` | `AutoplayRunId`, `Holder::Autoplay`, `reserve_for_autoplay`, `end_autoplay` |
| `Busy::{Job, Run}` (status view) | `Busy::{Job, Autoplay}` |
| `Job::run_for_run`, `accept_for_run` | `Job::run_accepted`, `GameWriter::accept_for_autoplay` |
| `JobRequest::next_in_run` | `JobRequest::next_in_autoplay` |
| `GameViewQuery::run_holds` | `GameViewQuery::autoplay_holds` |

## 3. Steps (one change, one commit; owner 2026-10-21)

The work is one change with one gate at the end. The steps run in this order, because each step makes the next one easier. Run `python build.py clippy` and the narrow `test-pattern` checks between steps; run the full gate once at the end.

### Step 1: the test-file split first (no behaviour change)

Do this before any engine edit, so the engine step edits small files.

- `tests/http/autoplay.rs` splits into `tests/http/autoplay/{mod.rs, run.rs, fragments.rs}`; shared helpers in `mod.rs`.
- `create_game_from` moves to `tests/http/support/`; `races.rs` uses it. One `story_of_game(state, game)` helper in `tests/http/support/http_fixtures.rs`, used by `autoplay` and `races.rs`. Widen the doc of `GameViewQuery::story_messages`.
- `python build.py test-pattern autoplay --test-target http` is green, with the same test count as before.

### Step 2: the engine (WP B, D, E, H, Q7)

1. `gate.rs`: §2.1, §2.5. `generation_status.rs`: remove `RunProgress`. `Busy` (in `gate.rs`) becomes `Busy::{Job(step), Autoplay { progress, step }}` with `AutoplayProgress { done, budget }` defined next to `Holder` as plain data (the gate must not import `application/autoplay/`, which imports the gate; the rules on the data live in `application/autoplay/`). `Decision::{Next, Finish}` is defined next to it. **Check:** `gate.rs` has 858 lines; if the change passes the file-length guardrail, move the busy-state types to `generation/busy.rs`.
2. `domain/model/generation.rs` (or `tail.rs`): `next_in_autoplay`, the one "awaits a Narration" rule; unit tests in `generation_tests.rs`, including the empty story.
3. `application/autoplay/`: `Budget`, the loop of §2.2, `AutoplayPause` (§2.3). Tracing: one line per cycle, one per end, as now.
4. `pipeline/job.rs`: `run_accepted(AcceptedJob, &JobDeps) -> JobEnd` (the loop accepts, the Job only runs); `run_for_run` goes; `JobDeps` gains the pause.
5. `games/catalogue.rs`: config writes through `save_config`; `reset(game: GameId)` and its handler in `chat_window.rs` passes `CurrentGame`.
6. `games/view_query.rs`: `status`, `is_busy`, `autoplay_holds` read the one holder.
7. HTTP: `autoplay` handler uses `Budget::parse` before the start; the status `busy_answer` reads `Busy::Autoplay`.
8. Spec `docs/specs/autoplay.md`, then tests in `tests/http/autoplay.rs`:
   - 41.9: a bad budget answers `400` also on a busy Game.
   - The between-Jobs scenarios (41.7, 41.11, 41.12, 41.13 second half, 41.18–41.20, 41.22) use "a run paused between two of its Jobs" and the real loop. A stop at the pause ends the run, and no next Job starts (`assert_no_unscripted_model_call`).
   - New: a run whose story cannot be read ends with the story's failure line, the Game is idle, and no model call is made (`Storage::with_failure(StorageCall::StoriesTail, TestOverride::internal(..))`, exemplar `tests/http/games_config.rs`).
   - New: a run whose Quantifier fails each attempt plays its whole budget (Q16).
   - 41.17 (a stale run end) keeps its point with `AutoplayRunId`.
   - Scenario 6.4 and `tests/http/races.rs` stay unchanged.

### Step 3: HTTP rendering and the dashboard (WP A, C, F, J)

1. §2.4: the move of the two constants, `Refusal::StillThinking` for the config write, `deleteMessage` on `answeredWaitStatus`.
2. §2.6: constructor arguments, the two macros, `morph:innerHTML` on `.games-panel`.
3. Specs and tests:
   - `docs/specs/autoplay.md` 41.18 intro: the Q4 reason. 41.12 is unchanged.
   - `docs/specs/browser_autoplay.md`: 42.6 extends to the swipe switch and the Delete (a refused Delete shows the busy message). 42.3 and 42.4 merge into one scenario ("disabled during the run, enabled after Stop"). 42.1 checks "Ready" first and proves the at-once poll (a `wait_for` under 5 s, or a check that `/status/generating` is requested at once). New stub scenario: an open rename form survives one Games-panel poll.
   - Test fixes: one assertion per control kind in the HTTP and browser control checks (no `>= 4` / `>= 3`); 41.22 splits into two tests, and the Retrigger test checks the enabled state after the stop.

### Step 4: the remaining test fixes, docs, glossary, tickets and the UI pass (WP G, I, K)

1. `JobTasks::wait_until_running_at_most` goes; `running()` becomes `pub`, and the test helper polls it with `wait_for_condition_async` (`tests/test_utils/wait.rs`).
2. `CONTEXT.md` through `/domain-modeling` (Q3).
3. Diátaxis docs for the new parts and names: `game_flow.md` ("The Job Entry and Busy" becomes the busy state and its holder), `architecture_system.md` (`GameWriter` row; `GameCatalogue` writes config through `GameWriter`; `AutoplayRun` row), `two-state-channels.md`, `storage.md`, `startup.md`, `architecture.md`. Check with `/chronicler-docs-hygiene`.
4. Tickets in `.scratch/` (per `docs/agents/issue-tracker.md`): per-tab Games (§1.2); the Narrator's role health after a Stop; an `autoplay_runs` history table (Q8).
5. The `/chronicler-ui-investigator` pass on the final build, with the real connections, on its own probe server and database: a 3-cycle run from the control on a fresh Game; shots of the progress, the view-only story log, the view-only Games panel with the badge, the story after the run, and a Stop in the middle of a second run; an open rename form during a run. Read the probe database and check that each Narration answers the Input before it. Write the notes to `tmp/autoplay-ui/NOTES.md`. The owner judges the impersonate preset from the output.

### Done when

- `python build.py` green (bash timeout 1200; read the newest `logs/build_*.log`).
- `scripts/validate_feature_spec.py` passes.
- One mutation check per new or changed test (break, see red, restore).
- `application/autoplay/` has no uncovered end branch (`cargo llvm-cov` in its own target dir, `ENVIRONMENT.md`).
- The UI pass notes in `tmp/autoplay-ui/NOTES.md`.
- The owner approves the commit.

## 4. Files to modify

- `src/application/generation/gate.rs`, `src/application/generation/job_tasks.rs`
- `src/application/autoplay/{driver.rs, mod.rs}` (+ the pause and `Budget`, new files if the guardrails ask)
- `src/application/pipeline/job.rs`, `src/application/games/{catalogue.rs, view_query.rs}`, `src/application/errors.rs`
- `src/domain/model/{generation.rs, generation_tests.rs, state/generation_status.rs}` (and `tail.rs` if the rule goes there)
- `src/adapters/driven/storage/{transaction.rs, games.rs}`
- `src/adapters/driving/http/{error.rs, app_state.rs, templates.rs, utils/response.rs}`, `action/handlers/{actions.rs, mod.rs}`, `autoplay/handlers/autoplay.rs`, `games/{handlers,templates}/games.rs`, `layout/handlers/endpoints.rs`, `chat_window/handlers/chat_window.rs`
- `assets/{index.html, dashboard-story-log.js, dashboard-action-area.js}`
- `tests/http/autoplay.rs` → `tests/http/autoplay/`, `tests/http/support/*`, `tests/browser/autoplay.rs`, `tests/browser/stub/autoplay.rs`, `tests/test_utils/stub_server.rs`
- `docs/specs/{autoplay.md, browser_autoplay.md}`, the Diátaxis docs of step 4, `CONTEXT.md`

## 5. Reuse

- `ScriptedModel::hold` / `ModelHold` (`src/test_support/scripted_model.rs`): the shape of the pause API.
- `JobTasks` in `JobDeps` (`src/application/generation/job_tasks.rs`): the precedent for a test-facing part wired in production.
- `Units::open` and the `Tx` methods (`gate.rs`, `storage/transaction.rs`): the config unit.
- `Storage::with_failure`, `StorageCall::StoriesTail`, `TestOverride::internal` (`storage/core.rs`, `storage/test_support.rs`), exemplar `tests/http/games_config.rs`.
- `wait_for_condition_async` (`tests/test_utils/wait.rs`).
- `hx-swap="morph:innerHTML"` as on `#story-log` (`assets/index.html`).
- `app_after_an_impersonate_with_a_failed_event` (`tests/http/support/app_wiring.rs`).

## 6. Verification

- At the end: `python build.py` (bash timeout 1200; read the newest `logs/build_*.log`).
- `scripts/validate_feature_spec.py`: every scenario has its tagged test.
- One mutation check per new or changed test.
- Coverage of `application/autoplay/` and the busy-state code in `gate.rs`: no uncovered branch of the loop's ends.
- Step 4: the UI pass, and the after-plan workflow with its 4 reviews.

## 6a. Failure modes

| New path | Failure | Handling |
|---|---|---|
| `Budget::parse` | not a number, 0, > 100 | `400` before any registry hold, on any Game |
| `reserve_for_autoplay` | Game busy | `200` busy answer (Q17) |
| `accept_for_autoplay` | story unreadable | existing `unreadable` refusal writes the failure line; loop ends `Refused`; Game idle (new test) |
| `accept_for_autoplay` | plan refusal (missing record) | existing `missing` refusal writes its line; loop ends `Refused` |
| `accept_for_autoplay` | holder gone (stop, reset, delete) | loop ends `Stopped`, writes nothing (pause-point test) |
| `Job::run_accepted` | step fails | the Job writes its failure line; loop ends `JobFailed` (41.4) |
| loop panics | — | `Drop for AutoplayRun` → `end_autoplay` removes the holder if the run id matches |
| `save_config` | Autoplay holds the Game | `Refusal::StillThinking` → `200` `BUSY_STATUS` (race only: the selects render disabled during a run) |
| `save_config` | write fails | `EngineError` → the existing `500` error fragment (`tests/http/games_config.rs`) |
| `AutoplayPause` | test forgets `release` | the hold's drop releases it, as `ModelHold` does |
| logs | — | one `tracing` line per cycle and per end, as now |

## 6b. Not in scope

- Per-tab Games (§1.2), the `autoplay_runs` table (Q8), the Narrator's role health after a Stop: tickets only.
- A cancel of the in-flight model call on Stop.
- Any change to plain Jobs, Scenario 6.4 or `tests/http/races.rs` beyond reusing `create_game_from` and `story_of_game`.
- A machine-readable busy signal (response header) and a posture re-render on a refused config write (owner: minimum scope, §2.4).
- The impersonate preset text (the owner judges it after the UI pass).

## 7. Past this plan

- Per-tab Games: the request carries its Game (route or header), `CurrentGame` reads it from the request, the client polls carry the Game id. Ticket.
- The Narrator's role health after a Stop (the stopped call's answer still counts). Ticket.
- An `autoplay_runs` history table for the evaluation of Autoplay output (reverses Q13/Q18 of the Autoplay plan). Ticket.
