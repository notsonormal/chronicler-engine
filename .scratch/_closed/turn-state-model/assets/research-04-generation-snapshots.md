# Research 04: the Snapshots each generation path saves

Ticket: [Research: trace the Snapshots each generation path saves](../issues/04-research-trace-generation-snapshots.md).

Labels: **known** = read in the code at the time of writing. **inferred** = follows from the code, not run. **tested** = a test pins it. No fact here is newly tested. Symbols, not line numbers.

## Rules every path shares (known)

- **R1. Two save functions.** `MessageService::save_message_and_snapshot` writes a Snapshot and can set pointers. `MessageService::save_state` (and `PipelineRun::persist`, which calls it) writes a Snapshot and never sets a pointer. `PipelineRun::persist_snapshot_or_err(label)` is `save_message_and_snapshot` plus error handling. The label only goes to the log.
- **R2. Pointer rule.** `save_message_and_snapshot` sets a pointer in two cases only:
  - the last Swipe of `narrative.retry_target`, if its `snapshot_id` is `None`. It then inserts the Swipe row and makes it active;
  - every history Message with `id == 0` (`Message::is_unpersisted`). It sets the active Swipe and Swipe 0, then inserts the Message and all its Swipes.
  A pointer, once set, is never moved by the pipeline. The first save after a Message or Swipe is created sets its pointer.
- **R3. Arriving location.** `save_message_and_snapshot` first runs `save_arriving_location`: it updates `location_header` on a Swipe row already written. `GameState::log_movement_completion` queues that write in the engine commit.
- **R4. `retry_target` is in memory only.** `NarrativeSnapshot` holds no messages and no `retry_target`. `GameState::push_message` appends a new Swipe to `retry_target` when the target and the new text have the same event-ness, and ignores `MessageType` in that case.
- **R5. Claim save.** Every gated entry goes through `ActionPipeline::claim_and_spawn` → `GenerationGate::try_claim`, which sets `Generating`/`Narrating` and calls `save_message_and_snapshot`. Call it **Sc**. It runs in the request handler, before the spawned task.
- **R6. Finalize.** `PipelineRun::phase_finalize` sets `Idle` (unless an error is set), clears the phase and calls `persist` (no pointer). Call it **Sf**.

## Correction to the seed diagram (known)

The seed says `Input.snapshot_id = S0`. That is wrong. `process_action` adds the Input in its `before_claim` closure, so **Sc** persists the Input and sets its pointer. **S0** ("pre-main snapshot") finds no unpersisted Message and sets nothing.

Consequences:

- **Sc** is written from `load_or_fresh` *before* `execute_action_with_inputs` clears `last_trigger`. The Input therefore points at a state that still holds the previous turn's `last_trigger`. **S0** holds `last_trigger: None`.
- A main ReNarrate Retry restores the Input's Snapshot (**Sc**) and goes to `run_from_input` directly, so nothing clears `last_trigger`. Inferred: after a main Retry whose redo fires no Trigger, `last_trigger` is the old Trigger, and `retrigger`'s guard (`last_trigger.is_some()`, last Message a non-event Narration) lets the player re-fire it. Not tested. Feeds ticket 06 and ticket 10.

The seed also misses two saves: the best-effort save after the Quantifier (**S2b**) and the Options-rewrite save (**S6**).

## Sequences

Notation: `→ Sx  label  [pointer set]`. `label` is the `persist_snapshot_or_err` label, or `(unlabelled)`.

### P1. Action: free action with text (`Action::FreeAction`, `run_from_input`)

```text
process_action
  before_claim: add Input (id 0)
  try_claim                        → Sc  (unlabelled)                 [Input.snapshot_id = Input.swipe0 = Sc]
spawned: execute_action_with_inputs
  load_or_fresh (= Sc); last_trigger = None (memory)
  phase_pre_main_snapshot          → S0  "pre-main snapshot"          [none]
  NarrationGeneration::run
    LLM; check_game_unchanged; add Narration (id 0)
                                   → S1  "pre-quantifier narration"   [Narration = swipe0 = S1]
  phase_post_generation
    phase = Quantifying            → S2  "pre-quantifier phase update" [none]
    run_post_generation_agents (Quantifier)
    low confidence? add System "NPC detection uncertain"
                                   → S2b (unlabelled, best-effort)    [that System Message, if any]
  phase_engine_commit (execute_freeaction_impl, consumes state)
    movement: current_room_id; unknown room? add System "Entered unknown location"
    log_movement_completion: header onto the Narration, queue arriving_location_write
    scene.npcs_in_area = Quantifier NPCs; evaluate_triggers; apply_npc_events
  build_trigger_request (may return None)
                                   → S3  "post-engine snapshot"       [unknown-room System, if any; R3 header write]
  retry_target → history (none here)
  [Trigger branch, P2]
  rewrite_options_after_turn (not if impersonated)
    clear options; Options agent; failure? add System
                                   → S6  (unlabelled, best-effort)    [options-failure System, if any]
  phase_finalize                   → Sf  (persist)                    [none]
```

What each state holds (known): **Sc** and **S0** = pre-turn world, `Generating`. **S1** = pre-turn world plus the Narration, `Generating`/`Narrating`, the previous turn's `current_options` and `last_backend_name` (the new names are set after `NarrationGeneration::run` returns). **S2**/**S2b** = same world, `Quantifying`. **S3** = first state with the Quantifier result, movement, encounter events and the encounter-log changes. **S6** = S3 plus the new Options set. **Sf** = `Idle`.

### P2. Trigger continuation inside a main turn

Runs after **S3** when `build_trigger_request` returns `Some`.

```text
phase_trigger_continuation_llm_call
  status Generating, phase GeneratingEvent, last_trigger = trigger
  check_game_unchanged             → S4  "pre-event snapshot"         [none]
  Trigger LLM; check_game_unchanged
  commit_trigger_narration: last_trigger, pending_event, add event Narration (id 0), mark_trigger_fired
                                   → S5  "post-trigger snapshot"      [event Message = swipe0 = S5]
reconcile_post_trigger_npcs (second Quantifier run; no save)
rewrite_options_after_turn         → S6
phase_finalize                     → Sf
```

**S5** holds the Trigger context, the post-movement room, the fired mark and the main Quantifier's scene, *before* the post-trigger reconciliation (known; matches the phase-2 ticket 09 Answer). The reconciled scene is first saved in **S6**.

Inferred: when a Trigger matches but `build_trigger_request` returns `None` (no room data, unreadable settings, missing preset, assembler error), the turn continues with no event, no message and no fired mark. The Trigger can match again next turn.

### P3. Action: continue and guide (`Action::FreeAction("")`, `Action::Guide`)

Same as P1, except `before_claim` adds no Input. **Sc** sets no pointer. The guided Narration stores `guide_text` on its Swipe (`add_message_with_inputs`). Known.

### P4. Action: impersonate (`Action::Impersonate`)

Same as P1/P2, except: no Input at claim; `NarrationGeneration` uses the impersonate preset and adds an **Input** (`impersonated = true`), which takes **S1**. The full tail runs: Quantifier on the impersonated text, engine commit, Trigger. The Options rewrite is skipped. Known.

### P5. Retry, common prefix (`ActionPipeline::retry` → `retry_last_response`)

```text
retry
  before_claim: last_turn_is_retryable
  try_claim                        → Sc  (unlabelled)                 [none]
  pre_spawn: check_retry_anchor (anchor's own Snapshot must load)
spawned: retry_last_response
  load_messages; resolve_retry_target (target = last Input or Narration)
  state_at_click = load_latest_snapshot (= Sc)
  restore id = is_event ? target's active-Swipe Snapshot : anchor's Snapshot
  reconstruct_retry_state: history truncated, retry_target = target (memory)
  … mode branch (P6–P10) …
  restore_click_time_state_after_failed_redo
                                   → Sr  (save_state, only on failure, see Failures)
```

### P6. Retry ReNarrate, main (target = last non-event, non-guided Narration)

Anchor = last Input. Restores the Input's **Sc** of the original turn (not S0, see Correction). History is cut after the Input. Then `retry_main_narration` → `run_from_input` = P1 from **S0**, with these differences (known):

- **S1'** appends a Swipe to `retry_target` and sets that Swipe's pointer [Swipe = S1']. No new Message.
- **S3'**: the R3 header write targets `retry_target` and its active Swipe (`arriving_message_mut`).
- After **S3'** the target joins the in-memory history. A Trigger in P2 then adds a *new* event Message.
- `last_trigger` is not cleared (see Correction).

### P7. Retry ReNarrate, guided (target = last Narration with `guide_text`)

Anchor = the guided Narration itself. Restores its **S1** (pre-quantifier). History is cut before it. `run_from_input` with empty input falls back to the target's stored `guide_text`. Otherwise P6. Known.

### P8. Retry ReNarrate, event (target = last Narration with `event_header`)

Anchor = last non-event Message (checked by `check_retry_anchor`). Restores the event Message's **active Swipe's** Snapshot (its **S5**). History is kept up to the anchor.

```text
retry_event_continuation
  last_trigger from the restored S5 (missing → TriggerMissing)
  load_world_bundle
  phase_trigger_continuation → phase_trigger_continuation_llm_call
                                   → S4' "pre-event snapshot"         [none]
    commit_trigger_narration: push_message adds a Swipe to retry_target
                                   → S5' "post-trigger snapshot"      [new event Swipe = S5']
  reconcile_post_trigger_npcs (no save)
  retry_target → history
  rewrite_options_after_turn       → S6'
  phase_finalize                   → Sf
```

No Narration, no Quantifier on the main text, no engine commit (known).

### P9. Retry ReImpersonate (target = last Input with `impersonated`)

Anchor = that Input. Restores its **S1** (the impersonate generation saved the Input there). History is cut before it. `run_from_input` with empty input falls back to the stored impersonate direction. P4 runs in full from **S0**: the new text is a Swipe on the Input [Swipe = S1'], then Quantifier, engine commit, Trigger. No Options rewrite. Known. This is the seed §2 conflict with `docs/specs/swipe_new.md` I.6.

### P10. Retry UserRegen (target = last plain Input)

Anchor = that Input. Restores its **Sc**. History is cut before it.

```text
retry_user_regen
  status Generating/Narrating (memory only; no pre-main save)
  NarrationGeneration::run with the regen instruction, MessageType::Narration
    push_message adds a Swipe to the Input target (R4)
                                   → S1' "pre-quantifier narration"   [Swipe = S1']
  retry_target → history
  phase_finalize                   → Sf
```

No Quantifier, no engine commit, no Trigger, no Options rewrite (known).

### P11. Retrigger (`ActionPipeline::retrigger` → `retrigger_event`)

Guard: `last_trigger.is_some()` and last Message is a non-event Narration. **Sc** [none]. `retrigger_event` loads the latest state (no rewind, no `retry_target`) and runs `retry_event_continuation`: **S4'** [none], `commit_trigger_narration` adds a *new* event Message, **S5'** [event Message = S5'], reconcile, **S6'**, **Sf**. Known.

### P12. On-demand Options (`Action::Options` → `process_options` → `execute_options_refresh`)

```text
try_claim                          → Sc  (unlabelled)                 [none]
execute_options_refresh
  status Generating, phase Options → So0 "pre-options snapshot"       [none]
  check_game_unchanged; load_world_bundle
  run_options_generation
    ok, non-empty: set options     → So1 (unlabelled)                 [none]
    else: add System               → So1 (finalize_options_failure)   [that System Message]
  phase_finalize                   → Sf
```

The turn-end rewrite (`rewrite_options_after_turn`) is the **S6** save in P1, P2, P8 and P11. It does not run for P4, P9 or P10. Known.

### P13. Narrator mode

`NarratorMode` only picks the preset bundle at game creation and in `GameCatalogue::switch_mode`. No pipeline code branches on it. Narrator-mode generations take the paths above. Known.

### P14. Starting-scenario Narration (no generation)

Two writers, one Snapshot each, both outside the gate (known):

- `bootstrap::init_game::load_game_state`, only when the game has no Snapshot: `inject_scenario_logs`, `init_scenario_npcs`, `storage.save_snapshot` → **Si**, then `try_persist_initial_message` [scenario Narration = swipe0 = Si].
- `GameCatalogue::persist_initial_state_with_swipes` (`create_game`, `reset`): `build_fresh_initial_state`, `storage.save_snapshot` → **Si**, `set_snapshot_id` + `persist_swipes` [scenario Narration = Si].

If the scenario text renders empty, `build_fresh_initial_state` leaves `pending_location` set in **Si**. The next Narration takes it as its location header (inferred).

### P15. Arrival narration (`spawn_arrival_task_if_needed` → `ArrivalTaskContext::run_inner`)

```text
bootstrap::run::prepare_state
  load_game_state (may save Si with no Message when the World has no scenario text)
  spawn_arrival_task_if_needed: only if the World has no non-empty scenario text
spawn_blocking: run_inner           (no GenerationGate slot)
  load_expecting_valid_state (fallback: build_fresh_initial_state)
  status = Generating (memory only)
  arrival LLM call
  ok: add Narration, status Idle; err: status Error
                                   → Sa  (unlabelled)                 [arrival Narration = swipe0 = Sa]
```

Corrections and additions to seed §3:

- **Known:** arrival never persists `Generating`. It writes one Snapshot, at the end. So `heal_stale_if_changed` has nothing to reset. The race is still a lost update: **Sa** is built from the state loaded before the LLM call, so a turn that saved in the meantime is overwritten as the latest Snapshot (inferred).
- **Inferred:** the spawn condition does not check the history. Every server start for a World with no scenario text appends one more arrival Narration to the current game. Not tested. `arrival_service_tests.rs` tests `run` only.
- **Known:** the arrival Narration has no location header. Arrival sets no `pending_location`.

### P16. Swipe switch (`MessageService::switch_swipe`)

No gate claim; the caller passes `is_generating`. Last Message only. `update_active_swipe`, then copies the target Swipe's Snapshot with `Idle`, default phase, `current_options` cleared → **Ss** (`storage.save_snapshot`, new row) [none]. Known.

What the copy restores, by Swipe kind (known from the sequences above):

| Swipe kind | Its pointer | World the switch restores |
|---|---|---|
| Narration, main turn or main/guided Retry | S1 | pre-turn: before Quantifier, movement and encounters |
| Event Narration (P2, P8, P11) | S5 | post-movement, Trigger fired, before reconciliation |
| Plain Input, Swipe 0 | Sc | pre-turn, stale `last_trigger` |
| Plain Input, UserRegen Swipe | S1' of P10 | pre-turn |
| Impersonated Input (P4, P9) | S1 | pre-turn |
| Scenario or arrival Narration | Si / Sa | game start |

## Failures (known unless marked)

| Where it fails | Writes | Left behind |
|---|---|---|
| `try_claim` save | none committed | slot released; HTTP error |
| Any `persist_snapshot_or_err` | the failing write, then `persist` (`save_state`) with `Error`, then `finalize_phase_error` → `save_message_and_snapshot` again | Error status. The second save retries the Message insert (inferred: a Message whose first insert failed can land with the error Snapshot as its pointer) |
| `NarrationGeneration` before the Narration (bundle, room, preset, LLM, empty text) | `finalize_phase_error` → `save_message_and_snapshot` | no Narration; Input (P1) stays with Sc; Error |
| `check_game_unchanged` → `Cancelled` | `handle_cancellation`: `load_or_fresh` of the **current** game, `Idle`, `persist` | Inferred: the write goes to the game the player switched to and sets it `Idle`, even if that game is generating. Feeds ticket 05 |
| Quantifier agent error | none (warn) | default result: no movement, room-default NPCs, System message at S2b |
| Engine commit | `persist_generation_error`: latest Snapshot (S2b) + Error via `save_state` | Narration or retry Swipe persisted with S1; no movement; R3 header not written |
| Trigger LLM, empty text, `commit_trigger_narration` | `finalize_phase_error` on the post-commit state | S3 world kept, `last_trigger` set, no event Message, Error. No pipeline code adds a System Message here |
| `reconcile_post_trigger_npcs` | `finalize_phase_error` | event Message with S5; Error |
| Options agent in the turn-end rewrite | System Message at S6 | empty Options set; not an Error |
| Any Retry that ends in Error | `restore_click_time_state_after_failed_redo` → **Sr** (`save_state`) | If no new Swipe reached the DB: the click-time state (Sc of the Retry) with the Error. If one did: the attempt's world stays (`DurableWorldAfterFailedRedo::Attempt`) |

## Table of save sites

Pointer column: what the save sets under R2. "none" means no unpersisted Message or unset `retry_target` Swipe exists there on the normal path (inferred for the sites marked *).

| # | Path(s) | Function / phase | Label | State held | Pointer set |
|---|---|---|---|---|---|
| 1 | all gated (P1–P12) | `GenerationGate::try_claim` | — | loaded latest + `Generating`/`Narrating`; P1 adds the Input | P1: Input → Sc. Others: none |
| 2 | P1–P4, P6, P7, P9 | `PipelineRun::phase_pre_main_snapshot` | pre-main snapshot | pre-turn (or rewound); main entry: `last_trigger` cleared | none* |
| 3 | P1, P3, P4, P6, P7, P9, P10 | `NarrationGeneration::run` | pre-quantifier narration | + Narration / Input / Swipe; before Quantifier | new Message → S1, or retry Swipe → S1' |
| 4 | P1–P4, P6, P7, P9 | `PipelineRun::phase_post_generation` | pre-quantifier phase update | phase `Quantifying` | none* |
| 5 | same as 4 | `PipelineRun::phase_post_generation` | — (best-effort) | after Quantifier, before engine commit | low-confidence System Message |
| 6 | same as 4 | `ActionPipeline::run_from_input` | post-engine snapshot | after engine commit (movement, scene, encounters); R3 header write | unknown-room System Message |
| 7 | P2, P8, P11 | `PipelineRun::phase_trigger_continuation_llm_call` | pre-event snapshot | `GeneratingEvent`, `last_trigger` set | none* |
| 8 | P2, P8, P11 | `PipelineRun::phase_trigger_continuation_llm_call` | post-trigger snapshot | after `commit_trigger_narration`, before reconciliation | event Message → S5, or event retry Swipe → S5' |
| 9 | P1–P3, P6–P8, P11 | `ActionPipeline::rewrite_options_after_turn` | — (best-effort) | after reconciliation and new Options | options-failure System Message |
| 10 | all generations | `PipelineRun::phase_finalize` → `persist` | — | `Idle` (or Error kept) | never (`save_state`) |
| 11 | P12 | `execute_options_refresh` | pre-options snapshot | `Generating`/`Options` | none* |
| 12 | P12 | `execute_options_refresh` | — | new Options set | none* |
| 13 | P12 | `finalize_options_failure` | — | + System Message | that System Message |
| 14 | all generations | `ActionPipeline::finalize_phase_error` (state given) | — | in-flight state + Error | any Message or retry Swipe still unset |
| 15 | engine-commit failure, Retry setup failures | `ActionPipeline::persist_generation_error` | — | latest Snapshot + Error | never |
| 16 | any `persist_snapshot_or_err` failure | `PipelineRun::persist` | — | in-flight state + Error | never |
| 17 | cancellation | `PipelineRun::handle_cancellation` | — | current game's latest + `Idle` | never |
| 18 | P5–P10 on failure | `restore_click_time_state_after_failed_redo` | — | click-time Snapshot + Error | never |
| 19 | P15 | `ArrivalTaskContext::run_inner` | — | state loaded at task start + arrival Narration, `Idle` or Error | arrival Narration → Sa |
| 20 | P14 | `init_game::load_game_state` | — | scenario start | scenario Narration → Si |
| 21 | P14 | `GameCatalogue::persist_initial_state_with_swipes` | — | scenario start | scenario Narration → Si |
| 22 | P16 | `MessageService::switch_swipe` | — | copy of the Swipe's Snapshot, `Idle`, no Options | never |
| 23 | before each claim; `/status` reset | `heal_stale_if_changed`, `reset_persisted_status` | — | latest + `Idle` | never |

Sites outside generation that also write a Snapshot, for ticket 05: `MessageService::edit_history`, `MessageService::delete_last`.

## Answers to the four questions, in short

1. **Phase sequence:** see P1–P16.
2. **Saves:** 9 pointer-capable saves can run in one main turn with a Trigger (Sc, S0, S1, S2, S2b, S3, S4, S5, S6) plus Sf. Only S1 and S5 set a generation's own pointer. The rest set pointers only for System Messages or the claim-time Input.
3. **Pointers:** a Narration or Input generation points at S1 (pre-Quantifier). An event points at S5 (pre-reconciliation). A free-action Input points at Sc (pre-turn, before `last_trigger` is cleared). No pointer ever holds the state after reconciliation or after the Options rewrite. The "set only while `None`" rule means a pointer never moves.
4. **Failures:** see the Failures table.

## Open points for later tickets

- Main Retry restores a stale `last_trigger` (Correction). Ticket 06, ticket 10.
- `handle_cancellation` writes into the new current game. Ticket 05.
- Arrival runs on every start for scenario-less Worlds and never persists `Generating`. Ticket 05; revise seed §3.
- A Trigger match whose request fails to build is silently dropped. Ticket 06.
- The phase-2 ticket 09 Answer names "the trigger-failure System entry". The current pipeline adds no System Message on Trigger failure. Ticket 09 should check where that entry comes from.
