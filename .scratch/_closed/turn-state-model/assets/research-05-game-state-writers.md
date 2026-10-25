# Research 05: the game-state writers and the slot each holds

Ticket: [Research: list the game-state writers and the slot each holds](../issues/05-research-game-state-writers.md). Code at `d6a047cd` plus the uncommitted work tree of 2026-10-11.

Labels: **known** = read in the code. **inferred** = follows from the code, not run. **tested** = a test pins it.

## 0. Ground facts

- **Known.** `GenerationStatus` is not on the Game row. It lives only in the Snapshot (`narrative.input_buffer.status` and `.phase`). Every Snapshot write is a status write. No storage call writes status by itself.
- **Known.** Every Snapshot, Message and Swipe write targets `Storage::game_id()`, read at the time of each storage call (`save_snapshot`, `insert_message`, `delete_message`, …). The game id is one process-global atomic, set by `Storage::set_game_id`. No writer passes a game id.
- **Known.** Each storage call locks the backend by itself. `save_message_and_snapshot` is up to five storage calls (`update_swipe_location_header`, `save_snapshot`, `insert_swipe`, `update_active_swipe`, `insert_message`) with no transaction around them.
- **Known.** A Snapshot write saves the whole state copy the writer holds (movement, scene, narrative minus history rows, status). History comes back from the Message rows on load (`MessageService::load_expecting_valid_state`). A stale copy therefore overwrites every Snapshot field, but it cannot remove Message rows. It only adds the unpersisted Messages it holds.
- **Known.** The `GenerationGate` registry is keyed by game id. A slot is claimed by `GenerationGate::try_claim`, inside `ActionPipeline::claim_and_spawn` only. It is released by `GenerationGuard::drop` (end of the spawned task), by `release_generation_slot` (a failed `pre_spawn`), or by `release_generation_slot_for_game` (`/status/reset-generating`).

## 1. Count reconciliation

| Symbol | Call lines outside the definitions |
|---|---|
| `persist_snapshot_or_err` | 7: `NarrationGeneration::run` ("pre-quantifier narration"), `execute_options_refresh` ("pre-options snapshot"), `run_from_input` ("post-engine snapshot"), `phase_post_generation` ("pre-quantifier phase update"), `phase_trigger_continuation_llm_call` ("pre-event snapshot", "post-trigger snapshot"), `phase_pre_main_snapshot` ("pre-main snapshot") |
| `save_message_and_snapshot` | 8: `GenerationGate::try_claim`, `ArrivalTaskContext::run_inner`, `execute_options_refresh` (options set), `finalize_options_failure`, `rewrite_options_after_turn`, `finalize_phase_error`, `phase_post_generation` (post-quantifier metadata), and the body of `persist_snapshot_or_err` |

- **Known.** 15 call lines, as the 2026-10-11 grep found. One of them is the body of the wrapper `persist_snapshot_or_err`, so there are 14 distinct save sites.
- **Inferred.** The seed's 17 is the 15 call lines plus the 2 definition lines.
- **Known.** These two symbols are not the full writer set. `MessageService::save_state` (5 callers), direct `Storage::save_snapshot` (6 callers), and the Message/Swipe row writers also write game state. Section 2 lists all of them.

## 2. The writers

Thread model: **handler** = an axum async handler that calls sync storage on the tokio worker thread. **spawned** = the closure that `claim_and_spawn` runs under `tokio::task::spawn_blocking`, with a `GenerationGuard`. **arrival task** = `runtime.spawn_blocking` from `spawn_arrival_task_if_needed`. **boot** = the main thread before the server starts.

State source: **fresh** = loaded from storage just before the write, with no LLM call between. **held across LLM** = a copy loaded before one or more LLM calls and saved after them. **pre-claim copy** = loaded in the handler before `try_claim`.

### 2a. Pipeline writers (slot held)

| Writer (symbol, label) | Entry point | Thread | Slot held | State source | Writes |
|---|---|---|---|---|---|
| `GenerationGate::try_claim` | `/action`, `/swipe/new`, `/retrigger`, `/action` Options, through `claim_and_spawn` | handler | yes (inserted just before the save) | pre-claim copy | Snapshot (`Generating`, `Narrating`); the new Input row for a free action |
| `heal_stale_if_changed` | `claim_and_spawn` (every gated entry) | handler | **no** (runs before `try_claim`) | pre-claim copy | Snapshot with `Idle`, only when it heals |
| `check_retry_anchor` → `persist_generation_error` | `/swipe/new` (`pre_spawn`) | handler | yes (then released by `claim_and_spawn`) | fresh | Snapshot with `Error` |
| `phase_pre_main_snapshot` ("pre-main snapshot") | `run_from_input` (action, main retry, re-impersonate) | spawned | yes | fresh (action: `execute_action_with_inputs` reloads; retry: rebuilt from the anchor Snapshot) | Snapshot; unpersisted Messages |
| `NarrationGeneration::run` ("pre-quantifier narration") | `run_from_input`, `retry_user_regen` | spawned | yes | held across LLM (narrator) | Snapshot; Narration row + Swipe; or a retry Swipe on the target |
| `phase_post_generation` ("pre-quantifier phase update", then post-quantifier metadata) | `run_from_input` | spawned | yes | held across LLM (narrator; the second save also across the quantifier) | Snapshot; the `[System]` NPC-uncertain row |
| `run_from_input` ("post-engine snapshot") | `run_from_input` | spawned | yes | held across LLM (narrator, quantifier) | Snapshot; `update_swipe_location_header` |
| `phase_trigger_continuation_llm_call` ("pre-event", "post-trigger") | `run_from_input`, `retry_event_continuation` (event retry, Retrigger) | spawned | yes | held across LLM | Snapshot; event Narration row |
| `rewrite_options_after_turn` | `run_from_input` (non-impersonate), `retry_event_continuation` | spawned | yes | held across LLM (adds the options call) | Snapshot; `[System]` rows |
| `execute_options_refresh` and `finalize_options_failure` | `/action` Options | spawned | yes | fresh, then held across the options LLM call | Snapshot; `[System]` row |
| `phase_finalize` (`PipelineRun::persist`) | every spawned path | spawned | yes (the guard drops after it) | held across LLM | Snapshot with `Idle` |
| `finalize_phase_error` | every spawned path | spawned | yes | held across LLM, or fresh through `persist_generation_error` when the state was consumed | Snapshot with `Error`; unpersisted rows |
| `persist_snapshot_or_err` error branch (`persist`) | same | spawned | yes | held across LLM | Snapshot with `Error` |
| `retry_last_response` → `persist_generation_error` | `/swipe/new` | spawned | yes | fresh | Snapshot with `Error` |
| `restore_click_time_state_after_failed_redo` | `/swipe/new` | spawned | yes | click-time Snapshot, held across the whole redo | Snapshot; checks `current_game_id` first |
| `PipelineRun::handle_cancellation` | any spawned path whose `check_game_unchanged` fails | spawned | the slot of the **old** game | fresh load of the **new** current game | Snapshot with `Idle` into the new current game |

- **Known.** `MessageService::save_message_and_snapshot` also calls `update_swipe_location_header`, `insert_swipe` and `update_active_swipe`. So every row in 2a can write those rows, not only the labelled ones.

### 2b. Writers with no slot

| Writer | Entry point | Thread | Gate check | State source | Writes |
|---|---|---|---|---|---|
| `ArrivalTaskContext::run_inner` | boot, `spawn_arrival_task_if_needed` | arrival task | none | held across LLM (loaded at task start) | Snapshot (`Idle` or `Error`); arrival Narration row + Swipe |
| `MessageService::switch_swipe` | `POST /message/:id/swipe/:index` | handler | reads `is_busy` before; claims nothing | Snapshot of the chosen Swipe | `update_active_swipe`; Snapshot with `Idle` |
| `MessageService::edit_history` | `POST /history/:id` | handler | none | fresh | Snapshot; `update_swipe_text` |
| `MessageService::delete_last` | `POST /history/delete` | handler | none | fresh | Snapshot; `delete_message` |
| `ActionPipeline::reset_persisted_status` | `/status/reset-generating` | handler | releases the slot first, does not stop the task | fresh | Snapshot with `Idle` |
| `ActionPipeline::heal_stale_status` | `/games/:id/switch`; boot (`build_app_graph`) | handler; boot | reads the registry | fresh | Snapshot with `Idle`, when it heals |
| `GameCatalogue::reset` | `POST /reset` | handler | none | none (new Game) | deletes the current Game; new Game, Snapshot, scenario row; `set_game_id` |
| `GameCatalogue::create_game` | `POST /games` | handler | none | none (new Game) | new Game, Snapshot, scenario row; `set_game_id` |
| `GameCatalogue::switch_game` | `/games/:id/switch` | handler | none | — | `set_game_id` (retargets every later write of every writer) |
| `init_game::load_game_state` | boot | boot | — | none (first Snapshot) | Snapshot; scenario row |

- **Known.** Game-config writers (`set_posture`, `switch_mode`, `set_preset_selection`, `rename_game`) and World writers write no Snapshot, Message or Swipe. `delete_world` refuses a World that has Games. `delete_game` refuses the current Game. The debug routes only read.
- **Known.** The status display (`generating_status_handler`) and `switch_swipe` read the registry, so they report idle while arrival runs.

## 3. Concurrent pairs that can lose an update

Each pair names the writer that loses and what is lost.

1. **Arrival vs a turn** (seed §3). Arrival saves its boot-time copy after its LLM call. If that save lands after the turn's last save, the latest Snapshot goes back to the boot state: the turn's movement, scene, options and `last_trigger` are lost. The arrival Narration row gets a higher id than the turn's rows, so it shows after the turn. If the save lands mid-turn, the turn's next save overwrites it, and only the Narration order is wrong. Mechanism **known**; race **inferred**; no test.
2. **A pipeline vs a game switch, reset or create.** All three call `set_game_id`. The spawned task keeps its slot on the old game id, and its later writes land in the new current game. `check_game_unchanged` runs only after the narrator call, around the trigger call, and once in `execute_options_refresh` before its LLM call. No check runs after the quantifier call or the options call. So the post-quantifier, post-engine, options-rewrite and finalize saves can write the old game's state into the new game. The new game's slot is free, so a turn can run there at the same time. Mechanism **known**; consequence **inferred**; no test.
3. **`handle_cancellation` writes into the new game.** On a detected switch it loads the new current game, sets `Idle`, and saves it. If the new game has a live turn, its persisted `Generating` becomes `Idle` mid-turn. The old game keeps a persisted `Generating`, which a later heal repairs. **Known**; `test_retry_user_regen_cancelled` and `test_retry_last_response_cancelled_at_phase_boundary` (`retry_tests.rs`) assert `Idle` on the game the flipper switched to, so they pin this write (**tested**, though the tests frame it as "the cancelled retry resets status").
4. **A split save across two games.** `Storage::game_id()` is read per storage call, so a switch inside `save_message_and_snapshot` can put the Snapshot in one game and the rows in another. **Inferred**; narrow window.
5. **`/status/reset-generating` vs the running task.** The handler frees the slot but the task runs on. A second claim then succeeds, and two spawned tasks write full-state copies of one game. Each overwrites the other's Snapshot. **Known** mechanism; race **inferred**.
6. **Pre-claim copy vs the finishing turn.** `claim_and_spawn` loads state before it claims. If turn T1 saves its last Snapshot and drops its guard between that load and `try_claim`, the new claim saves a copy older than T1's end. The spawned task then reloads that older Snapshot as its base. T1's post-engine state, options and `Idle` are lost. **Inferred**; window = the handler's time from load to claim.
7. **`heal_stale_if_changed` vs the finishing turn.** Same window as 6: the handler loads `Generating`, T1 finishes and drops its guard, heal sees no slot, and saves the whole stale copy with `Idle`. **Inferred.**
8. **`edit_history` or `delete_last` vs a running turn.** No gate check. Each saves a fresh full copy. If a pipeline save lands between their load and save, their older copy becomes the latest Snapshot until the pipeline's next save. After the pipeline's last save this loses its finalize (`Idle`, options, movement). `delete_last` can also delete the running turn's Input row. **Known** that no check exists; loss **inferred**.
9. **`switch_swipe` vs a new claim.** It checks `is_busy` and then writes; a claim can land between them. The switch's restored `Idle` Snapshot can then become the base that the new turn's spawned task reloads. **Inferred**; narrow window.

## 4. What `heal_stale_if_changed` assumes

`GenerationGate::heal_stale` resets the persisted status to `Idle` when it is `Generating` (`GenerationStatus::is_generating`) and the registry has no live slot for the current game id. `heal_stale_if_changed` then saves the whole copy it was given.

| Assumption | Holds? | Breaks when |
|---|---|---|
| A1. Every live writer that persists `Generating` holds the slot of the game it writes to. | No | Pair 2 (a pipeline writes `Generating` into a game whose slot it does not hold); pair 5 (the slot was freed under a live task). |
| A2. A writer with no slot never persists `Generating`. | Yes, today | Arrival sets `Generating` only in memory; its one save is `Idle` or `Error` (**known**). No 2b writer persists `Generating`. |
| A3. No slot means the generation ended, so the persisted status is final or stale. | Mostly | Pair 5, and pair 2 (seen from the new game). Arrival is outside this: it has no slot and no persisted status. |
| A4. The copy it saves is current, so saving the whole copy only changes the status. | No | Pair 7: the copy was loaded before the slot freed. Boot heal and switch heal load fresh, so they hold. |
| A5. The registry key and the write target are the same game. | Only while no `set_game_id` runs | Pair 2, pair 4. |

- **Known.** `phase_finalize` saves before the guard drops, so a normal end never leaves `Generating` without a slot. A panic does, and heal exists for that case (`docs/diataxis/explanation/two-state-channels.md`).

## 5. The arrival race (seed §3), step by step

| Seed step | Verdict |
|---|---|
| At startup, a World with no scenario text spawns arrival with `spawn_blocking`. | **Confirmed, known.** The only condition is `has_scenario` false. History is not checked, so arrival runs on **every** boot of such a World and appends another arrival Narration each time (**known**; whether that is wanted is not in this ticket). |
| Arrival persists `Generating`. | **Refuted, known.** `run_inner` sets `Generating` on its local copy only. Its one save is at the end, with `Idle` or `Error`. |
| It claims no slot. | **Confirmed, known.** |
| `POST /action`: heal resets "Generating with no slot" to `Idle`. | **Refuted as worded.** There is nothing to heal, because arrival never persisted `Generating` (boot heal ran in `build_app_graph` before arrival spawned). The claim succeeds because no slot is held. The result is the same: the turn starts. |
| The turn runs and saves. | **Confirmed, known.** |
| Arrival's `save_message_and_snapshot` saves a state older than the turn. | **Confirmed as mechanism, known**; lost update **inferred** (pair 1). Extra effects: the arrival row is ordered after the turn's rows; an arrival LLM failure writes `Error` over the turn's status. |
| The status display shows idle during arrival. | **Confirmed, known.** It reads the registry. |
| (new) A game switch during the arrival LLM call sends the arrival save into the new game. | **Inferred** from `Storage::game_id()` at save time. |

### What a test needs

- **Known.** `src/test_support/` and `tests/` never construct `ArrivalTaskContext` or call `spawn_arrival_task_if_needed`. `src/application/arrival_service_tests.rs` runs arrival alone.
- A unit test in `src/application/` can do it: `ArrivalTaskContext::new_for_test` and `run` are crate-visible. Give arrival a `MockBackend` with `with_delay` and the `narration_started` flag (`src/adapters/driven/llm/providers/mock.rs`), run it on a thread, wait for the flag, run a full turn through `process_action` with a mock quantifier that moves the player, join, and assert that the latest Snapshot keeps the move and that the arrival Narration is not after the turn's rows.
- An HTTP test needs `TestAppBuilder` to spawn arrival, which it does not do today.
