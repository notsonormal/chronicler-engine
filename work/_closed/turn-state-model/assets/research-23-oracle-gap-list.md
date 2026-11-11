# Research 23, appendix: the oracle-gap list

Asset of [Compare the target design with the code](../issues/23-compare-target-design-with-code.md). The triage of these rows is in [`research-23-design-vs-code.md`](research-23-design-vs-code.md) §5.

A `scout` agent wrote this list. The ticket's session checked it against the code. Two rows were wrong, and are corrected in place:

- **9.4.** `Storage::delete_message` is a hard `DELETE` (known: `storage/messages.rs`). Research 07 found the same. The row said "soft".
- **12.2.** Arrival sets `Generating` on its copy in memory only. Its one save is at the end, with `Idle` or `Error` (known: `ArrivalTaskContext::run_inner`; research 05 §5). The row said that the persisted status reads `Generating` while it runs.

"Contradicts" is a label that the scout added. It marks a row where the code does the opposite of the spec.

---


Read-only reconnaissance of `/workspace/chronicler-engine` at commit `9cbf3112` (working tree dirty only in `.scratch/`).
Spec under test: `.scratch/_closed/turn-state-model/assets/clean-spec.md` version 3.
No file was modified except this report.

## 0. How to read this

**Label** (how the behaviour was established):

- **known** — read in the source.
- **tested** — a named test pins it (`src/**/*_tests.rs` unit mirror, or `tests/` integration).
- **inferred** — follows from the code but no single line or test states it.

**Spec status** (what the spec says about it):

- **absent** — the spec says nothing.
- **loose** — the spec states it at lower resolution, or in another requirement; `R-nn` / `§n` cited.
- **black box** — the spec places it inside a black-box contract (§3.1–§3.7); a rewrite keeps the black box, so the row is not a lifecycle gap. Kept for completeness, with the code/contract mismatch noted where one exists.
- **contradicts** — *added label.* The spec states the opposite (or a requirement that the code does not meet). This is stricter than a gap: a rewrite tested against the spec would change the behaviour, and today's tests would fail.

Two caveats:

1. The present-day oracle is not the clean spec. Today's tests carry `// [docs/specs/<name>.md] SCENARIO: n.m` tags against `docs/specs/*.md`, which are far more detailed than the clean spec for some of these rows. A row marked *absent* may still be pinned by a scenario of the current feature specs.
2. Where a rewrite keeps a black box, differences inside that black box are not lifecycle gaps even when they are real. Those rows are labelled and marked.

---

## 1. Pipeline, work, busy and stop

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 1.1 | A stop (`POST /status/reset-generating`) only releases the per-game registry slot and resets the persisted status. The running work is not cancelled; its LLM call returns and its later save units still land (status and story change after the stop answered "reset"). | `layout::handlers::endpoints::reset_generating_handler`, `GenerationGate::release_generation_slot_for_game`, `ActionPipeline::reset_persisted_status` | known; `tests/http/dashboard.rs::test_generating_status_answers_from_live_registry` pins only the status half | **contradicts R-81** (the engine drops the step, keeps only what the work saved before) |
| 1.2 | Busy is the in-memory per-game registry slot, dropped by an RAII guard when the task ends (or the process dies). | `GenerationGate::is_busy`, `GenerationGuard::drop`, `GenerationSlot` | known | loose R-72, R-92 |
| 1.3 | A persisted `Generating` status with no live slot is healed to `Idle` lazily, before validating or claiming (any Action/Retry/Retrigger/`/options`), and on game switch. There is no start-up pass. | `GenerationGate::heal_stale`, `ActionPipeline::heal_stale_if_changed`, `heal_stale_status` | known; `core_tests::test_switch_game_heals_stale_status_for_the_current_game` | loose R-80, R-92 |
| 1.4 | Work is refused with "shutting down" while the process shutdown token is cancelled; the answer is 503. | `ActionPipeline::claim_and_spawn`, `ActionPipeline::is_shutting_down` | known; `core_tests::test_pipeline_cancels_when_token_already_cancelled` | absent |
| 1.5 | The claim itself writes: status `Generating`, phase `Narrating`, a snapshot, plus the `Input` row the caller added in `before_claim`. So the Input is durable before any LLM call. | `GenerationGate::try_claim`, `ActionPipeline::claim_and_spawn` | known | loose R-02, R-11a |
| 1.6 | The pipeline re-checks the active game id at phase boundaries (`check_game_unchanged`) and abandons the run when it changed; every storage write instead resolves the game at call time. | `PipelineRun::check_game_unchanged`, `Storage::current_game_id` | known; `narration_generation_tests::test_run_game_changed_returns_cancelled` | **contradicts R-74** (a change of active Game must not move the work's writes) |
| 1.7 | Generation ids are process-global monotonic counters, not per game. | `GenerationGate::next_generation_id` | known | absent (record) |
| 1.8 | Only the swipe switch checks busy on the HTTP side. `POST /history/:id` (edit) and `POST /history/delete` do not check busy at all and write while a generation runs. | `switch_swipe_handler`, `edit_history_handler`, `delete_history_handler` | known | **contradicts R-70** |
| 1.9 | The retry pre-check is "the story has any `Input` row anywhere, or the last input-or-narration is guided". A retry is therefore offered on a last Message whose own turn had no Input. | `MessageHistory::last_turn_is_retryable`, `ActionPipeline::retry` | known | loose R-44 |
| 1.10 | A retry claim failure (missing anchor, missing snapshot) answers 500 **and** writes a failure line. | `ActionPipeline::check_retry_anchor` | known; `retry_tests::test_retry_returns_internal_error_when_anchor_has_no_snapshot_id` | loose R-46 |

## 2. Narration generation and the persist order

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 2.1 | Every LLM answer is post-processed before use: a leading `<channel\|>`, `<thought>…</thought>`, `<\|channel>thought…<channel\|>`, `<\|turn>model`, `<turn\|>`, `<\|turn>` are removed, each line's leading whitespace is stripped, and the whole text is trimmed. | `LlmCallRecorder::complete`, `sanitize_llm_output` | known; `sanitize_tests` | absent (application-layer output post-processing; §3.1 does not state it) |
| 2.2 | The same sanitising runs for **every** role (narrator, quantifier, options) because it sits in the recorder, and the sanitised text is what the parsers see and what forensics stores as `parsed_response`. | `LlmCallRecorder::complete` | known | absent |
| 2.3 | The empty-text check runs after sanitising, so an answer that is only a thought block counts as empty. | `PipelineRun::call_narrator`, `phase_trigger_continuation_llm_call` | known; `core_tests::test_pipeline_empty_narration_sets_error_message_and_no_narration` | loose R-19 |
| 2.4 | An empty answer produces failure kind `UnreadableAnswer` with raw text `LLM Error: empty response`; the status line is the unreadable-answer line, so empty text is not named as empty. | `PipelineRun::call_narrator`, `generation_failure_summary` | known | **contradicts R-19** ("the failure says that the text was empty") |
| 2.5 | The player's text is filtered for prompt use: every `{{ … }}` block in the submitted command becomes `[FILTERED]`; other layers use placeholder rendering instead. | `sanitize_for_prompt`, `LayerRenderer::render_user_layer` | known; `sanitize_tests` | **contradicts §3.5** ("It removes nothing") |
| 2.6 | The room check runs inside the generation, after the claim already persisted the `Input`. An action in an invalid room leaves the Input in the story. | `NarrationGeneration::run` (`resolve_room`), `ActionPipeline::claim_and_spawn` | known; `tests/http/actions.rs::test_nonexistent_room_sets_error_status_http` (asserts only "no Narration") | **contradicts R-20** ("For a free-text Action, the Input is not saved") |
| 2.7 | The invalid-room failure kind is `SceneMissing`, its line is "The current scene could not be found."; the invalid room's name is not shown. | `narration_generation::NarrationGeneration::run`, `generation_failure_summary` | known | **contradicts R-20** ("The status names the invalid room") |
| 2.8 | The preset is resolved per generation from the Game's active preset ids (system vs impersonate), plus `settings.response_length`; a redo re-reads the current ids. | `NarrationGeneration::resolve_preset_choice`, `PipelineRun::load_preset_and_response_length` | known; `tests/http/swipe_new.rs::test_retry_re_impersonate_uses_current_preset_content_http` | loose R-59, §3.1 |
| 2.9 | The narration prompt is a fixed layer stack: system (role/instructions/global rules), then `<GameState>` (room name + description), `<KnownNpcs>` (every NPC with "(in room)"/"(elsewhere)" and a summary or 3 description lines), `<NpcsInRoom>` (with relationships between present NPCs), `<PlayerCharacter>` (dropped for impersonate), `<WorldLore>`, `<ConversationHistory>`, the preset's style/format tail, `<PlayerInput>`, `<Guide>`. | `LayerRenderer::render_*` | known; `assembler_tests` | black box §3.5 |
| 2.10 | The guide is rendered as a final layer and skipped when it trims to empty; the guide is stored on the Swipe (`guide_text`), including an empty guide from a bare `/guide`. | `LayerRenderer::render_guide_layer`, `Message::is_guided`, `Action::parse` | known | absent (empty guide marking the Swipe guided) / loose R-01 |
| 2.11 | `/action` trims the whole command before parsing, so leading and trailing whitespace in player text is discarded (and a whitespace-only command is a continue with no Input). | `action_handler`, `action_confirm_handler` | known; `tests/http/actions.rs::test_empty_command_continuation_no_input_http` | absent |
| 2.12 | The generation text is saved as a new Message, or as a new Swipe on the retry target; the save writes a snapshot and assigns that snapshot id to the new Message/Swipe. The dock's option set is not touched by this save. | `GameState::push_message`, `MessageService::save_message_and_snapshot` | known; `core_tests::test_pipeline_saves_narration_before_quantifier` | loose R-11a, §4.3 |
| 2.13 | A redo reuses the retry target's stored inputs when the caller passes none (guide, impersonated mark, impersonate direction). | `ActionPipeline::run_from_input` | known | loose R-40 |
| 2.14 | A user regen is generated from a fixed instruction wrapper ("Regenerate the user's previous message as an alternate swipe…" with the original text in `<original_user_message>` tags) and the answer is stored as an `Input` Swipe; the regen runs no analysis, no event generation and no options rewrite. | `ActionPipeline::build_user_regen_instruction`, `retry_user_regen` | known; `retry_tests::test_retry_flow_user_regen_mode_stays_tail_less` | loose R-18, R-61 (wrapper text is absent) |
| 2.15 | The analyse-then-commit order is: text save (snapshot 1) → quantifier call → engine commit → post-engine snapshot (snapshot 2) → event call → post-trigger snapshot → post-trigger reconcile → dock/message save → finalize. A single free-text Action can therefore write several snapshots. | `ActionPipeline::run_from_input` | known; `research 04`; `core_tests::test_pipeline_saves_narration_before_quantifier` | loose R-90, R-11a |

## 3. Analysis (the Quantifier call)

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 3.1 | The quantifier call is attempted at most twice; a low-confidence result or a transport error triggers the second attempt. | `quantify_room_with_llm_call` | known; `orchestration_tests::test_quantifier_retry_on_low_confidence`, `test_quantifier_retry_on_llm_error`, `test_quantifier_all_attempts_fail_fallback` | absent (no retry count in §3.2) |
| 3.2 | Confidence rule: `High` when JSON parses (even with an empty list), `Medium` when the text fallback matched known ids, `Low` otherwise. | `parse_with_movement`, `extract_npcs` | known; `orchestration_tests::test_determine_npcs_high_confidence`, `test_determine_npcs_medium_confidence` | absent |
| 3.3 | JSON is accepted bare, inside a ```` ```json ```` fence, or as the span from the first `{` to the last `}`; unknown ids are dropped. | `try_parse_json_full` | known; `parser_tests` | absent |
| 3.4 | A `movement.type` other than "entering"/"leaving" (including "in") parses to no type while the `destination` is kept. | `parse_with_movement` | known | absent |
| 3.5 | `High`/`Medium` adopt the detected NPC list; `Low` keeps the scene's current NPCs **and keeps the detected movement**. | `process_quantifier_result`, `static_npc_result` | known; `orchestration_tests::test_static_npc_result_preserves_movement` | **contradicts §3.2** (a fallback result is "no movement, and the Characters that the scene holds now") |
| 3.6 | After the fallback, if the NPC list is empty and the confidence is not `High`, the engine appends `[System] NPC detection uncertain — using room defaults` to the **story** and stores confidence `Low`. | `PipelineRun::phase_post_generation` | known | **contradicts R-84** (a notice is not a Message); absent for the text |
| 3.7 | The quantifier reads: the current room (name, description, navigation), the scene NPCs before the turn, all World NPC ids+names, all **static** room ids+names, the player name, the last 4 Messages, and the new narration text labelled `"{player_name}: {narration}"` as `player_action`. | `determine_npcs_in_room`, `QuantifierPromptBuilder::build` | known; `prompt_tests` | black box §3.2; **mismatch noted**: §3.2 supplies "the player's text that led to it", the code passes the narration (`player_input` is not read) |
| 3.8 | The active quantifier preset text is rendered with `{{user}}` and prepended to the built system prompt; when it is unreadable/absent, only the built-in instruction tail is used. | `QuantifierAgent::execute`, `QuantifierPromptBuilder::build_system_prompt` | known | absent (a per-role preset override is not in §3.2/§3.5) |
| 3.9 | The quantifier call passes `max_tokens = None` (the connection default). | `quantify_room_with_llm_call` | known | black box §3.2 |
| 3.10 | The result's confidence is stored on the scene as `format!("{:?}")` and in the snapshot, and is shown by `GET /debug/state`. | `PipelineRun::phase_post_generation`, `SceneState::quantifier_confidence` | known; `tests/http/requires_migration/debug.rs` | absent (record) |

## 4. World rules, movement, rooms, encounters

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 4.1 | The movement decision uses only `movement.destination`; `movement_type` is never consulted, so a "leaving" movement with a destination still moves the player. | `GameState::handle_movement`, `GameState::execute_freeaction_impl` | known; `game_state_tests::test_execute_freeaction_impl_with_movement` | loose §3.4 |
| 4.2 | A destination that matches no room id creates a dynamic room: id `dynamic_<unix seconds>`, name = the destination string, description "A place you have never seen before.", no exits, no items. | `Room::new_dynamic`, `GameState::attempt_movement` | known; `game_state_tests::test_new_dynamic_room`, `test_handle_movement_creates_dynamic_room` | absent (spec is silent on id/name/description) |
| 4.3 | Two unknown destinations added within the same second produce the same `dynamic_<secs>` id, and the second `insert` overwrites the first room entry. | `Room::new_dynamic`, `GameState::attempt_movement` | inferred | absent |
| 4.4 | Creating a dynamic room also appends `[System] Entered unknown location: <room id>` to the **story**. | `GameState::attempt_movement` | known; `game_state_tests::test_handle_movement_creates_dynamic_room` | **contradicts R-84** |
| 4.5 | Dynamic rooms are not in the quantifier's `<available_rooms>` list, so the quantifier can only re-enter one by its `dynamic_<secs>` id; a destination given as its name creates yet another room. | `determine_npcs_in_room`, `MapDef::get_room_by_id`, `attempt_semantic_walk` | known | loose §3.4 (see §3.2's "every Character of the World" clause for the added-room rule) |
| 4.6 | Triggers are evaluated **before** the encounter events are applied, so a Trigger's condition reads `times_met` from before the Characters that entered this turn were counted; the code comments this as deliberate. | `GameState::execute_freeaction_impl` | known; `game_state_tests::prop_execute_freeaction_impl_preserves_mutation_order` | loose §3.4 ("conditions hold in the new game state") |
| 4.7 | Encounter events are a set difference of the previous and new Character id lists: `Entered` sets `currently_meeting` and increments `times_met`; `Left` clears `currently_meeting`; a Character that leaves keeps its `times_met`. | `NpcEventList::from_diff`, `GameState::apply_npc_events` | known; `game_state_tests::test_apply_npc_events_entered`, `_left`, `_increments_times_met` | loose §3.4 |
| 4.8 | On a room change the engine also marks every Character the analysis reported as `currently_meeting` before the diff runs. | `GameState::update_npc_encounters_on_room_change` | known; `game_state_tests::test_handle_movement_sets_currently_meeting` | loose §3.4 |
| 4.9 | The scene is replaced wholesale by the analysis's ids mapped to cards; ids unknown to the World are dropped. | `GameState::execute_freeaction_impl` | known | loose §3.4 |
| 4.10 | The encounter record is a map over all Characters ever met; `times_met` starts at 1 and `currently_meeting` true for each of the World scenario's starting Characters. | `GameState::init_scenario_npcs`, `NpcEncounterLog` | known; `game_state_tests::test_npc_encounter_log_initializes_with_starting_room_npcs` | loose R-77 |
| 4.11 | Consistency assertions over the room and the Character maps run only with the `diagnostics` feature; without it an invalid room or unknown Character id passes silently. | `GameState::assert_state_consistency` (both cfg arms) | known | absent (build configuration) |
| 4.12 | The arriving room name is written onto the arriving Message's **active Swipe**; when the Message row already exists, the write is deferred to the next `save_message_and_snapshot` (`ArrivingLocationWrite`). If the process dies in between, the room report is never written. | `GameState::log_movement_completion`, `MessageService::save_arriving_location` | known; `game_state_tests::test_handle_movement_queues_the_arriving_location_write_for_a_saved_message` | **contradicts R-90** (no unit for the Swipe and its state); absent for the deferral |

## 5. Triggers

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 5.1 | When several Triggers match, the one chosen is **nondeterministic**: evaluation iterates `npcs.values()` (a `HashMap`) and takes the first match overall (and within one Character, its trigger order). | `GameState::evaluate_triggers`, `execute_freeaction_impl` (`.next()`) | known | loose §3.4 ("at most one Trigger that fires"; which one is unspecified) |
| 5.2 | A Trigger's conditions are: optional `room_id` equal to the current room id (absent = any room), `times_met` against `threshold` by `Eq`/`Lt`/`Gte`, and — for a non-repeat Trigger — "not already marked fired". `currently_meeting` is not a condition. | `Trigger`, `TriggerRequirement`, `NpcEncounterLog::check_condition` | known; `game_state_tests::test_evaluate_triggers_*`, `test_room_scoped_trigger_*` | loose §3.4 |
| 5.3 | A Trigger whose Character is **not** in the scene can fire (evaluation loops over every World Character). | `GameState::evaluate_triggers` | known; `game_state_tests::test_evaluate_triggers_fires_for_npc_not_in_area` | absent |
| 5.4 | The fired mark is written only for a non-repeat Trigger, and only inside the event commit — i.e. after the event text was produced. | `GameState::commit_trigger_narration`, `NpcEncounterLog::mark_trigger_fired` | known; `game_state_tests::test_commit_trigger_narration_marks_non_repeat_trigger_fired` | loose R-16 |
| 5.5 | `commit_trigger_narration` silently returns `Ok(())` on blank continuation text: no Message, no fired mark, no header. | `GameState::commit_trigger_narration` | known; `game_state_tests::test_commit_trigger_narration_empty_text_is_noop` | loose R-16, R-19 (the caller refuses blank text first, so the branch is latent) |
| 5.6 | The Trigger's direction is rendered with `{{user}}` = the persona name when the match is built, and the rendered text is what is stored. | `GameState::execute_freeaction_impl` (`TemplateVars`), `TriggerMatch` | known | absent |
| 5.7 | When a Trigger matches but the event request cannot be built (room missing from the map, unreadable settings, missing preset, assembly error), `build_trigger_request` returns `None` and the event is **silently skipped**: no event Narration, no failure line, no trigger context, no fired mark, and the turn ends idle with the options rewrite. | `PipelineRun::build_trigger_request`, `ActionPipeline::run_from_input` (`if let Some(request)`) | known | absent / **contradicts R-17** (a failed event narration is reported and leaves a Retrigger) |
| 5.8 | The event request pre-assembles the full system prompt, user prompt and `max_tokens` and stores them in the trigger context, which is persisted in the snapshot; the event call re-uses them instead of re-assembling. | `PipelineRun::build_trigger_request`, `StoredTriggerContext` | known | absent (record shape) |
| 5.9 | The event user prompt contains the fixed instruction block ("Previous narration: … Continue the scene naturally, incorporating the trigger event into the narrative. Do NOT repeat or contradict what was already described…"). | `PipelineRun::build_trigger_request` | known | black box §3.1 |
| 5.10 | The event generation sets `last_trigger` and the phase before persisting a pre-event snapshot, so the trigger context is durable before the event call. | `PipelineRun::phase_trigger_continuation_llm_call` | known; `core_tests::test_pre_event_snapshot_saved_before_continuation` | loose R-67 |
| 5.11 | After a successful event, a **second** quantifier call ("post-trigger reconcile") re-derives the scene from the event text, replaces the scene with its Character list and applies the enter/leave diff. A movement it reports is **discarded** (the reconcile never calls the movement path), so the event's analysis cannot move the player. The main path's low-confidence fallback (`[System] NPC detection uncertain`) is **not** applied here. | `PipelineRun::reconcile_post_trigger_npcs`, `run_post_generation_agents` | known; `tests/http/actions.rs::test_trigger_continuation_reruns_quantifier_detects_new_npc_http` | loose R-14, R-57 |
| 5.12 | The reconcile pass has its own failure modes: a failed call falls back (empty/current NPCs) and a storage error sets a failure line. | `PipelineRun::reconcile_post_trigger_npcs` | known | loose R-12 |

## 6. Retry

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 6.1 | Retry modes are chosen from the last input-or-narration Message: event or Narration → `ReNarrate`, impersonated `Input` → `ReImpersonate`, plain `Input` → `UserRegen`. | `ActionPipeline::resolve_retry_target`, `RetryMode` | known; `retry_tests::test_retry_flow_*` | loose R-51–R-61 |
| 6.2 | The anchor is the last `Input` row **anywhere** in the story (a guided target anchors on itself; an event target anchors on the previous non-event Message). | `MessageService::find_retry_anchor_msg` | known; `retry_tests::test_retry_flow_guided_turn_retries_without_older_input_text`, `test_retry_target_and_anchor_agree_when_a_system_notice_trails_an_event_narration` | loose R-40, R-44 |
| 6.3 | The redo's in-memory history is truncated at the anchor (`anchor+1` for `ReNarrate`, `anchor` for the other modes), so Messages after the anchor are excluded from the redo's prompt (their rows stay in storage and return on the next load). | `ActionPipeline::reconstruct_retry_state` | known | absent |
| 6.4 | For a redo of a continue (or any Narration whose Swipe carries no guide), the redo's input text is the most recent `Input` row's text in the story, and the anchor jumps back to it. | `ActionPipeline::retry_last_response` (`last_input_text()`), `find_retry_anchor_msg` | inferred from known | loose R-40; note: the target design's §4.5 says the start state is the Swipe state of the Message before the target, which the jump does not follow |
| 6.5 | A failed redo restores the click-time Snapshot only when no new Swipe landed for the target Message; when one landed, the attempt's world stays. | `ActionPipeline::restore_click_time_state_after_failed_redo`, `durable_world_after_failed_redo`; `DurableWorldAfterFailedRedo` | known; `retry_tests::test_retry_that_wrote_its_narration_keeps_that_world_http` (`tests/http/swipe_new.rs`), `tests/http/swipe_new.rs::test_failed_retry_does_not_roll_the_movement_back_http` | loose R-47, R-48 |
| 6.6 | The restore re-reads the *latest* snapshot to decide whether a failure is present, and copies the failure and phase onto the restored click-time state. | `ActionPipeline::restore_click_time_state_after_failed_redo` | known | loose R-47 |
| 6.7 | An event redo restores from the **event Message's own Swipe Snapshot** (taken after that event was committed and before the post-trigger reconcile), so it carries the trigger context, the fired mark and the world as of the event's own commit. | `ActionPipeline::retry_last_response` (`restore_snapshot_id`), `phase_trigger_continuation_llm_call` | known; `tests/http/swipe_new.rs::test_event_retry_does_not_carry_the_previous_swipes_encounter_http` | loose R-55, R-56 |
| 6.8 | A guided redo passes an empty input string and relies on the Swipe's stored guide. | `ActionPipeline::retry_last_response` (`target_is_guided`) | known; `retry_tests::test_retry_flow_guide_only_narration_retries_with_empty_input` | loose R-51, R-53 |
| 6.9 | The event redo path (`retry_event_continuation`) is shared with the Retrigger and reads `state.narrative.last_trigger`; missing → `TriggerMissing`, failure line "Retry failed: missing trigger context". | `ActionPipeline::retry_event_continuation` | known; `retry_tests::test_retry_event_missing_trigger_context` | loose R-46, R-56 |
| 6.10 | A retry claim with the gate already busy answers "Still thinking…" (200). | `ActionPipeline::retry`, `map_process_action_result` | known; `retry_tests::test_retry_returns_concurrent_generation_when_gate_busy` | loose R-70 |

## 7. Retrigger

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 7.1 | The pre-check needs `state.narrative.last_trigger` **and** a last Message that is a `Narration` with no event header. An impersonated `Input` as the last Message is refused with 400. | `ActionPipeline::retrigger` | known; `tests/http/retrigger.rs::test_retrigger_last_message_not_narration_returns_400_http` | **contradicts R-62** (an Impersonate Input is a valid last Message) |
| 7.2 | The trigger context is read from the state's `last_trigger`, not from the Message/Swipe; nothing is stored on the Message. Because `last_trigger` survives a delete, a Retrigger is still allowed after the event Message was deleted (and after other later Messages were deleted). | `ActionPipeline::retrigger_event`, `retry_event_continuation`, `MessageService::delete_last` | known; `tests/http/retrigger.rs::test_retrigger_creates_new_event_narration_no_rollback_http` | **contradicts R-17a, R-62** (the trigger context lives on the active Swipe of the last Message) |
| 7.3 | `last_trigger` is cleared only by a new Action (`execute_action_with_inputs` sets it to `None`); no other request clears it. | `ActionPipeline::execute_action_with_inputs` | known | loose R-17a |
| 7.4 | A Retrigger always rewrites the offered set after the event (resolving the toggle from the Game row), and never runs the main quantifier or a movement. | `ActionPipeline::retry_event_continuation` | known; `retry_tests::test_retry_event_continuation_rewrites_options_when_enabled`, `tests/http/retrigger.rs::test_retrigger_does_not_rerun_quantifier_http` | loose R-63 |
| 7.5 | A failed Retrigger writes the failure line and leaves the story ending in the same Message. | `ActionPipeline::retry_event_continuation`, `finalize_phase_error` | known; `tests/http/retrigger.rs::test_retrigger_trigger_narration_failure_sets_error_http` | loose R-66 |

## 8. Options

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 8.1 | `/options` is refused (400, "No scene to generate options for yet") when the history is **empty** — not when no Message has text. | `ActionPipeline::process_options`, `process_options` (empty-history check) | known; `tests/http/options.rs::test_options_without_scene_history_errors_http` | **contradicts R-27** ("no Message has text") |
| 8.2 | Parsing tries three strategies in priority order — `<suggestion>…</suggestion>`, `^\s*\d+[.)]\s+`, `Suggestion N:` — and the **first strategy that yields any item wins**, so one answer never mixes shapes. At most `DEFAULT_OPTION_COUNT` = 3 items, each trimmed, empty items skipped. | `parse_options`, `DEFAULT_OPTION_COUNT` | known; `options/parser_tests` (`parses_suggestion_tags`, `parses_numbered_list`, `first_matching_strategy_wins_no_shape_mixing`, `trims_to_requested_count`) | absent (count and strategies) |
| 8.3 | The options call is attempted at most twice; a transport error or a zero-parse attempt retries. | `options_with_llm_call`, `MAX_OPTIONS_ATTEMPTS` | known | absent |
| 8.4 | A failed `/options` keeps the prior offered set and appends `[System] Options generation failed: <error>` to the **story**, then finalises as a normal (non-error) turn. | `ActionPipeline::execute_options_refresh`, `finalize_options_failure` | known; `options_tests::test_process_options_failure_keeps_prior_set`, `tests/http/options.rs::test_failed_regeneration_keeps_prior_set_http` | **contradicts R-83/R-84** (the spec's notice lives in the dock and is not a Message) |
| 8.5 | "No options agent registered" is reported as `Ok(Vec::new())` and surfaces as the same System-Message path with "…is not available". | `ActionPipeline::run_options_generation`, `options_tests::test_no_options_agent_surfaces_unavailable_message` | known | **contradicts R-83** |
| 8.6 | The turn-end rewrite clears the offered set first; when the toggle is on it regenerates, and a failure leaves the set empty plus a System Message; when the toggle is off it stops at the clear. | `ActionPipeline::rewrite_options_after_turn` | known; `options_tests::test_always_on_*`, `test_toggle_off_clears_set_after_turn` | loose R-31, R-32, R-33 |
| 8.7 | The toggle is read from the Game row and falls back to the World's value when the row is unreadable. | `PipelineRun::resolve_options_always_on` | known; `options_tests::test_unreadable_game_row_falls_back_to_world_toggle` | loose R-31 |
| 8.8 | An impersonate turn skips the turn-end rewrite entirely — including when its generation saved an event Narration (the event branch runs, then the `if !impersonated` guard skips the options run). | `ActionPipeline::run_from_input` | known; `options_tests::test_always_on_skipped_after_impersonate_turn`, `tests/http/options.rs::test_impersonate_keeps_offered_set_http` | **contradicts R-31** (an event Narration triggers the run) |
| 8.9 | The Options prompt is the active options preset text (fallback built-in) as system prompt plus room name/description/navigation, the scene NPCs (id, name, description), the last 4 Messages, the player name, the option count and a fixed closing instruction. | `OptionsPromptBuilder::build`, `generate_options` | known; `options/prompt_tests` | absent (black box §3.3 does not state the prompt) |
| 8.10 | The offered set is stored as a `Vec<String>` inside the snapshot (`current_options`), not as its own record, so it travels with every snapshot write and is restored by any state load. | `NarrativeState::current_options`, `NarrativeSnapshot` | known | absent (record) |

## 9. Edit, delete, reset, create, switch

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 9.1 | Edit replaces the text of the **active Swipe of any Message, by id** (not only the last), with no validation of the text (empty text is accepted), no busy check, and no change to the status. An unknown id answers 500. | `MessageService::edit_history`, `MessageHistory::edit`, `edit_history_handler` | known; `message_service_tests::test_edit_history_updates_text_and_snapshot` | **contradicts R-70**; loose R-75 |
| 9.2 | Edit writes when a snapshot exists; with no snapshot the write is skipped and the handler still answers 200 "Edited". | `MessageService::edit_history` (`if latest.is_some()`) | known | absent |
| 9.3 | Delete removes the last Message **even when it is the only Message**, leaving an empty story; the failure line is not cleared and the delete writes a snapshot carrying the old status forward. | `MessageService::delete_last`, `MessageHistory::delete_last`, `delete_history_handler` | known | **contradicts R-76** (never the only Message), **contradicts R-70** (busy), **contradicts R-21a** (a delete clears a failure) |
| 9.4 | **Corrected.** Delete is a hard `DELETE` of the Message row; its Swipes go by `ON DELETE CASCADE`; its Snapshot rows stay (research 07 open point 5). The in-memory history pops. | `Storage::delete_message`, `MessageHistory::delete_last` | known | absent (storage detail) |
| 9.5 | Create/reset build the fresh Game's preset ids from the **current settings mode registry** (system/quantifier/impersonate) and `settings.active_options_prompt_preset_id`, while mode, posture and the options toggle come from the World. | `GameCatalogue::create_game`, `GameCatalogue::reset` | known | absent (the spec states only World + Persona) |
| 9.6 | The fresh Game's name is always regenerated: `generate_game_name(world_name, existing)` = `{world}_{YYYY-MM-DD}_{max+1}`, where create passes the names of **all** games and reset passes only the names of games of the **same World**. | `generate_game_name`, `GameCatalogue::create_game`, `GameCatalogue::reset` | known; `catalogue_tests` | absent |
| 9.7 | When the World's scenario text is empty, create/reset produce **no Message at all** — the story is empty (not an opening Narration with empty text). | `MessageService::build_fresh_initial_state`, `GameCatalogue::persist_initial_state_with_swipes` | known; `games_create.rs` fixtures | **contradicts R-77** ("When the World's opening text is empty, the opening Narration has empty text") |
| 9.8 | With a non-empty scenario text, the opening Narration is the scenario text rendered with `{{user}}` and its room report is the starting room's name. | `MessageService::build_fresh_initial_state`, `GameState::inject_scenario_logs` | known | loose R-25a, R-77 |
| 9.9 | Create sets the new Game active, persists the opening, and restores the previous current game id when that persistence fails. | `GameCatalogue::create_game` | known | loose R-78 |
| 9.10 | Reset removes the active Game row with all its records and creates a new one with the **same** World key and Persona key. | `GameCatalogue::reset` | known; `tests/http/reset.rs::test_reset_clears_history_http` | loose R-77 |
| 9.11 | Switch validates the Game exists (400 "Game not found"), sets the process-wide current game id, then heals that Game's stale status. | `GameCatalogue::switch_game`, `switch_game_handler` | known; `tests/http/games_switch.rs` | loose R-79, R-80 |
| 9.12 | `POST /reset`, `POST /games` and `POST /games/:id/switch` answer 200 with an `HX-Refresh: true` header and an empty body; a reset failure answers 500 with the message. | `reset_handler`, `create_game_handler`, `switch_game_handler`, `ok_refresh` | known | loose §2 ("the dashboard reloads") |
| 9.13 | The active-Game pointer is process-wide (`Storage::current_game_id`), set at bootstrap from the newest Game of the configured World and by create/reset/switch; every read and write resolves it at call time. | `Storage::current_game_id`, `resolve_game_id` | known | **contradicts R-74** (see 1.6) |
| 9.14 | At startup the engine picks the newest existing Game of the configured World (last message timestamp, then game id) and creates one only when none exists. | `resolve_game_id`, `find_latest_game_for_world` | known; `tests/bootstrap/run_branches.rs` | absent (outside the spec per the target design's D-26) |

## 10. Status, failure display, views, HTTP surface

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 10.1 | `GET /status/generating` answers, in order: the failure disclosure (short line + `Details` popover with the raw text) when the loaded state holds a failure; else the live phase string; else `idle`. A state-load failure answers `idle`. It never answers "busy". | `generating_status_handler` | known; `tests/http/dashboard.rs::test_generating_status_answers_from_live_registry` | loose R-21, R-71, R-72 |
| 10.2 | Phase strings are `narrating`, `quantifying`, `generating-event`, `options`. | `GenerationPhase::as_endpoint_str` | known | loose R-71 |
| 10.3 | There are 8 failure kinds (`Unreachable`, `PromptTooLong`, `UnreadableAnswer`, `TokenBudgetSpent`, `SaveFailed`, `SceneMissing`, `PresetMissing`, `Other`) with fixed one-line summaries; there is **no** empty-text kind, the save line is "The last turn could not be saved." and the default line is "The last turn failed to generate." | `GenerationFailureKind`, `generation_failure_summary` | known; `tests/http/failure_display.rs` | **contradicts R-19, R-21** (an empty-text line, and the design's wording) |
| 10.4 | Error classification maps engine errors to kinds (`ContextOverflow`/`PromptBuild` → `PromptTooLong`; `ParseError`/`EmptyResponse`/`Generation` → `UnreadableAnswer`; `Database`/`Io` → `SaveFailed`; `RoomNotFound` → `SceneMissing`; `PresetNotFound` → `PresetMissing`; everything else, including `GameNotFound` and `Config`, → `Other`). | `GenerationFailure::from_engine_error` | known; `generation_status_tests` | loose R-21 |
| 10.5 | The raw error line carries a step prefix added by `with_context` ("Trigger narration failed: …", "Retry failed: …"). | `GenerationFailure::with_context` | known | absent |
| 10.6 | The status lives inside every snapshot (`input_buffer.status` + `phase`), so each save writes the status again and every load restores it. | `NarrativeSnapshot::input_buffer`, `MessageService::write_snapshot` | known; `research 04` | loose §7 free point (R-90 still applies) |
| 10.7 | The switch-heal rewrites the persisted status only when the heal changed it. | `ActionPipeline::heal_stale_if_changed` | known | absent |
| 10.8 | The story-log fragment renders the **first 50** Messages of the ascending list (`take(MAX_LOG_DISPLAY)`), so a story longer than 50 hides the newest Messages — and with them the Delete, Retry/Swipe and Retrigger controls, which render on the last **rendered** entry only. | `AppState::render_story_log`, `NarrativeLogTemplate::new` | known | **contradicts §3.7** (the controls are on the last entry; the log "shows each Message") |
| 10.9 | The Delete control renders when `loop.last && entries.len() > 1`; the Retrigger control when the last rendered entry is a Narration without an event header and the state's `last_trigger` is set. | `NarrativeLogTemplate` source, `MessageEntryView`, `NarrativeLogTemplate::new` | known; `tests/http/story_log.rs` | loose §3.7 |
| 10.10 | A `System` Message renders as a `system` log type; when it is the last entry, the swipe/Retry controls and any Retrigger do not render. | `MessageEntryView::from`, `NarrativeLogTemplate` | known; `options_tests::system_messages_contain` | absent |
| 10.11 | The dock renders nothing at all when the offered set is empty — there is no notice element or notice record anywhere in the code; the controls are disabled while the Game is busy. | `AppState::render_options_dock`, `OptionsDockTemplate`, `OptionsDockViewModel` | known; `tests/http/options.rs::test_narration_turn_clears_offered_set_http` | **contradicts R-85** (a notice in the dock in place of the set) |
| 10.12 | The header renders the game display name plus one row per LLM role (narrator, quantifier, options) taken from the newest forensics row per role, globally across Games, from the last 50 forensics rows; each row can show backend+model and the last error, with a Details toggle. | `AppState::render_header`, `GameViewQuery::role_health`, `header_fragment_html`, `LLM_MESSAGE_RETENTION_LIMIT` | known; `tests/http/llm_messages.rs` | loose R-87 |
| 10.13 | `GET /fragment/llm-messages` renders the latest 50 forensics rows with pretty-printed raw request/response JSON. | `AppState::render_llm_messages` | known; `tests/http/llm_messages.rs` | absent |
| 10.14 | Forensics: one row per call is saved by the recorder; a **failed** attempt also saves a row (with a blank raw request, the error text and the raw response body when the transport had one). `parsed_response` holds the sanitised text. | `LlmCallRecorder::complete`, `failed_message` | known; `tests/storage/llm_message_storage.rs` | loose R-87 (record shape) |
| 10.15 | `POST /action` answers `200` with `<span class="status thinking">Thinking...</span>` when started, `200 "Still thinking..."` when busy, `503 "Server is shutting down"` while shutting down, and `500` with "Failed to process action: …" for any other error — including no Game (`GameNotFound`), which the spec wants as `400`. | `dispatch_action`, `action_handler`, `ApplicationError::into_response` | known; `actions_tests` | **contradicts R-45** |
| 10.16 | There are three action routes: `/action` (dispatch), `/action/check` (text-check preview when enabled and the command is not an engine command) and `/action/confirm` (dispatch plus status-swap headers). The spec knows only `POST /action`. | `build_router`, `action_check_handler`, `action_confirm_handler`, `dispatch_with_status_headers` | known; `tests/http/requires_migration/text_check.rs`, `tests/http/actions.rs::test_confirm_retargets_status_display` | absent (text-check itself is out of scope, the split is not) |
| 10.17 | A guided or `/impersonate` command bypasses the text check; an unknown slash command and ordinary text do not. | `Action::is_engine_command`, `action_check_handler` | known; `tests/http/actions.rs::test_slash_command_bypasses_text_check_http` | absent |
| 10.18 | `POST /swipe/new` answers `200 "Retrying..."` when started, `200 "Still thinking..."` when busy, `503` while shutting down, `400` for a validation error and `500 "Internal Server Error"` for an internal one. | `map_process_action_result`, `retry_handler` | known; `tests/http/swipe_new.rs::test_retry_concurrent_generation_returns_still_thinking_http` | loose §2 |
| 10.19 | `POST /retrigger` answers on the same pattern with "Retriggering...". | `map_process_action_result`, `retrigger_handler` | known; `tests/http/retrigger.rs::test_retrigger_concurrent_generation_returns_still_thinking_http` | loose §2 |
| 10.20 | `POST /message/:id/swipe/:index` while busy answers **`503` "Generation in progress, please wait..."**, not `200 "Still thinking..."`. | `switch_swipe_handler`, `MessageService::switch_swipe` (`ConcurrentGeneration`), `ApplicationError::into_response` | known | **contradicts R-70** |
| 10.21 | The swipe switch on a non-last Message answers `400 "Only the last message can be swiped"`; an out-of-range index is written to `active_swipe_index` **before** the bounds check, then answers `500 "Internal Server Error"`; a swipe with no snapshot id answers 500. | `MessageService::switch_swipe` | known; `tests/http/swipe_switch.rs` | absent (the pre-write and the codes) |
| 10.22 | A successful swipe switch copies the target Swipe's snapshot into a **new** snapshot row with status `Idle`, phase `Narrating`, options cleared and `created_at = now`, then answers with the story-log fragment. | `MessageService::switch_swipe` | known; `tests/http/swipe_switch.rs::test_switch_swipe_leaves_the_game_idle_http`, `_drops_the_offered_option_set_http` | loose R-37, R-38; absent (the copy) |
| 10.23 | `POST /history/:id` answers `200 "Edited"`; `POST /history/delete` answers an empty `200`. | `edit_history_handler`, `delete_history_handler` | known; `tests/http/story_log.rs` | loose §2 |
| 10.24 | `POST /status/reset-generating` answers `"reset"` or `"failed"` as plain text and does not distinguish busy from idle. | `reset_generating_handler` | known | loose R-81 |
| 10.25 | The debug busy route answers `"true"`/`"false"` from the registry; `GET /debug/state` answers a JSON view with the room id, the scene Character ids, status, phase, the encounter log, the last 5 history entries, the history length, the dynamic-room list and count, the last raw error, the stored confidence, the last backend/model names and the offered set; `GET /debug/backend` answers the resolved provider name and model. | `debug_is_generating_handler`, `GameViewQuery::get_debug_state`, `debug_backend_handler` | known; `tests/http/requires_migration/debug.rs` | loose §2 (the debug route) / absent for the shape |
| 10.26 | `POST /reset` on a World/persona problem answers `500` with the message text (not a user-facing 400). | `reset_handler` | known; `tests/http/requires_migration/core.rs` | absent |

## 11. Storage, records, restart

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 11.1 | A save is a sequence of separate statements, not one unit: the deferred Swipe location update, then a **new snapshot row** (full state, including status and options), then the retry target's Swipe snapshot id and active index, then each unpersisted Message row and all its Swipe rows. | `MessageService::save_message_and_snapshot`, `write_snapshot`, `save_arriving_location` | known; `tests/storage/snapshot_storage.rs`, `message_storage.rs` | **contradicts R-90** ("one save of a Message, its Swipe and its state is one unit") |
| 11.2 | A Message row carries its kind, timestamp, active swipe index and a soft-delete flag; a Swipe row carries text, its snapshot id, the room report, the event header and the stored inputs. | `Message`, `Swipe`, `MessageEntry` | known | loose §2.1 of the design (the spec leaves the records to the design) |
| 11.3 | The game state has no separate live record: the newest snapshot row **is** the state, and the "latest snapshot" is re-read for every request. | `Storage::load_latest_snapshot`, `MessageService::load_expecting_valid_state` | known; `research 04` | loose R-39 (the spec does not say where state lives) |
| 11.4 | A state-load failure is swallowed: the engine logs, rebuilds a "fresh initial state" from the World, and if that also fails returns an empty `GameState::new("")`. | `MessageService::load_or_fresh` | known | **contradicts R-46, R-91** (missing data is a data-integrity failure) |
| 11.5 | On load, an out-of-range `active_swipe_index` silently falls back to 0 with a warning. | `Storage::load_messages_with_swipes`, `Message::ensure_valid_swipe_index` | known | absent |
| 11.6 | The in-memory history caps at 1000 Messages on append and drops the oldest; the storage rows are unaffected and return on the next load. | `MessageHistory::append`, `MAX_MESSAGES` | known; `game_state_tests::prop_log_turns_never_exceed_max_capacity` | absent |
| 11.7 | The prompt's history layer is capped at `MAX_HISTORY_TOKENS` (16000) by keeping the **last** 64000 characters of the labelled history text, which can cut mid-line; the fitted user prompt then drops whole history lines oldest-first to fit the connection's context budget (with a "(History truncated to fit context window)" marker when it drops everything). | `truncate_to_budget`, `LayerRenderer::render_history_layer`, `fit_messages_to_context`, `trim_history_to_budget` | known; `token_budget_tests`, `context_tests` | absent (§3.5 free point; the spec leaves "how much of the story each call reads" to the design) |
| 11.8 | Context fitting fails the call with `ContextOverflow` when the system prompt plus the safety margin plus the minimum input budget exceeds the connection's context window; otherwise it shrinks `max_tokens` to fit. | `fit_messages_to_context`, `PromptAssembler::resolve_budget` | known; `context_tests` | loose §3.1 (the "prompt is too long" failure) |
| 11.9 | After a restart: the registry is empty, so no Game is busy; the persisted status may still read `Generating` and `GET /status/generating` reports `idle`, the persisted failure line and offered set stay. | `GenerationGate::new`, `generating_status_handler` | known; `tests/http/dashboard.rs::test_generating_status_answers_from_live_registry` | loose R-21a, R-92, R-93 |
| 11.10 | A `last_backend_name` and `last_model_name` pair is stored in every snapshot and shown by `GET /debug/state`. | `NarrativeState::last_backend_name`, `ActionPipeline::run_from_input` | known | absent (record) |
| 11.11 | `InputBuffer::input` is a stored field that nothing outside tests ever writes. | `InputBuffer::input` | known | absent (dead field) |

## 12. Startup and arrival narration

| # | Behaviour | Deciding symbol | Label | Spec status |
|---|---|---|---|---|
| 12.1 | At startup, when the World's scenario text is **empty** (and only then), an arrival task runs and writes an LLM-generated opening Narration into the story. | `spawn_arrival_task_if_needed`, `ArrivalTaskContext::run_inner` | known; `tests/bootstrap/run_branches.rs` | **contradicts R-77** ("No LLM call writes an opening when a Game is created or reset") |
| 12.2 | The arrival task runs **outside** the generation gate and outside any work entry: the Game is not registered busy while it runs, so another Action can be accepted concurrently. **Corrected:** the persisted status does not read `Generating`; arrival sets it in memory only and saves once at the end. | `spawn_arrival_task_if_needed`, `ArrivalTaskContext::run_inner` | known | **contradicts R-72** (the status must agree with whether work runs) |
| 12.3 | The arrival task re-loads the state inside the task, force-injects the scenario logs when the history is empty, sets status `Generating`, and on a missing preset or a failed call stores a failure status but still saves. | `ArrivalTaskContext::run_inner`, `GameState::inject_scenario_logs` | known | absent |
| 12.4 | The arrival prompt is built with an **empty** user message and an empty history, the active system preset and the connection's token budget. | `ArrivalTaskContext::run_inner` | known | black box §3.1 (the prompt content) |
| 12.5 | When no snapshot row exists at startup, the engine writes an initial snapshot plus the scenario opening (same rule as create), and only then does the arrival check run. | `init_game::load_game_state`, `try_persist_initial_message` | known | loose R-77 |

---

## 13. Counts

A row with two statuses is counted once, under its strongest status.

| Area | Rows | absent | loose | black box | contradicts |
|---|---|---|---|---|---|
| 1. Pipeline / work / busy / stop | 10 | 2 | 5 | 0 | 3 |
| 2. Narration generation | 15 | 4 | 6 | 1 | 4 |
| 3. Analysis | 10 | 6 | 0 | 2 | 2 |
| 4. World rules / movement / encounters | 12 | 3 | 7 | 0 | 2 |
| 5. Triggers | 12 | 3 | 7 | 1 | 1 |
| 6. Retry | 10 | 2 | 8 | 0 | 0 |
| 7. Retrigger | 5 | 0 | 3 | 0 | 2 |
| 8. Options | 10 | 4 | 2 | 0 | 4 |
| 9. Edit / delete / reset / create / switch | 14 | 5 | 5 | 0 | 4 |
| 10. Status / views / HTTP | 26 | 8 | 13 | 0 | 5 |
| 11. Storage / restart | 11 | 5 | 4 | 0 | 2 |
| 12. Startup / arrival | 5 | 1 | 1 | 1 | 2 |
| **Total** | **140** | **43** | **61** | **5** | **31** |

The 31 "contradicts" rows are the ones a spec-driven rewrite would change; the 43 "absent" and 61 "loose" rows are the oracle gap proper.

## 14. The highest-value rows

Ranked by "a rewrite tested against the clean spec would silently lose or change this":

1. **10.8 / 10.9** — the story log renders only the first 50 Messages and puts every control on the last rendered one. The spec says the log shows each Message. A rewrite that follows the spec drops no behaviour, but a rewrite that keeps today's tests must keep the cap.
2. **10.11 / 8.4 / 8.5 / 3.6 / 4.4** — five distinct `[System]` Messages are written into the story (options failure, options unavailable, uncertain NPC detection, unknown location) and there is no notice record at all. The spec's notice lives in the dock and is not a Message.
3. **9.7 / 12.1** — an empty World opening produces an empty story at create/reset, and an LLM-generated arrival Narration at startup.
4. **9.3** — delete can empty the story (and 9.1 edits any Message with no busy check while the spec refuses both while busy).
5. **1.1** — a stop does not stop the work; its later writes still land.
6. **10.15 / 10.20** — `POST /action` with no Game answers 500, and a swipe switch on a busy Game answers 503; the spec fixes both as 200 "Still thinking..." / 400.
7. **5.7** — a matched Trigger whose event request cannot be built is dropped in silence.
8. **6.2 / 6.3 / 6.4** — the retry anchor is the last `Input` row anywhere, and the redo's prompt is truncated there.
9. **2.5 / 2.1 / 2.2** — `{{…}}` in player text becomes `[FILTERED]`, and every LLM answer is sanitised for all roles before its empty/parse check.
10. **11.1 / 11.4** — no single save unit, and a state-load failure silently rebuilds a fresh state.
