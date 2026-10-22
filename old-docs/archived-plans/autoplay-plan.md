# Build plan: Autoplay

## How to use this plan

This plan builds Autoplay in two slices. Slice 1 is the engine and HTTP. Slice 2 is the dashboard. The slices run in order, and each slice is one branch from the current `main`: `autoplay-engine` and `autoplay-dashboard`.

The implementer of a slice reads §1 to §3 and the section of that slice. The reviewer reads the same sections.

Every design decision comes from the grilling session of 2026-10-17 with the owner, and from the plan review after it. The decision ids (Q1 to Q20) refer to the table in §1. A gap that needs a decision stops the slice and goes to the owner. Do not decide it yourself.

Facts with no label are known: read in the code at commit `d417f446`. **Inferred** facts are labelled. Check them before you rely on them.

## 1. What Autoplay is, and the decisions

Autoplay lets the engine play the Persona by itself. A run starts ordinary Jobs, one after the other: an `/impersonate` Job, then a Continue Job that answers it, until the run stops. Autoplay is a client of Jobs, as the player is. Autoplay has two goals: a game mode to watch, and a source of long Games for the evaluation of LLM features. The evaluation output is the Game's Message history in the database.

| Id | Decision |
|---|---|
| Q1 | One driver in the application layer. The dashboard is one driving adapter. A headless runner can be a second adapter later |
| Q2 | Impersonate writes each player Input, with the Game's impersonate preset |
| Q3, Q16 | A run stops for one of three reasons: the player presses Stop, the cycle budget runs out, or a Job ends with a failure line. A failed Options run (a Notice) and a Quantifier fallback do not stop it |
| Q4, Q18 | The run is transient. It lives only in the server process. A restart ends it, and the engine writes nothing about it |
| Q5 | The output is the Message history. Judging the output is a later, separate feature |
| Q7 | While a run holds the Game, the existing Stop ends the run and also cuts off the running Job, as it does today |
| Q8 | A run belongs to its Game, not to the active-Game pointer. It keeps running when the player switches Games |
| Q9 | The headless runner is deferred. The driver must not depend on the HTTP layer |
| Q10 | One cycle is one Impersonate Job plus the Continue Job that answers it. Event Narrations that a Trigger fires in the Continue Job are part of the cycle. Each Job runs its Options run as today |
| Q11 | No standing direction. The Persona is the only steering |
| Q12 | The start needs a cycle budget. The default is 10. The maximum is 100; a larger value answers `400` (owner, slice-1 review). There is no "unlimited" value |
| Q13 | No marker in the story. Autoplay Inputs are Impersonate Inputs, like the Inputs of `/impersonate` |
| Q14 | While a run holds the Game, the Game is view-only. The status shows "Autoplay: cycle n/N" |
| Q15 | Moot. The engine writes no System Messages since the Rust rebuild |
| Q17 | A start on a busy or held Game is refused with "Still thinking..." |
| Q19 | No limit on parallel runs across Games |
| Q20 | A run is **a series of ordinary Jobs under a run hold**. The run hold keeps the Game reserved between two Jobs of the run. The Job is unchanged (plan review, replaces the earlier "one long Job") |
| Q21 | A cycle ends when a Job leaves the story on a Narration: the Input is answered by a Continue or by a Trigger's event Narration (owner, slice 2). The budget counts these answered Inputs |
| Q22 | A normal Job gets no Stop button. Only a run holds the Stop control (owner, slice 2) |
| Config | While a run holds the Game, the config writes are refused (§5). A normal Job does not refuse them, so Scenario 6.4 stays |

**The budget counts answered Inputs.** A cycle ends when a Job of the run ends without a failure line and leaves the story on a Narration (Q21). The budget counts these cycles. It has no special rule for the first cycle (owner, Q10 follow-up).

## 2. The facts the design rests on

- `JobPlan::for_request` in `domain/model/generation.rs` is a pure function of the request and the end of the story. `JobRequest::Impersonate(None)` and `JobRequest::Action("")` (a Continue) already have plans.
- `Job::start` in `application/pipeline/job.rs` accepts the request through `GameWriter::accept`, spawns the Job with `tokio::task::spawn_blocking`, and returns at once. `JobTasks` watches the spawned task, so a tier-1 test can wait for it.
- `Job::run` ends through `Job::end`. Today it returns nothing, so a caller cannot see how the Job ended.
- `GameWriter` in `application/generation/gate.rs` keeps one job entry per busy Game in one registry. `require_idle` refuses every story write while the entry exists: an accept, a swipe switch, an Edit and a Delete.
- `GameWriter::accept` clears the failure line in its accept unit. So a driver that reads the failure line after the next accept reads nothing. The driver must learn the end of a Job from the Job itself.
- `GameWriter::stop`, `replace` (the reset) and `delete` remove the job entry. The Job's next step or save is refused (`JobFailure::EntryMoved`), and the Job ends.
- A failed Options agent writes a Notice and is not a Job failure: `options_step` returns `Ok` with `OptionsResult::Unavailable`.
- An Impersonate Job runs no Options run after it: `Kind::saves_a_narration` is false for an Impersonate.
- A Continue after an Impersonate Input answers that Input. When a Narration request has no input text, `Narrator::user_message` uses the Input that ends the history window as the `<PlayerInput>`. Slice 1 still pins this with a `ScriptedModel` call capture.
- A fresh Game starts with an opening Narration (`Maker::WorldOpening`). So a run on a fresh Game starts with Impersonate.
- `GET /status/generating` answers with the bare step name while a Job runs (Scenario 39.1). `assets/dashboard-action-area.js` maps the name to the status text.
- The config writes of `GameCatalogue` are not under the busy rule: the Narrator Mode switch (`POST /games/{id}/mode`), the posture (`POST /games/{id}/posture`) and the preset selections (`POST /games/{id}/presets`). Scenario 6.4 (`docs/specs/actions.md`, test in `tests/http/races.rs`) requires that a preset change during a normal Job applies to the Job's next model call.
- The always-on toggle has no per-Game write route. The World holds it, and a Game copies it at creation.
- **Inferred:** the dashboard disables only the dock buttons while a Game is busy. The story-log controls (Retry, swipe, Edit, Delete, Retrigger) stay visible, and the server refuses their requests. Slice 2 checks this.

## 3. The design

```text
Run R holds the Game ───────────────────────────────────────────┐
├ Job Impersonate ┐ held ┌ Job Continue ┐ held ┌ Job Impersonate ...
└ Stop / budget / failure line ends R ─────────────────────────┘
```

### 3.1 The run hold

- The `GameWriter` registry gains a **run hold** per Game, next to the job entries and under the same lock. A run hold has its run id, the cycles done and the budget.
- While a run hold exists, every write that `require_idle` guards is refused with "Still thinking...", also between two Jobs of the run. Only an accept that names the run's own id passes.
- `stop`, `replace` and `delete` remove the run hold with the job entry, in the same hold.
- The start (`GameWriter::start_run` or similar) checks that the Game has no job entry and no run hold, and then makes the run hold. A budget of 0 is a bad request (`400`).
- The Game is **busy** while a job entry or a run hold exists. `GameViewQuery::status` and `is_busy` read both. While the run hold exists, the status carries the progress, with the step when a Job runs and without it between two Jobs.

### 3.2 The next request

- One pure rule in `domain/model/generation.rs` gives the next request of a run from the end of the story. When the last Message is an Input (typed or Impersonate), the next request is a Continue (`JobRequest::Action("")`). In all other cases, the next request is `JobRequest::Impersonate(None)`. This one rule is also the start rule of Q10.

### 3.3 The driver

- A new driver in `application/autoplay/` runs the run. It is one blocking task, watched by `JobTasks`, so a tier-1 test waits for the whole run.
- Each loop step: read the end of the story, ask the rule of §3.2 for the next request, accept it for the run, and run the Job to its end in the driver's own task. The driver does not start a second Job before the first ends.
- `Job` gives the driver a way to run one accepted Job and return how it ended: done, a failure line, or the entry moved. `Job::start` keeps its behaviour for every other caller. The implementer chooses the split, for example an accept-and-run function next to `Job::start`.
- After a Job that ended done, the driver asks the run hold to end the cycle. The run hold reads the end of the story in the same registry hold. When the last Message is an Input, it counts nothing and the run goes on. Otherwise it adds one to the progress. A story that cannot be read ends the run with the failure line of the story.
- The run ends, and the driver removes the run hold, when:
  - the budget is reached;
  - a Job ended with a failure line. The line stays on the Game, because no later accept clears it;
  - an accept for the run is refused. The refusal writes its failure line, as today;
  - the run hold is gone (a stop, a reset or a delete). The driver ends without a write.
- Each Job writes its own records, its Options run and its failure line as today. The driver writes only the run hold.
- Each cycle writes one `tracing` line with the Game, the run id, the cycle number and the budget. The end of the run writes one line with the end reason. These lines are the only record of the end reason, because the story has no run marker (Q13).

### 3.4 The HTTP start

- Add `POST /autoplay` with a `cycles` form field. It starts the run on the request's Game through `CurrentGame`. It answers at once, like the other Job starts: the status fragment, or "Still thinking..." on a busy Game (Q17).
- Stop uses the existing `POST /status/reset-generating`. No new stop route.
- **Inferred:** a run holds one blocking thread for its whole run. The runtime's default limit of blocking threads is far above the number of parallel runs that one player starts (Q19).

## 4. Slice 1: the engine and HTTP

**Branch:** `autoplay-engine`.

**Content.**

1. The run hold in the `GameWriter` registry, and its effect on `require_idle`, `stop`, `replace`, `delete`, `status` and `is_busy` (§3.1).
2. The next-request rule (§3.2).
3. The run-one-Job function of `Job` and the driver (§3.3).
4. The progress in `GET /status/generating` (§3.1).
5. `POST /autoplay` (§3.4).
6. The config writes under the run hold (§5).

**Spec first.** Write `docs/specs/autoplay.md`, Feature 41, before the tests. It needs at least these scenarios:

- A run on a fresh Game with a budget of 2 saves Impersonate Input, Narration, Impersonate Input, Narration, and the Game is then idle.
- A run whose last Message is an unanswered Input starts with a Continue. That Continue counts as cycle 1.
- A Trigger that fires inside a Continue Job adds its event Narration to the cycle. The budget counts the answered Inputs. A Trigger that fires in an Impersonate Job answers its Input with the event Narration and ends the cycle.
- A failed narrator call ends the run, and the failure line stays on the Game. No later Job starts.
- A failed Options run writes its Notice, and the run continues.
- A stop during a Job of a run ends the run. The Job's late answer changes nothing (as Scenario 39.5).
- A stop between two Jobs of a run ends the run, and no next Job starts.
- A start on a busy Game, and on a Game that a run holds, answers "Still thinking...".
- A budget of 0 answers `400`.
- While a run holds the Game, also between two of its Jobs, a typed action, a Retry, a swipe switch, an Edit and a Delete answer "Still thinking...".
- While a run holds the Game, the mode switch, the posture change and the preset change answer "Still thinking...", and the Game's config is unchanged.
- The status poll answers with the progress while a run holds the Game. Write the exact answer here.
- A run continues when the player switches the active Game.
- A reset of the Game ends its run.

**Tests.** One tier-1 HTTP test per scenario with `ScriptedModel`, in the `http` test binary (`tests/http/autoplay.rs`). Use `ScriptedModel::hold` for the stop and busy scenarios. The between-Jobs scenarios need a way to stop the driver between two Jobs; if `ScriptedModel::hold` cannot reach that point, the implementer stops and asks. The next-request rule is a pure function in `domain/model/`, so it gets a plain unit test. Scenario 6.4 and its test stay unchanged.

**Docs.**

- `docs/diataxis/reference/game_flow.md`, section "The Job Entry and Busy": a Game can also have a run hold; while a run hold exists, only the run's Jobs are accepted; a stop, a reset and a delete remove it. The Job sections stay unchanged.
- `CONTEXT.md` stays unchanged (owner).
- `docs/diataxis/reference/frontend/http_routes.md` regenerates from `router.rs`.

**Done when.**

1. Every scenario of Feature 41 has its test, and `scripts/validate_feature_spec.py` passes. The tests use `ScriptedModel` only. Slice 1 has no real-LLM check, because the output of the model does not change the code under test (owner, plan review).
2. The commit gate of `/simple-after-plan` passes, with `python build.py` green.
3. The owner approves the commit.

## 5. The config writes during a run

**Decision (owner, 2026-10-17, plan review):** while a run hold exists, a config write is refused with `200` "Still thinking...", and the config is unchanged. The config writes are the Narrator Mode switch, the posture and the preset selections. A normal Job does not refuse them, so Scenario 6.4 stays.

This replaces the earlier choice (refuse during any Job). That choice reversed Scenario 6.4, which was written on purpose, and it was made before the plan review found that scenario.

The rename of a Game (`POST /games/{id}/rename`) changes only its display name, not the story, so it stays open during a run.

**Constraint (inferred):** the run-hold check and the config write must happen in one hold of the registry. Otherwise a run can start between the check and the write. Today only `GameWriter` holds the registry, so the config writes must go through a `GameWriter` hold. The implementer chooses how, and keeps `GameCatalogue` as the owner of the config columns where possible.

## 6. Slice 2: the dashboard

**Branch:** `autoplay-engine`, the same branch as slice 1, on top of its commit (owner, 2026-10-21).

**Content.**

1. An Autoplay control beside the command form: a budget field with the default 10 and a start button (Q12).
2. While a run holds the Game, the status display shows "Autoplay: cycle n/N" with the step. The command form shows only Stop (Q7, Q14).
3. While a run holds the Game, the story-log controls that change the story are disabled or hidden: Retry, swipe switch, Edit, Delete, Retrigger, and the dock buttons. Reading, scrolling and the Game switch stay open (Q14). Check the inferred fact of §2 first.
4. While a run holds the Game, the config controls (Narrator Mode, posture, presets) are disabled. The server already refuses them (§5).
5. The Games list marks a Game that a run holds (Q8).

**Spec first.** Write `docs/specs/browser_autoplay.md`, Feature 42. It needs scenarios for the start with the default budget, the progress status, the view-only controls, Stop, and the Games list mark.

**Tests.** One browser test per scenario in the `browser` test binary.

**Docs.** `docs/diataxis/reference/frontend/dashboard.md` names the Autoplay control and the view-only rule, if the doc lists the controls of the command form. The slice checks.

**Done when.**

1. The same three checks as slice 1, with Feature 42.
2. A `/chronicler-ui-investigator` pass (`.agents/skills/chronicler-ui-investigator/SKILL.md`) with the real connections that the settings hold, on its own probe server and database. The pass starts a run of 3 cycles from the dashboard control on a fresh Game, and captures shots of: the progress status during the run, the view-only controls, the story after the run, and a Stop in the middle of a second run. It also reads the story in the probe database and checks that each Narration answers the Impersonate Input before it. A result that shows a weak impersonate preset (the Persona repeats itself or narrates) goes to the owner. It does not block the slice, because the fix is preset text (owner, plan review: the real-LLM check is this pass, not a step of slice 1).

## 7. Past this plan

- The headless runner (Q9): a CLI subcommand or an external script. It uses the driver of §3.3. The owner decides after use of the dashboard control.
- The judging of Autoplay output (Q5): transcript review, an LLM judge, or scripted probes.
- Engine-wide config changes during a run (a preset's text, a connection, the response length) still apply to the next Job of the run. The run hold covers only the Game's own config.
- The dead `MessageType::System` variant: no code writes it. Its removal is a separate cleanup.
