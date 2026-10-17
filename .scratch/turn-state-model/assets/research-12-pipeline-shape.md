# Research 12: the shape of the generation pipeline

Ticket: [Research: survey the pipeline's shape for the fix-or-rewrite decision](../issues/12-research-pipeline-shape.md). Code at `d6a047cd` plus the uncommitted work tree of 2026-10-11 (17 modified files in `src/application` and `src/domain`, from the phase-2 ticket 15 work).

Labels: **known** = read in the code or in git at the time of writing. **inferred** = follows from the code, not run. **tested** = a test pins it. No fact here is newly tested. Symbols, not line numbers. Sizes are line counts from `wc -l` (known).

This asset reports facts. It does not recommend a fix or a rewrite. Vocabulary from `/codebase-design`: module, interface, depth, seam, adapter, locality.

Other assets this one uses, and does not repeat:

- [Research 04](research-04-generation-snapshots.md): the save sequence of each path, and the Snapshot names Sc, S0, S1, S2, S3, S4, S5, S6, Sf.
- [Research 05](research-05-game-state-writers.md): every writer of game state and the slot it holds.
- [Research 06](research-06-retry-tails.md): what each Retry form restores and re-runs.

## 1. Module map

### 1a. Modules and sizes (known)

| Module | Lines (non-test) | Lines (test file) | Holds |
|---|---|---|---|
| `pipeline/action_pipeline/core.rs` | 605 | 1397 | `ActionPipeline` struct and constructors; `claim_and_spawn`; `heal_stale_if_changed`, `heal_stale_status`, `reset_persisted_status`; `run_from_input` (164 lines); `retry_event_continuation` (65 lines); `retry_main_narration`; `finalize_phase_error`; `persist_generation_error`; `run_post_generation_agents`; `phase_engine_commit` |
| `pipeline/pipeline_run.rs` | 487 | — (tested through `core_tests.rs`) | `PipelineRun<'a>` (a borrowed `&ActionPipeline` plus `started_for: u64`) and its phase methods: `phase_pre_main_snapshot`, `call_narrator`, `phase_post_generation`, `phase_trigger_continuation_llm_call`, `reconcile_post_trigger_npcs`, `build_trigger_request` (71 lines), `phase_finalize`, `handle_cancellation`, `persist`, `persist_snapshot_or_err`, `check_game_unchanged`, preset and posture lookups |
| `pipeline/action_pipeline/retry.rs` | 394 | 2114 | `ActionPipeline::retry`, `retry_last_response` (104 lines), `RetryMode`, `RetryTarget`, `resolve_retry_target`, `reconstruct_retry_state`, `check_retry_anchor`, `retry_user_regen` (45 lines), `restore_click_time_state_after_failed_redo`, `durable_world_after_failed_redo` |
| `pipeline/action_pipeline/options.rs` | 223 | 365 | `process_options`, `execute_options_refresh` (65 lines), `run_options_generation`, `finalize_options_failure`, `rewrite_options_after_turn` |
| `pipeline/narration_generation.rs` | 174 | 386 | `NarrationGeneration::run`: world bundle load, room, preset, prompt, narrator call, `check_game_unchanged`, `add_message_with_inputs`, the "pre-quantifier narration" save |
| `pipeline/action_pipeline/action.rs` | 75 | 640 | `process_action` (parses the `Action`, adds the Input in `before_claim`), `execute_action_with_inputs` |
| `pipeline/action_pipeline/retrigger.rs` | 49 | 230 | `retrigger` (validation), `retrigger_event` |
| `pipeline/phase_error.rs` | 24 | — | `PhaseError` |
| `pipeline/action_pipeline/mod.rs`, `pipeline/mod.rs` | 34 | — | re-exports |
| `message_service.rs` | 273 | (18 tests) | `MessageService`: `load_or_fresh`, `load_expecting_valid_state`, `save_state`, `save_message_and_snapshot`, `save_arriving_location`, `build_fresh_initial_state`, `switch_swipe`, `edit_history`, `delete_last`, `find_retry_anchor_msg`, `find_retry_anchor` |
| `generation/gate.rs` | 145 | 154 | `GenerationGate`: registry `HashMap<u64, GenerationSlot>`, `heal_stale`, `try_claim`, `guard`, `release_generation_slot`, `release_generation_slot_for_game`, `is_busy` |
| `generation/guard.rs`, `generation/slot.rs` | 90 | 109 | `GenerationGuard` (releases on drop), `GenerationSlot`, `release_owned_slot` |
| `arrival_service.rs` | 190 | (3 tests) | `ArrivalTaskContext` (10 constructor fields), `run`, `run_inner` |
| `bootstrap/init_game.rs` | 176 | — | `load_game_state`, `try_persist_initial_message`, `spawn_arrival_task_if_needed` |
| `domain/model/state/game_state.rs` | 553 | 1840 | `push_message` (the `retry_target` Swipe append), `add_message`, `add_message_with_inputs`, `log_movement_completion`, `arriving_message_mut`, `handle_movement`, `apply_npc_events`, `commit_trigger_narration`, `execute_freeaction_impl`, `evaluate_triggers` |
| `domain/model/state/narrative_state.rs` | 63 | — | `NarrativeState`, with two `#[serde(skip)]` fields: `retry_target` and `arriving_location_write` |

Totals (known): the `pipeline/` folder is 2065 non-test lines. With `message_service.rs`, `generation/`, `arrival_service.rs` and `init_game.rs`, the turn code is 2952 non-test lines in the application and bootstrap layers. The pipeline test files hold 5132 lines and 124 test functions.

### 1b. Public seams and their callers (known)

| Seam | Visibility | Production callers |
|---|---|---|
| `ActionPipeline::process_action` | `pub` | `actions.rs` handler (`POST /action`). `Action::Options` routes on to `process_options` |
| `ActionPipeline::retry` | `pub` | `retry_handler` (`POST /swipe/new`) |
| `ActionPipeline::retrigger` | `pub` | `retrigger_handler` (`POST /retrigger`) |
| `ActionPipeline::heal_stale_status` | `pub` | `bootstrap::wiring` (boot), `games.rs` switch handler |
| `ActionPipeline::reset_persisted_status` | `pub` | `/status/reset-generating` handler |
| `ActionPipeline::recorder` | `pub` | `init_game::spawn_arrival_task_if_needed` (to build arrival) |
| `GenerationGate::is_busy` | `pub` | `chat_window.rs`, `endpoints.rs`, `debug.rs`, `app_state.rs` (options dock) |
| `GenerationGate::release_generation_slot_for_game` | `pub` | `/status/reset-generating` handler |
| `GenerationGate::try_claim`, `heal_stale` | `pub` | `ActionPipeline::claim_and_spawn`, `heal_stale_if_changed` only |
| `MessageService::switch_swipe`, `edit_history`, `delete_last` | `pub` | HTTP handlers, with no slot (research 05, §2b) |
| `MessageService::save_message_and_snapshot` | `pub` | 8 call lines: `try_claim`, arrival, 3 in `options.rs`, `finalize_phase_error`, `phase_post_generation`, `persist_snapshot_or_err` (research 05, §1) |
| `ArrivalTaskContext::new`, `run` | `pub`, `pub(crate)` | `spawn_arrival_task_if_needed` |

Known: the five entry-path methods behind `claim_and_spawn` (`execute_action_with_inputs`, `retry_last_response`, `retrigger_event`, `execute_options_refresh`, and `run_from_input`) are `pub` or `pub(crate)`, but the only production caller of each is the spawned closure of its own entry, or another entry method. They are wide for the tests (§5), not for production callers.

### 1c. Depth observations (known facts, `/codebase-design` terms)

- **`PipelineRun` is a bag of phases, not an ordering.** It has 17 methods. It owns no phase order. The order lives in five caller functions: `run_from_input`, `retry_event_continuation`, `retry_user_regen`, `execute_options_refresh`, and (outside the pipeline) `ArrivalTaskContext::run_inner`.
- **`ActionPipeline` is split by entry path, not by behaviour.** Five files carry `impl ActionPipeline` blocks (`core.rs`, `action.rs`, `retry.rs`, `retrigger.rs`, `options.rs`). The `inherent-impl-locality` guardrail requires the impls to stay in the `action_pipeline/` folder (`.scratch/_closed/inherent-impl-locality/`, ticket 04).
- **`NarrationGeneration` is the one deepened phase.** It hides bundle load, room, preset, prompt, narrator call, cancellation check, Message add and save behind `new(run, inputs).run(state) -> NarrationOutcome`. It was extracted on 2026-08-31 (`d03c337f`) by the guided-generations work.
- **`MessageService` mixes three concerns:** state load (`load_or_fresh`), the save-and-pointer rule (`save_message_and_snapshot`), and three player edits with no slot (`switch_swipe`, `edit_history`, `delete_last`). It also holds the Retry anchor rule (`find_retry_anchor_msg`).
- **`GenerationGate` writes game state.** `try_claim` sets `Generating`/`Narrating` and calls `save_message_and_snapshot` (the **Sc** save). `heal_stale` mutates the status, and its caller saves. So the gate is not only a registry.
- **Generation status is written by hand.** 33 non-test lines assign `input_buffer.status` or `input_buffer.phase`, in 7 files: `pipeline_run.rs` 12, `core.rs` 6, `gate.rs` 4, `retry.rs` 4, `arrival_service.rs` 3, `options.rs` 2, `message_service.rs` 2. The 2026-10-04 count in `architecture-deepening` ticket 09 was 39 lines in 8 files.
- **Two pieces of turn state live in memory only:** `retry_target` and `arriving_location_write` (`#[serde(skip)]`). The domain sets them (`push_message`, `log_movement_completion`), and `MessageService::save_message_and_snapshot` reads and clears them. So a pointer write depends on a domain field that the next save consumes.

## 2. Entry paths by phase

### 2a. The entry paths (known)

| # | Entry | Route | Claim | Spawned body | Ordering function |
|---|---|---|---|---|---|
| E1 | Free action with text | `POST /action` | `claim_and_spawn`; `before_claim` adds the Input | `execute_action_with_inputs` | `run_from_input` |
| E2 | Continue (`FreeAction("")`), Guide | `POST /action` | as E1, no Input | as E1 | `run_from_input` |
| E3 | Impersonate | `POST /action` | as E2 | as E1 | `run_from_input` |
| E4 | Main Retry, guided Retry | `POST /swipe/new` | `claim_and_spawn`; `pre_spawn` = `check_retry_anchor` | `retry_last_response` → `retry_main_narration` | `run_from_input` |
| E5 | Re-impersonate | `POST /swipe/new` | as E4 | `retry_last_response` → `retry_main_narration(state, "")` | `run_from_input` |
| E6 | Event Retry | `POST /swipe/new` | as E4 | `retry_last_response` → `retry_event_continuation` | `retry_event_continuation` |
| E7 | User regen | `POST /swipe/new` | as E4 | `retry_last_response` → `retry_user_regen` | `retry_user_regen` |
| E8 | Retrigger | `POST /retrigger` | `claim_and_spawn`; `before_claim` checks `last_trigger` and the last Message | `retrigger_event` → `retry_event_continuation` | `retry_event_continuation` |
| E9 | On-demand Options | `POST /action` with `/options` | `claim_and_spawn` | `execute_options_refresh` | `execute_options_refresh` |
| E10 | Arrival | boot, `spawn_arrival_task_if_needed` | **none** | `ArrivalTaskContext::run` | `run_inner` |

Five ordering functions serve ten entry paths (known).

### 2b. Matrix: which phase code each path runs

Columns: **Claim** = `claim_and_spawn` (heal, validate, `try_claim` → Sc, spawn with guard). **Pre** = `phase_pre_main_snapshot` (S0). **N** = `NarrationGeneration::run` (S1). **Q** = `phase_post_generation` (S2, Quantifier, S2b). **E** = `phase_engine_commit` → `execute_freeaction_impl`. **Tb** = `build_trigger_request`. **S3** = the "post-engine snapshot" call in `run_from_input`. **T** = `phase_trigger_continuation_llm_call` (S4, S5). **R** = `reconcile_post_trigger_npcs`. **Ap** = "`retry_target` → history" append. **O** = `rewrite_options_after_turn` (S6). **F** = `phase_finalize` (Sf). **Err** = how a failure is written. **RB** = `restore_click_time_state_after_failed_redo`.

Cell values: **S** = the shared phase method, called from the shared ordering function `run_from_input`. **s** = the shared phase method, called from a separate ordering function. **own** = the path has its own code for this step. **—** = the path does not run this step.

| Path | Claim | Pre | N | Q | E | Tb | S3 | T | R | Ap | O | F | Err | RB |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| E1–E3 Action | S | S | S | S | S | S | S | S | S | S | S (not E3) | S | `finalize_phase_error` | — |
| E4–E5 Main, guided, re-impersonate Retry | S | S | S | S | S | S | S | S | S | S | S (not E5) | S | `finalize_phase_error` | yes |
| E6 Event Retry | S | — | — | — | — | — | — | s (through `phase_trigger_continuation`) | s | own (second copy) | s | s | `finalize_phase_error` | yes |
| E7 User regen | S | own (status set by hand, **no S0 save**) | s | — | — | — | — | — | — | own (third copy) | — | s | `finalize_phase_error` | yes |
| E8 Retrigger | S | — | — | — | — | — | — | s | s | own (second copy, no target) | s | s | `finalize_phase_error` | — |
| E9 Options | S | own ("pre-options snapshot") | — | — | — | — | — | — | — | — | own (`run_options_generation`, not `rewrite_options_after_turn`) | s | `finalize_phase_error`, or `finalize_options_failure` (a System Message, then Idle) | — |
| E10 Arrival | **own: none** (status `Generating` set by hand, not saved) | — | **own** (`PromptContext::build_narration_prompt` + `recorder.complete`, not `PromptAssembler::assemble`) | — | — | — | — | — | — | — | — | own (status `Idle` or `Error` set by hand, one `save_message_and_snapshot`) | own (log only) | — |

All cells known from the code. Cross-check with research 06's phase table: the same phases, seen per Retry form there and per ordering function here.

### 2c. Where the paths duplicate (known)

- **The event tail exists twice.** `run_from_input` runs T → R → O → F inline after S3. `retry_event_continuation` runs T → R → Ap → O → F as its own sequence. Both call the same phase methods, but each has its own error and cancellation branches. The Ap step is in a different place: `run_from_input` appends after S3 and before T; `retry_event_continuation` appends after R.
- **The "`retry_target` → history" append exists three times:** `run_from_input`, `retry_event_continuation`, `retry_user_regen`.
- **The world bundle is loaded in three places:** `NarrationGeneration::run`, `retry_event_continuation`, `execute_options_refresh`, all through `ActionPipeline::load_world_bundle`. `MessageService::build_fresh_initial_state` calls `Storage::world_bundle_for` directly. Arrival loads the game, World and Persona with three `require_*` calls instead.
- **The narrator call exists twice.** `PipelineRun::call_narrator` uses `PromptAssembler::assemble` (storage-backed budget). Arrival uses `PromptContext::build_narration_prompt` with budget values read at spawn time. `architecture-deepening` ticket 10 records the same duplication.
- **The Options step exists twice.** `rewrite_options_after_turn` (turn end: clear, generate, save) and `execute_options_refresh` (on demand: keep the old set on failure). Both call `run_options_generation`.
- **There are five failure writers:** `finalize_phase_error` (held state, `save_message_and_snapshot`, or a fresh load when the state was consumed), `persist_generation_error` (fresh load, `save_state`), the error branch of `persist_snapshot_or_err` (held state, `save_state`), `finalize_options_failure` (System Message, then Idle), and arrival (status in its one save). `handle_cancellation` writes `Idle` through a sixth path (fresh load).

### 2d. Where the paths diverge (known; effect inferred where marked)

- **E7 skips the S0 save.** `retry_user_regen` sets `Generating`/`Narrating` in memory and goes to `NarrationGeneration::run` directly. Its first save in the task is S1. Every other narrating path saves S0 first.
- **E6 and E8 share one function with different inputs.** E6 enters with a restored Snapshot and a `retry_target`. E8 enters with `load_or_fresh` and no target. Inside `retry_event_continuation` the only difference is whether `retry_target.take()` returns `Some`. `push_message` then decides between a new Swipe on the target (E6) and a new event Message (E8).
- **E10 is not gated.** Research 05 §5 has the race. The `GenerationGate` registry never sees arrival, so `is_busy` is false during it (known).
- **The game id of a run is read in the task.** `run_from_input`, `retry_event_continuation`, `retry_user_regen` and `execute_options_refresh` each read `storage.current_game_id()` when the spawned task starts. `claim_and_spawn` holds the claimed `game_id` but passes it only to the guard. Inferred: a game switch between the claim and the task start gives `PipelineRun::started_for` the new game, and `check_game_unchanged` then cannot see the switch.

## 3. Where the Snapshot saves and pointer rules live

### 3a. Saves (known, counts from research 05 §1)

- 14 distinct save sites call `save_message_and_snapshot` or `persist_snapshot_or_err`. They sit in 7 files: `gate.rs` 1, `narration_generation.rs` 1, `pipeline_run.rs` 5 (4 labelled, 1 best-effort), `core.rs` 2 (S3, `finalize_phase_error`), `options.rs` 4 (one labelled, three direct), `arrival_service.rs` 1. (Research 05 counts the labelled `execute_options_refresh` call and the three direct calls in `options.rs` as four sites.)
- `MessageService::save_state` has 5 callers that write a Snapshot and set no pointer: `persist` (which `phase_finalize` uses), `reset_persisted_status`, `heal_stale_if_changed`, `persist_generation_error`, `restore_click_time_state_after_failed_redo`.
- `Storage::save_snapshot` is called directly by `switch_swipe`, `edit_history`, `delete_last`, `init_game::load_game_state` and `GameCatalogue::persist_initial_state_with_swipes`.

So Snapshot writes are spread across every phase. No module owns "which Snapshot is the result of this turn".

### 3b. The pointer rule (known)

The rule "a new Message or Swipe points at the next Snapshot saved" has one main home and two copies:

| Home | Used by |
|---|---|
| `MessageService::save_message_and_snapshot` | every pipeline save and arrival |
| `init_game::try_persist_initial_message` | boot, first Snapshot of a Game |
| `GameCatalogue::persist_initial_state_with_swipes` | `POST /games`, `POST /reset` |

But **which** Snapshot a pointer gets is not decided in any of them. It is decided by three things together:

1. where the domain creates the Message or Swipe: `GameState::push_message`, which uses `retry_target` (domain);
2. which save runs first after that: phase order, in the five ordering functions (application);
3. the rule "set only while `None`" (`save_message_and_snapshot`).

The location header follows a fourth path: `log_movement_completion` (domain) queues `arriving_location_write`, and the next `save_message_and_snapshot` applies it as a row update (research 04, R3).

Inferred: a change to "a Swipe points at its turn's result" (seed §1) must touch the phase order or the pointer rule, and each of the five ordering functions saves at a different point. Research 08 found the same spread for the location header.

## 4. History

### 4a. Churn (known, from `git log --follow`)

The repository has 714 commits from 2026-03-14 to 2026-10-11. 108 distinct commits touched the 13 turn files (`pipeline/**` non-test, `message_service.rs`, `generation/{gate,guard,slot}.rs`, `arrival_service.rs`). Caveat: `--follow` can match a similar older file, so 108 is an upper bound (inferred).

| Month (2026) | Commits |
|---|---|
| May | 20 |
| Jun | 14 |
| Jul | 33 |
| Aug | 24 |
| Sep | 10 |
| Oct (1–11) | 7 |

| Subject prefix | Commits |
|---|---|
| `refactor` | 63 |
| `feat` | 19 |
| `fix` | 11 |
| `docs` | 8 |
| other (`chore`, `test`, `inv`, `F2`, `Remove…`) | 7 |

Per file (commits, first commit date):

| File | Commits | Since 2026-08-11 | First |
|---|---|---|---|
| `action_pipeline/core.rs` | 75 | 15 | 2026-05-18 |
| `pipeline_run.rs` | 44 | 14 | 2026-06-23 |
| `arrival_service.rs` | 25 | 7 | 2026-07-05 |
| `message_service.rs` | 22 | 7 | 2026-07-09 |
| `generation/gate.rs` | 19 | 1 | 2026-07-09 |
| `action_pipeline/action.rs` | 15 | 15 | 2026-08-14 |
| `action_pipeline/retry.rs` | 9 | 9 | 2026-08-14 |
| `narration_generation.rs` | 8 | 8 | 2026-08-31 |
| `action_pipeline/retrigger.rs` | 3 | 3 | 2026-08-14 |
| `action_pipeline/options.rs` | 2 | 2 | 2026-09-11 |

### 4b. Structural reshapes (known, commit subjects)

The pipeline was reshaped about once a month:

| Date | Commit | Change |
|---|---|---|
| 2026-05-10 | `994138b6` | generation gate added (an `AtomicBool`) to stop action races |
| 2026-05-19 | `a5e71a4c` | `ActionPipelineBackend` trait extracted |
| 2026-05-27 | `c952f9b9` | ADR-019 retry pipeline refactor |
| 2026-05-30 | `2d36d240` | `find_retry_anchor` extracted; "split god service" |
| 2026-06-13 | `8cdae38e` | error model unified onto `GenerationStatus` in the state |
| 2026-06-23 | `2fd24857` | phase methods re-attached; snapshot bug fixed |
| 2026-07-02 | `8cdfc211` | `PipelineRun<'a>` extracted |
| 2026-07-05 | `22b6e53a` | arrival service extracted |
| 2026-07-09 | `56645a02` | `GenerationGate` extracted from the application service |
| 2026-07-10 | `3804594d` | per-game generation tracking |
| 2026-07-29 | `15071094` | retry module removed |
| 2026-08-03 | `1aa03781` | gate integrated into retry and retrigger |
| 2026-08-14 | — | `action.rs`, `retry.rs`, `retrigger.rs` split out again (inherent-impl locality) |
| 2026-08-15 | `b319579d` | heal before validation; `RetryTarget` extracted |
| 2026-08-20 | `b721622b` | guided generation and impersonate wired |
| 2026-08-31 | `d03c337f` | `narration_generation` extracted |
| 2026-09-06 | `45033a6e` | Swipe generation inputs flattened |
| 2026-09-11 | — | `options.rs` added |
| 2026-10-06 | `5fa46615` | heal moved into `claim_and_spawn` |

### 4c. Defects fixed in this code (known)

From commit messages and closed tickets. Each row names the root it shares with the seed, when one exists (inferred grouping).

| When | Where recorded | Defect | Root group |
|---|---|---|---|
| 2026-05-10 | `994138b6` | concurrent actions raced | writer rule |
| 2026-06-01 | `d61955d8` | status stuck on "Generating narration…" while the Quantifier ran | hand-written status |
| 2026-06-23 | `2fd24857` | a failed pre-main snapshot result was dropped | save discipline |
| 2026-07-12 | `c5ba1958` | `phase_narrate` did not resolve dynamic rooms (1 of 5 room lookups lacked the fallback) | duplicated lookups |
| 2026-09-06 | `7df55ec9` | a guided-turn Retry anchored on an older Input's Snapshot | pointer and anchor rule |
| 2026-10-04 | `8449a491`; `dashboard-ui-review` ticket 43 | a swipe switch restored a `Generating` Snapshot and locked the page | pointer set mid-turn |
| 2026-10-06 | `5fa46615`; `agent-workflow-friction` tickets 09, 11 | a stale `Generating` with no slot locked the page | writer rule, hand-written status |
| 2026-10-11 | `9b78b671`; `dashboard-ui-review-phase2` ticket 09 | event Retry always failed, and the failure rolled the game back | pointer set mid-turn; test fixture drifted from the real save order |
| 2026-10-11 | `9b78b671`; `dashboard-ui-review-phase2` ticket 10 | a failed Trigger wrote a raw error as a System Message | System Message channel |
| 2026-10-11 | work tree; `dashboard-ui-review-phase2` ticket 15 | the location header landed one Message late | pointer set mid-turn |

Still open (known; from the seed and research 04–08): the Swipe pointer before the Quantifier (seed §1); the re-impersonate spec conflict (seed §2); the arrival race (seed §3); `last_trigger` kept after a main Retry (research 04); the main-Retry anchor on the last Input row, which breaks continue and post-impersonate Retries (research 06); `check_retry_anchor` validates a pointer the event redo does not restore (research 07).

Six of the ten fixed defects and five of the six open defects come from when a pointer is set or who writes state (inferred grouping).

### 4d. Open plans and tickets on the same code (known)

- `.scratch/architecture-deepening/` (open map, no ticket resolved): ticket 04 "Consolidate the ActionPipeline turn lifecycle", ticket 05 "Split MessageService by concern", ticket 08 "Fold generation slot and guard into GenerationGate", ticket 09 "Give the input buffer named generation transitions", ticket 10 "Route arrival narration through the one narration-generation path". Each is a grilling ticket on a module this map also touches.
- `docs/plans/persistence-error-discipline-plan-revised.md` (status "Planned", 2026-08-12): make the best-effort S2b save in `phase_post_generation` a `PhaseError::PersistFailed`. The code still swallows it (known).
- `docs/plans/reliability-and-cancellation-plan.md` (status "Planning / Sub-plans pending", 2026-06-28).
- `docs/plans/t6-messagehistory-encapsulation.md` (status "Planning — ready", 2026-06-28).
- A prior decision on the pointer (commit `8449a491`, ticket 43): "Persisting the post-turn snapshot was rejected because a Swipe's snapshot is the retry anchor for guided / impersonate / user-regeneration redos." This is the same conflict the seed §1 table shows.

## 5. Seams for tests

### 5a. What the tests drive today (known)

| Tier | Seam it enters | Count |
|---|---|---|
| Unit (`src/**/*_tests.rs`) | `ActionPipeline` methods directly, with `MockBackend` and `Storage::new_in_memory()`. Calls: `run_from_input` 28, `retry_last_response` 26, `retry` 13, `execute_action_with_inputs` 21, `process_action` 10, `retry_user_regen` 3, `retrigger_event` 3, `retrigger` 3, `retry_event_continuation` 2, `retry_main_narration` 1 | 124 test functions in the pipeline test files; 18 in `message_service_tests.rs`; 10 in `generation/`; 3 in `arrival_service_tests.rs` |
| HTTP E2E, tier 1 (`tests/http/`) | the axum router, with `MockBackend`; waits with `wait_idle` | `actions.rs` 22, `swipe_new.rs` 30, `retrigger.rs` 9, `options.rs` 15, `swipe_switch.rs` 2 |
| Bootstrap smoke (`tests/bootstrap/`) | `bootstrap::run()` | — |

- **Known.** A unit test can drive one entry path without HTTP: it builds an `ActionPipeline` with `make_test_pipeline_with_mock_quantifier` and calls the spawned body (`run_from_input`, `retry_last_response`, `retrigger_event`) on the test thread. It then reads storage.
- **Known.** A test that goes through `claim_and_spawn` (`process_action`, `retry`, `retrigger`) must wait for the spawned task. `action_tests::wait_for_gate_idle` does this.
- **Known.** Tier 1 may assert only through read seams (`tests/STRATEGY.md`, "Domain outcome"). A Snapshot pointer has no read seam, so a pointer fact is a unit or driven-adapter assertion by that rule.
- **Known.** The harness never runs arrival through HTTP (seed §3). `arrival_service_tests.rs` has 3 unit tests that call `ArrivalTaskContext::run` directly.

### 5b. Hand-copied save order in fixtures (known)

Many Retry tests do not run a real turn first. They seed Messages, Swipes and Snapshots by hand in an assumed save order:

- `fixtures::seed_main_narration_flow` and `seed_event_flow` (used in 9 places), plus `seed_swipe_with_stored_inputs`;
- `retry_tests.rs` helpers `add_input_and_save`, `add_narration_and_save`, `save_pre_main`, `save_pre_event`, `insert_message_with_swipe`.

These helpers call `Storage::save_snapshot` and set pointers themselves. They do not go through `save_message_and_snapshot`. The phase-2 ticket 09 defect passed its tier-1 test because `seed_event_flow` linked a Snapshot that the real pipeline never links. The comment on `seed_main_narration_flow` still says the Input links to the "pre-main" Snapshot; research 04 found that the Input links to **Sc** (the claim save). So the fixture order and the real order still differ (known).

### 5c. Where a model-level test could sit (inferred)

- **No pure turn model exists.** The domain has pure steps (`execute_freeaction_impl`, `commit_trigger_narration`, `push_message`), but the turn states (rewind point, result) and the pointer timing exist only as the interleaving of phase calls, LLM calls and saves in the five ordering functions. A model-level test today can only observe them through storage.
- **The nearest existing seam** is the unit tier: an `ActionPipeline` on in-memory `Storage` with `MockBackend`, driven through a real entry body, then read back through `MessageService::load_messages` and `Storage::load_snapshot_by_id`. This seam has two real adapters behind `Storage` (`Backend` in-memory and SQLite), so it is a real seam in `/codebase-design` terms.
- **The seam a model test would want** is one interface that answers "after this entry, which Snapshot is each Message's and Swipe's pointer, and what does that Snapshot hold". No module offers it. The facts are split across `GameState::push_message` (domain), the five ordering functions (application) and `MessageService::save_message_and_snapshot` (application).

## 6. Facts for the "Fix or rewrite" decision

A list, with no weighting. Each line points to its section.

- Ten entry paths, five ordering functions, one shared narration phase (`NarrationGeneration`). §2a, §1c.
- `run_from_input` (164 lines) serves six of the ten paths. §2a.
- The event tail is written twice, and the target append three times. §2c.
- Arrival re-implements the narration phase and runs with no slot. §2b, §2d.
- 14 pointer-capable save sites in 7 files, plus 10 pointer-free Snapshot writers. The pointer rule has one main home, but the pointer timing is set by phase order. §3.
- 33 hand-written status lines in 7 files. §1c.
- 108 commits on these files in 5 months; 63 of them refactors; a structural reshape about once a month. §4a, §4b.
- 10 fixed and 6 open defects in this code; most of them share the pointer-timing or writer root. §4c.
- Five open `architecture-deepening` tickets already propose deepening the same modules. §4d.
- The unit tier can drive each entry path without HTTP. Retry tests seed the save order by hand, and that hand order has drifted from the real one. §5.
