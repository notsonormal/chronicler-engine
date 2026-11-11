# Research 10: the test coverage of generation states, Snapshot pointers, retry forms and game-state writers

Ticket: [Research: map the test coverage of generation states and retry forms](../issues/10-research-test-coverage.md).

Labels: **known** = read directly in the cited test or code. **tested** = an assertion found that proves the fact. **inferred** = reasoning from cited code/test, not run. Rows and pointer names (`Sc`, `S0`, `S1`, `S1'`, `S2`, `S2b`, `S3`, `S4`, `S5`, `S5'`, `S6`, `Sf`, `Sr`, `So0`, `So1`, `Si`, `Sa`, `Ss`) come from [research 04](research-04-generation-snapshots.md) and are not renamed here. Writer names come from [research 05](research-05-game-state-writers.md) §2.

## 0. Method and the commands to re-run

This asset was written by reading, in this order: the ticket, [seed-evidence.md](seed-evidence.md), research 04, 05, 06 and 07; `tests/AGENTS.md`; `tests/STRATEGY.md`; `src/test_support/quantifier.rs`; `src/adapters/driven/llm/providers/mock.rs`; `src/adapters/driven/utils/mock.rs`; `src/test_support/context.rs` and `src/test_support/fixtures.rs`; then the tests named in the ticket's scope.

Search commands actually used (re-runnable; `grep` and `rg` are equivalent here):

```bash
# scenario tags + test names per in-scope file
grep -n -E "^(async )?fn |SCENARIO:|#\[test\]|#\[tokio::test\]" tests/http/{actions,reset,retrigger,swipe_new,swipe_switch,options,story_log,narrator_mode,failure_display,dashboard}.rs

# state-asserting reads in the whole HTTP tier (room, NPCs, pointers)
grep -rn "get_debug_state\|current_room_id\|load_latest_snapshot\|load_snapshot_by_id\|snapshot_id\|npcs_in_area\|npc_encounter_log\|times_met\|last_trigger" tests/http/*.rs tests/http/requires_migration/*.rs

# the same for the unit tier
grep -rn "snapshot_id\|load_snapshot_by_id\|load_latest_snapshot\|active_swipe" src/application/pipeline/action_pipeline/*_tests.rs src/application/pipeline/narration_generation_tests.rs src/application/message_service_tests.rs src/application/generation/*_tests.rs

# arrival reachability
grep -rn "arrival\|ArrivalTaskContext\|spawn_arrival_task_if_needed" tests/ src/test_support/
grep -rn "ArrivalTaskContext\|spawn_arrival_task_if_needed" src/ --include=*.rs

# the reset/heal endpoints
grep -rn "reset-generating\|reset_persisted_status\|heal_stale" tests/ src/ --include=*.rs

# browser tier: any restored-state assertion
grep -rn "current_room_id\|npcs_in_area\|snapshot\|location_header" tests/browser/ --include=*.rs

# scenario text for the two seed defects
grep -n "22\.1\|22\.2\|I\.6\|quantifier" docs/specs/swipe_new.md
```

Two facts about the harness that frame every verdict below (both **known**):

- **The status endpoint is a display read for busy-ness.** `generating_status_handler` returns `"idle"` whenever no live registry slot exists and the persisted status carries no failure; it returns the phase string from the *persisted* record only while a slot is live (`tests/http/dashboard.rs` SCENARIO 39.1 pins exactly that: "stale Generating should answer idle"). Therefore every `assert_eq!(fetch_generating_status(&app).await, "idle")` in `tests/http/` is a **display** assertion — the registry, not the restored Snapshot. It returns the failure text from the persisted status, so an *error* string read through this endpoint **is** a state assertion.
- **`MessageService::load_messages` / `load_or_fresh` / `load_expecting_valid_state` / `GameViewQuery::get_debug_state` / `get_current_options` are read seams** (`tests/STRATEGY.md`, Domain outcome). `Storage::load_latest_snapshot` / `load_snapshot_by_id` / `load_messages_with_swipes` are driven-adapter reads, allowed at the unit and driven-adapter tiers only.

## 1. Coverage matrix

### 1a. Generation states

Verdict column: **state** = reads the restored/persisted state through a read seam (or, at the unit/storage tier, through the service or `Storage`); **display** = reads HTML, a header, or the registry-backed status endpoint.

| State | Test | Tier / kind | Scenario tag | Verdict |
|---|---|---|---|---|
| `Idle` persisted at turn end (`Sf`) | `test_retry_flow_narration_mode_makes_new_swipe` | unit (`retry_tests.rs`) | untagged/unit — name describes the behaviour | **state** — `load_or_fresh().narrative.input_buffer.status` |
| `Idle` persisted at turn end | `test_cancellation_resets_state_to_idle` | unit (`core_tests.rs`) | untagged/unit | **state** |
| `Idle` persisted at turn end | `test_switch_game_heals_stale_status_for_the_current_game` | unit (`core_tests.rs`) | untagged/unit | **state** — `load_expecting_valid_state` |
| `Idle` persisted at turn end | `test_reset_generating_status_sets_idle` | unit (`gate_tests.rs`) | untagged/unit | **state** |
| `Idle` persisted, HTTP route | `test_switch_game_heals_stale_generating_status` | http tier-1 (`games_switch.rs`) | `docs/specs/games.md` **18.3** | **state** — `load_expecting_valid_state` after the switch |
| `Idle` after every generation/retry | `test_retry_replaces_narration_with_new_swipe_http`, `test_event_retry_replaces_event_narration_with_new_swipe_http`, `test_retry_re_impersonate_generates_input_swipe_http`, `test_retry_user_regen_generates_input_swipe_http`, `test_retry_completes_when_quantifier_returns_no_movement_http`, `test_switch_swipe_leaves_the_game_idle_http`, `test_retrigger_creates_new_event_narration_no_rollback_http`, and ~30 more (grep `fetch_generating_status`) | http tier-1 | 9.1, 10.1, 22.1, 22.2, 9.6, 36.1, 13.1, … | **display** — registry-backed endpoint; a stale persisted `Generating`/`Quantifying` answers `idle` (pinned by 39.1), so these cannot fail for a status-write defect |
| `Generating` at claim (`Sc`) | — | — | — | **no test** reads the persisted `Generating` written by `try_claim`. `test_pipeline_persists_input_before_narration`, `test_input_persisted_before_narration_http` (1.2) assert the **Input row**, not the status |
| `Narrating` / `Quantifying` phase mid-flight (`S1`→`S2`) | `test_phase_transitions_to_quantifying_during_post_generation` | unit (`action_tests.rs`) | untagged/unit | **state** — `load_or_fresh().phase == Quantifying` while the quantifier is blocked on a barrier |
| `Quantifying` phase mid-flight | `test_narration_saved_before_quantifying_phase` | unit (`action_tests.rs`) | untagged/unit | **state** — persisted phase + Narration row, mid-flight |
| `Narrating` phase kept on narration failure | `test_pipeline_phase_stays_narrating_on_narration_failure` | unit (`core_tests.rs`) | untagged/unit | **state** — persisted phase and `status.failure()` |
| `GeneratingEvent` (`S4`) | `test_pre_event_snapshot_saved_before_continuation` | unit (`core_tests.rs`) | untagged/unit | **display-only equivalent** — asserts only `storage.load_latest_snapshot().db_id.is_some()`; the test is named for `S4` but reads no phase and no pointer |
| `Error` (any cause) | `test_pipeline_returns_error_on_narration_failure`, `test_pipeline_empty_narration_sets_error_message_and_no_narration`, `test_pipeline_room_not_found_sets_error_status`, `assert_error_status` callers in `retry_tests.rs` (`test_retry_room_not_found_sets_error`, `test_retry_llm_error_sets_error`, `test_retry_empty_narration_sets_error`, `test_retry_user_regen_missing_target`, `test_retry_user_regen_narrator_fails`, `test_retry_renarrate_with_no_input_persists_error`, `test_retry_returns_internal_error_when_anchor_has_no_snapshot_id`, `test_retry_returns_internal_error_when_snapshot_row_missing`) | unit | untagged/unit | **state** — `load_or_fresh().status` matched against `GenerationStatus::Error` with a substring |
| `Error` through the endpoint | `test_generation_error_clamps_with_raw_text_in_disclosure`, `test_missing_preset_failure_clamps_with_the_preset_line` | http tier-1 (`failure_display.rs`) | `docs/specs/failure_display.md` **38.3**, **38.5** | **display** for the clamped line and the disclosure markup, but the state is *arranged* directly via `TestAppBuilder::generation_status`, not produced by a generation — **inferred**: these pin the rendering, not any writer |
| `Error` after a failed retry | `test_retry_llm_failure_sets_error_http`, `test_retry_empty_narration_sets_error_http`, `test_retry_room_not_found_sets_error_http`, `test_event_retry_trigger_narration_failure_sets_error_http` | http tier-1 (`swipe_new.rs`) | `docs/specs/swipe_new.md` **11.5**, **11.6**, **11.7**, **11.8** | **display** — the error text is read from `/status/generating`; since the endpoint reads the persisted failure, this is a state assertion *through a driving adapter* |
| `Error` after a failed Trigger in an Action / a failed Retrigger | `test_trigger_narration_failure_preserves_main_sets_error_http`, `test_retrigger_trigger_narration_failure_sets_error_http` | http tier-1 (`actions.rs`, `retrigger.rs`) | `docs/specs/actions.md` **2.4**; `docs/specs/retrigger.md` **14.5** | **display** for the error line + **state** for the preserved rows (`load_messages`) |
| Options phase (`So0`, `So1`) | — | — | — | no test reads `GenerationPhase::Options` from the persisted record. `test_process_options_replaces_set` asserts the option set, not the phase |

### 1b. Snapshot states and pointers

"Pointer" = a `Swipe::snapshot_id`. Per research 04 R2, only some states ever become a pointer. The **Restored-state assertion** column asks whether any test loads that state and compares a field of the restored world (room, NPC set, options, `last_trigger`, status).

| State | Is it a pointer? | Test that loads it | Tier / kind | Scenario tag | Verdict |
|---|---|---|---|---|---|
| `Sc` (claim-time; free-action Input) | yes | `test_retry_uses_edited_input_text_http` (9.4) reads the Input's text; `test_failed_retry_does_not_roll_the_movement_back_http` (11.9) reads the room after a failed redo | http tier-1 | 9.4, 11.9 | 11.9 is **state** for the room (`get_debug_state().current_room_id == "room2"`); 9.4 is **state** for the Input text. No test compares a *restored* `Sc` field to the click-time field |
| `S0` (pre-main) | **no** — `save_message_and_snapshot` sets nothing there (research 04, Correction) | `test_pre_main_snapshot_saved_before_narration` | unit (`core_tests.rs`) | untagged/unit | **not a state assertion** — only `load_latest_snapshot().db_id.is_some()`. The test's name claims the `S0` save; the assertion would pass for any later save. **Gap** |
| `S1` (pre-quantifier Narration / impersonate Input) | yes | `test_retry_flow_guided_turn_retries_without_older_input_text` (forensics prove the redo read the guided Narration's Snapshot: `Current Location: Marker Room`); `test_guided_retry_reports_the_room_its_redo_arrives_in_http` (22.5); `test_retry_re_impersonate_generates_input_swipe_http` (22.1) | unit; http tier-1 | untagged/unit; 22.5, 22.1 | 22.5 and the unit guided test are **state** (room / prompt location). 22.1 only restores it implicitly and asserts swipes/text — **no restored-state assertion**. No test compares the restored world to the pre-turn world, and none asserts that `S1` is *pre*-quantifier |
| `S1'` (retry Swipe) | yes | `test_retry_appends_swipe_to_same_message`, `test_event_retry_redo_swipe_names_no_room_of_its_own`, `test_retry_reruns_quantifier_moves_player_http` (9.2) | unit; http tier-1 | untagged/unit; 9.2 | **state** for the Swipe row and active index; 9.2 additionally asserts the room the redo arrived in. No test reads `S1'` back as a Snapshot |
| `S2` (pre-quantifier phase update) | no | `test_phase_transitions_to_quantifying_during_post_generation` | unit | untagged/unit | **state** for the phase; no pointer read |
| `S2b` (post-quantifier metadata) | only for System Messages | `test_pipeline_continues_if_quantifier_save_fails` | unit | untagged/unit | **state** weakly — asserts the run completes; not the metadata save |
| `S3` (post-engine) | only for the unknown-room System Message | `test_pipeline_quantifier_detects_movement` | unit | untagged/unit | **state** — room after the engine commit. The `S3` Snapshot row itself is never read back |
| `S4` (pre-event) | no | `test_pre_event_snapshot_saved_before_continuation` | unit | untagged/unit | **not a state assertion** (see 1a). **Gap** |
| `S5` (post-trigger event Message) | yes | `test_retry_target_and_anchor_agree_when_a_system_notice_trails_an_event_narration` loads the event Message's pointer and reconstructs; `test_event_retry_does_not_carry_the_previous_swipes_encounter_http` (10.5) | unit; http tier-1 | untagged/unit; 10.5 | 10.5 is **state** — `times_met == 0` and `npcs_in_area` empty after the event retry. The unit test is **state** for the reconstructed history, not the world |
| `S5'` (event retry Swipe / retrigger event Message) | yes | `test_event_retry_replaces_event_narration_with_new_swipe_http` (10.1), `test_event_retry_after_retrigger_adds_a_swipe_http` (10.4), `test_retry_flow_event_mode_continues_trigger` | http tier-1; unit | 10.1, 10.4 | **state** for Swipe count / active index. No test reads the `S5'` Snapshot back |
| `S6` (Options rewrite) | only for option-failure System Messages | `test_always_on_generates_options_after_narration_turn_http` (24.6), `test_narration_turn_clears_offered_set_http` (24.16), `test_failed_always_on_regeneration_clears_set_and_reports_http` (24.17), `test_always_on_without_an_agent_empties_set_and_reports_http` (24.20) | http tier-1 | 24.6, 24.16, 24.17, 24.20 | **state** for the persisted option set (`get_current_options`) plus **display** (`/fragment/options-dock`). The Snapshot row and its phase are not read |
| `Sf` (finalize) | no | `test_retry_flow_*` (all four assert final `Idle` through `load_or_fresh`) | unit | untagged/unit | **state** for status; the row itself is not read |
| `Sr` (click-time restore after a failed redo) | no | `test_failed_retry_does_not_roll_the_movement_back_http` (11.9), `test_event_retry_trigger_narration_failure_sets_error_http` (11.8) | http tier-1 | 11.9, 11.8 | **state** — room and fired-trigger mark survive. No test asserts the *pre-Swipe* rollback branch (spec 11.10 covers the other branch and is tested by `test_retry_that_wrote_its_narration_keeps_that_world_http`) |
| `So0` / `So1` (on-demand Options) | no | `test_process_options_replaces_set`, `test_process_options_empty_history_rejected`, `test_process_options_failure_keeps_prior_set` | unit (`options_tests.rs`) | untagged/unit | **state** for the option set; the pre-options Snapshot is not read |
| `Si` (scenario start) | yes — Scenario Narration | `test_create_game_handler_http` (17.1), `test_reset_clears_history_http` (7.1), `test_reset_then_execute_works_http` (7.2) | http tier-1 | 17.1, 7.1, 7.2 | **state** for the stored rows; 17.1's own comment states the Snapshot is asserted through `message_service_tests`, and indeed **no test reads `Si` or the Scenario Narration's pointer** |
| `Sa` (arrival) | yes — arrival Narration | `test_run_produces_and_persists_narration`, `test_run_falls_back_to_fresh_state_on_load_failure`, `test_run_returns_early_without_narration_on_world_fetch_failure` | unit (`arrival_service_tests.rs`) | untagged/unit — "unit tests for `ArrivalTaskContext`" | **state** — status, Narration row, forensics row. These call `ArrivalTaskContext::run` directly; no HTTP or concurrency test reaches `Sa` (see §4) |
| `Ss` (the copy `switch_swipe` writes) | no (new row) | `test_switch_swipe_restores_a_settled_snapshot` | unit (`message_service_tests.rs`) | untagged/unit | **state** for status, phase and `current_options`. The hand-seeded fixture points both Swipes at **one** Snapshot, so the test cannot detect that a switch restores the wrong world. **Gap** |

**Pointers no test reads, explicitly:** `S0` and `S4` are not pointers at all (research 04 R2), and no test reads their Snapshot rows either — the two tests named after them assert only that *a* latest Snapshot exists. `S1`, `S5`, `S5'` and `S1'` are pointers whose rows no test loads back except `S5` (one unit test, history-only). `Sc` is read only indirectly. `Si` and `Sa` pointers are never read back by any test.

### 1c. Retry forms

| Form | Test | Tier / kind | Scenario tag | Verdict |
|---|---|---|---|---|
| Main Retry (`ReNarrate`, main) | `test_retry_replaces_narration_with_new_swipe_http`, `test_retry_reruns_quantifier_moves_player_http`, `test_retry_preserves_input_message_http`, `test_retry_uses_edited_input_text_http`, `test_retry_reevaluates_triggers_http`, `test_retry_completes_when_quantifier_returns_no_movement_http`, `test_retry_swipe_without_movement_reports_no_room_http` | http tier-1 (`swipe_new.rs`) | 9.1–9.6, 9.8 | 9.2 and 9.5 are **state** (room, event count); 9.8 is **display** (Swipe `location_header`); 9.1/9.3/9.4 are **state** for stored rows and text |
| Main Retry | `test_retry_flow_narration_mode_makes_new_swipe`, `test_retry_appends_swipe_to_same_message`, `test_retry_recovers_after_llm_failure` | unit (`retry_tests.rs`) | untagged/unit | **state** — status, Swipe count, active index |
| Guided Retry | `test_guided_retry_reports_the_room_its_redo_arrives_in_http` | http tier-1 | `docs/specs/swipe_new.md` **22.5** | **state + display** — `guided.swipes.len() == 2` (state) and the arriving room header (display) |
| Guided Retry | `test_retry_flow_guide_only_narration_retries_with_empty_input`, `test_retry_flow_guided_turn_retries_without_older_input_text` | unit | untagged/unit | **state** — Swipe count, guided flag, and (forensics) that the redo's prompt names the guided Snapshot's room |
| Event Retry | `test_event_retry_replaces_event_narration_with_new_swipe_http`, `test_event_retry_does_not_rerun_quantifier_http`, `test_event_retry_after_retrigger_adds_a_swipe_http`, `test_event_retry_does_not_carry_the_previous_swipes_encounter_http`, `test_event_retry_after_a_deleted_action_keeps_its_own_trigger_http`, `test_event_retry_matches_with_a_prior_swipe_switch_http`, `test_event_retry_keeps_the_room_name_off_the_event_swipe_http` | http tier-1 | 10.1, 10.2, 10.4, 10.5, 10.6, 10.7, 10.8 | 10.2 is **state** (room unchanged); 10.5 is **state** (encounter log + `npcs_in_area`); 10.6/10.7 are **state** (`times_met`); the rest are **state** for rows and **display** for headers |
| Event Retry | `test_retry_event_continuation_happy_path`, `test_retry_target_and_anchor_agree_when_a_system_notice_trails_an_event_narration`, `test_retry_event_continuation_rewrites_options_when_enabled`, `test_retry_event_continuation_clears_options_when_disabled` | unit | untagged/unit | **state** — status, history, options |
| Re-impersonate | `test_retry_re_impersonate_generates_input_swipe_http` | http tier-1 | `docs/specs/swipe_new.md` **22.1** | **state** for rows/text/`impersonated`; **no assertion at all** for the scenario's "the player has not moved" line (see §3) |
| Re-impersonate | `test_retry_re_impersonate_preserves_record_http` (22.3), `test_retry_re_impersonate_uses_current_preset_content_http` (22.4) | http tier-1 | 22.3, 22.4 | **state** — stored inputs, preset content |
| Re-impersonate | `test_retry_flow_impersonate_mode_runs_full_tail` | unit | untagged/unit | **state** — Swipe count, `scene.quantifier_confidence.is_some()` (the tail ran). This is the only test that pins the tail, and it pins it through a mock quantifier |
| User regen | `test_retry_user_regen_generates_input_swipe_http` | http tier-1 | `docs/specs/swipe_new.md` **22.2** | **state** for rows/text; the "has not moved" line is unasserted (same shape as 22.1) |
| User regen | `test_retry_flow_user_regen_mode_stays_tail_less`, `test_retry_user_regen_missing_target`, `test_retry_user_regen_narrator_fails`, `test_retry_user_regen_cancelled` | unit | untagged/unit | **state** — Swipe count, `quantifier_confidence.is_none()`, error text, status |
| Retrigger | `test_retrigger_creates_new_event_narration_no_rollback_http`, `test_retrigger_does_not_rerun_quantifier_http`, `test_retrigger_trigger_narration_failure_sets_error_http`, `test_retrigger_no_trigger_context_returns_400_http`, `test_retrigger_no_messages_returns_400_http`, `test_retrigger_last_message_not_narration_returns_400_http`, `test_retrigger_last_message_is_event_continuation_returns_400_http`, `test_retrigger_no_game_context_returns_400_http`, `test_retrigger_concurrent_generation_returns_still_thinking_http` | http tier-1 (`retrigger.rs`) | 13.1, 13.2, 14.1–14.6, 15.1 | 13.2 is **state** (room unchanged); 13.1 and 14.5 are **state** for rows and the story-log fragment (**display**); the 400/409 tests are **display** |
| Retrigger | `test_retrigger_event_cancels_cleanly`, `test_retrigger_event_cancelled_at_phase_boundary`, `test_retrigger_event_emits_error_on_world_fetch_failure`, `test_retrigger_returns_concurrent_generation_when_gate_busy`, `test_retrigger_returns_shutting_down_when_token_cancelled`, `test_retrigger_heals_stale_status_before_validation_error` | unit | untagged/unit | **state** |
| Retrigger guard (`last_trigger` after a **main** Retry) | — | — | — | **does not exist.** No test asserts `last_trigger` after a main Retry (research 06, D8). `test_execute_action_clears_last_trigger` covers only the plain Action path |
| Swipe switch | `test_switch_swipe_leaves_the_game_idle_http`, `test_switch_swipe_drops_the_offered_option_set_http` | http tier-1 (`swipe_switch.rs`) | `docs/specs/swipe_switch.md` **36.1**, **36.2** | 36.1 **display** (registry-backed status); 36.2 **display** (`/fragment/options-dock`) — though the dock is rendered from `current_options`, so it is a state fact read as markup |
| Swipe switch | `test_switch_swipe_restores_a_settled_snapshot`, `test_switch_swipe_rejects_concurrent_generation` | unit (`message_service_tests.rs`) | untagged/unit | **state** — status, phase, options. Not the room or NPC set |
| Swipe switch (within another scenario) | `test_retry_swipe_without_movement_reports_no_room_http` (9.8), `test_event_retry_matches_with_a_prior_swipe_switch_http` (10.7) | http tier-1 | 9.8, 10.7 | **display** (Swipe header) and **state** (`times_met`) respectively. **Neither asserts the restored room.** |
| On-demand Options | `test_on_demand_options_render_in_dock_http`, `test_options_without_scene_history_errors_http`, `test_regenerate_replaces_set_without_history_http`, `test_failed_regeneration_keeps_prior_set_http`, `test_on_demand_options_without_an_agent_reports_unavailable_http`, `test_numbered_list_response_parses_http`, `test_options_work_in_if_mode_http`, `test_mode_switch_keeps_offered_set_http`, `test_using_offered_option_submits_as_input_http`, `test_options_dock_icons_are_defined_by_the_shell` | http tier-1 (`options.rs`) | 24.1, 24.2, 24.4, 24.5, 24.18, 24.10, 24.11, 24.19, 24.13, 24.14 | **state** (`get_current_options`) + **display** (dock markup) |
| On-demand Options | `test_process_options_replaces_set`, `test_process_options_during_shutdown_returns_shutting_down`, `test_process_options_empty_history_rejected`, `test_process_options_failure_keeps_prior_set`, `test_no_options_agent_surfaces_unavailable_message`, `test_process_options_heals_stale_status_before_validation_error` | unit | untagged/unit | **state** |

### 1d. Game-state writers (research 05 §2)

| Writer | Test | Tier / kind | Scenario tag | Verdict |
|---|---|---|---|---|
| `GenerationGate::try_claim` (Sc) | `test_input_persisted_before_narration_http` (1.2), `test_pipeline_persists_input_before_narration` | http tier-1; unit | `docs/specs/actions.md` **1.2** | **state** for the Input row. The `Generating` status it writes is never asserted |
| `GenerationGate::try_claim` (busy → 409) | `test_retry_concurrent_generation_returns_still_thinking_http` (12.1), `test_retrigger_concurrent_generation_returns_still_thinking_http` (15.1), `test_retry_returns_concurrent_generation_when_gate_busy`, `test_retrigger_returns_concurrent_generation_when_gate_busy`, `test_switch_swipe_rejects_concurrent_generation` | http tier-1; unit | 12.1, 15.1 | **display** (status string) / **state** (unit errors) |
| `heal_stale_if_changed` (pre-claim) | `test_process_action_heals_stale_status_before_validation_error`, `test_process_options_heals_stale_status_before_validation_error`, `test_retry_heals_stale_status_before_validation_error`, `test_retrigger_heals_stale_status_before_validation_error` | unit | untagged/unit | **state** — healed to `Idle` via `load_expecting_valid_state` |
| `heal_stale_status` (game switch / boot) | `test_switch_game_heals_stale_status_for_the_current_game`, `test_boot_heal_resets_stale_generating_status`, `test_switch_game_heals_stale_generating_status` | unit; http tier-1 | untagged/unit; `docs/specs/games.md` **18.3** | **state** |
| `ArrivalTaskContext::run_inner` (`Sa`) | `test_run_produces_and_persists_narration`, `test_run_falls_back_to_fresh_state_on_load_failure`, `test_run_returns_early_without_narration_on_world_fetch_failure` | unit | untagged/unit | **state**, but only in isolation (see §4) |
| `phase_pre_main_snapshot` (`S0`) | `test_pre_main_snapshot_saved_before_narration` | unit | untagged/unit | **weak** — `load_latest_snapshot().db_id.is_some()`. **Gap** |
| `NarrationGeneration::run` (`S1`/`S1'`) | `test_run_free_action_adds_narration_message`, `test_run_guided_generation_consumes_guide`, `test_run_impersonate_adds_input_message`, `test_run_bundle_load_failure_returns_fetch_failed`, `test_run_room_not_found_sets_error_status`, `test_run_missing_preset_sets_error_status`, `test_run_narrator_error_sets_error_status`, `test_run_game_changed_returns_cancelled`, `test_run_save_failure_returns_persist_failed`; `test_streaming_narration_saved_before_quantifier_complete` | unit | untagged/unit | **state** — the message row is added and persisted. No pointer assertion |
| `phase_post_generation` (`S2`, `S2b`) | `test_phase_transitions_to_quantifying_during_post_generation`, `test_pipeline_saves_narration_before_quantifier`, `test_pipeline_continues_if_quantifier_save_fails`, `test_phase_post_generation_returns_persist_error_on_pre_quantifier_save_failure` | unit | untagged/unit | **state** for the phase; the post-quantifier metadata save is only "does not fail" |
| `run_from_input` post-engine (`S3`) | `test_pipeline_quantifier_detects_movement`, `test_narration_persisted_even_if_quantifier_changes_state` | unit | untagged/unit | **state** — room after the commit |
| `phase_trigger_continuation_llm_call` (`S4`, `S5`) | `test_pipeline_trigger_happy_path`, `test_pipeline_trigger_empty_continuation`, `test_pipeline_trigger_complete_failure`, `test_trigger_continuation_save_post_trigger_error`, `test_pre_event_snapshot_saved_before_continuation` | unit | untagged/unit | **state** — `last_trigger.is_some()`, event header, narration count. The `S5` Snapshot row is not read |
| `rewrite_options_after_turn` (`S6`) | `test_always_on_generates_options_after_narration_turn`, `test_always_on_no_agent_clears_set_and_surfaces_message`, `test_always_on_skipped_after_impersonate_turn`, `test_toggle_off_clears_set_after_turn`, `test_world_toggle_does_not_enable_always_on_for_running_game`, `test_always_on_failure_clears_set_and_surfaces_system_message`; HTTP 24.6, 24.15, 24.16, 24.17, 24.20 | unit; http tier-1 | untagged/unit; `docs/specs/options.md` 24.6, 24.15, 24.16, 24.17, 24.20 | **state** — option set and System Message rows |
| `execute_options_refresh` / `finalize_options_failure` (`So0`, `So1`) | `test_process_options_replaces_set`, `test_process_options_failure_keeps_prior_set`, `test_no_options_agent_surfaces_unavailable_message` | unit | untagged/unit | **state** |
| `phase_finalize` (`Sf`) | `test_retry_flow_*`, `test_pipeline_runs_to_completion`, plus every `load_or_fresh` `Idle` assertion | unit | untagged/unit | **state** |
| `finalize_phase_error` | `test_pipeline_empty_narration_sets_error_message_and_no_narration`, `test_retry_event_trigger_narration_fails`, `test_retry_event_empty_continuation_triggers_error`, `retry_records_canonical_game_not_found_when_game_missing` | unit | untagged/unit | **state** |
| `persist_generation_error` | `test_pipeline_returns_error_on_narration_failure`, `test_retry_returns_internal_error_when_no_anchor_message`, `test_retry_records_missing_snapshot_id` | unit | untagged/unit | **state** |
| `restore_click_time_state_after_failed_redo` (`Sr`) | `test_failed_retry_does_not_roll_the_movement_back_http` (11.9), `test_retry_that_wrote_its_narration_keeps_that_world_http` (11.10) | http tier-1 | 11.9, 11.10 | **state** — room |
| `handle_cancellation` | `test_retry_user_regen_cancelled`, `test_retry_last_response_cancelled_at_phase_boundary`, `test_retrigger_event_cancelled_at_phase_boundary`, `test_pipeline_cancels_after_main_narration`, `test_pipeline_cancels_during_trigger_continuation` | unit | untagged/unit | **state** — the flipped-to game's persisted `Idle`. Per research 05 §3 pair 3 this is the write it pins; the tests frame it as "the cancelled retry resets status", so the cross-game target is **inferred** from the test's own comment and the flipping thread |
| `MessageService::switch_swipe` (`Ss`) | `test_switch_swipe_restores_a_settled_snapshot`, 36.1, 36.2 | unit; http tier-1 | untagged/unit; 36.1, 36.2 | **state** (status/phase/options) and **display** (dock). No room/NPC assertion |
| `MessageService::edit_history` | `test_edit_history_updates_text_and_snapshot`; `test_retry_uses_edited_input_text_http` (9.4) | unit; http tier-1 | untagged/unit; 9.4 | **state** — the edited text reaches history and the retry prompt |
| `MessageService::delete_last` | `test_delete_last_removes_message_and_snapshot`; `test_delete_last_between_actions_http` (8.1), `test_delete_mid_sequence_http` (8.2), `test_delete_input_then_retry_fails_gracefully_http` (8.3), `test_delete_removes_entry_from_fragment_http` (8.4) | unit; http tier-1 | untagged/unit; 8.1–8.4 | **state** — rows; `test_event_retry_after_a_deleted_action_keeps_its_own_trigger_http` (10.6) additionally drives a delete in a loop |
| `ActionPipeline::reset_persisted_status` (`/status/reset-generating`) | `test_reset_generating_status_sets_idle` (unit); `test_reset_generating_handler` (quarantine, asserts the body contains "reset" only) | unit; quarantine | untagged/unit (quarantine untagged by rule) | **state** at the unit tier; the HTTP route has **no** state-asserting test (see §4) |
| `GameCatalogue::create_game` | `test_create_game_handler` (17.1), `test_create_game_handler_unknown_world_key` (17.2), `test_create_game_handler_unknown_persona_key` (17.3) | http tier-1 | `docs/specs/games.md` 17.1–17.3 | **state** for the new game id and the scenario Message row; **display** for the `HX-Refresh` header |
| `GameCatalogue::reset` | `test_reset_clears_history_http` (7.1), `test_reset_then_execute_works_http` (7.2) | http tier-1 | `docs/specs/reset.md` 7.1, 7.2 | **state** |
| `GameCatalogue::switch_game` | `test_switch_game_handler_success` (18.1), `test_switch_game_handler_not_found` (18.2), `test_switch_game_heals_stale_generating_status` (18.3) | http tier-1 | 18.1–18.3 | **state** for the current game id and the healed status; **display** for `HX-Refresh` |
| `init_game::load_game_state` (`Si`) | `test_run_persona_not_found_after_world_fallback` and the other `tests/bootstrap/run_branches.rs` tests | bootstrap smoke | untagged | **display/response** — startup branches; no Snapshot assertion. `test_switch_game_heals_stale_generating_status` and `test_debug_state_endpoint_*` exercise boot indirectly |
| Concurrent writers (research 05 §3 pairs 1–9) | only pair 3 (`handle_cancellation`) has tests (see above). No test covers arrival vs a turn, a pipeline vs a game switch, a split save, `/status/reset-generating` vs a running task, a pre-claim copy vs a finishing turn, `heal_stale_if_changed` vs a finishing turn, `edit_history`/`delete_last` vs a running turn, or `switch_swipe` vs a new claim | — | — | **does not exist** |

### 1e. Tier coverage of the harness itself

| Harness piece | Used by | Note |
|---|---|---|
| `seed_main_narration_flow` | 11.9, 11.10, 10.4, `retrigger_tests`, `retry_tests` | **known:** it writes the Input pointer and the Narration pointer by hand and then saves the moved world — i.e. it *reproduces* research 04's P1 save order. A defect in how the pipeline sets those pointers cannot be detected by any test that uses it |
| `seed_event_flow` | 10.1, 10.2, 10.4, 10.5, 10.6, 10.7, 10.8, 11.8, `retry_tests`, `retrigger_tests` | same, for the event Message's `S5` pointer and the fired-trigger mark |
| `seed_swipe_with_stored_inputs` | `test_retry_flow_impersonate_mode_runs_full_tail`, `test_retry_flow_guide_only_narration_retries_with_empty_input`, `test_retry_flow_guided_turn_retries_without_older_input_text`, `test_retry_flow_user_regen_mode_stays_tail_less` | seeds one Message with one Swipe and an explicit pointer — the retry forms are tested against a hand-built point, not a produced one |
| `insert_message_with_swipe` | many unit and HTTP tests | copies `snapshot_id` from the Message onto the Swipe, mirroring `save_message_and_snapshot` R2 |

## 2. Masking test doubles

### 2a. `MockBackend` (`src/adapters/driven/llm/providers/mock.rs`)

**known** fixed outputs:

| Branch | Condition | Fixed output | What it hides |
|---|---|---|---|
| Narrator | `with_narrations` seeded | the seeded strings, cycled by call index | Real narration content; harmless for state |
| Narrator, unseeded | no `with_narrations` | `[MockNarration] {extracted <PlayerInput>}` | Content only |
| Narrator, empty | `with_empty_response` | `""` | Drives the error path deliberately |
| Non-narrator (quantifier, options, trigger) with a caller-seeded `with_prompt_responses` | any | the seeded string, cycled by a **shared** `call_index` | See 2b: the shared counter is what lets a test assert call *order*, and it is also what makes a mis-ordered save invisible if the answers happen to be similar |
| Non-narrator, **no** seed, `agent_name == "options"` | always | three canned `<suggestion>` lines | Nothing about state; makes the options path appear to work with no agent behaviour |
| Non-narrator, **no** seed, anything else | always | `"[Continuation: …]"` — **not JSON** | **This is the important one.** A quantifier fed this answer fails parsing, and the pipeline then falls back to `QuantifierResult::default()`: empty `npcs_in_room`, `QuantifierConfidence::Low`, **no movement**. Because `phase_post_generation` then back-fills `npc_ids` from the current `scene.npcs_in_area` (the room defaults) and appends `"[System] NPC detection uncertain — using room defaults"`, every test that uses `MockBackend::default()` for the quantifier gets: **no movement**, **room-default NPCs**, and **an extra trailing System Message** |
| Trigger branch | `with_trigger_narration_fail` / `should_fail` | error | Deliberate |

**known**: `should_fail` and `trigger_narration_should_fail` are consumed by the *dispatcher and options* branch, but the quantifier branch is exempt from `should_fail` only via `with_fail_first_n`'s doc comment ("Trigger and quantifier calls never consume this failure budget") — `should_fail() == true` **does** fail the quantifier branch. `with_fail()` on the narrator therefore also disables the quantifier, which several error tests rely on.

### 2b. The mock Quantifier

Two distinct forms:

1. **Absent agent.** `app_with_narrator` / `app_with_narrator_and_settings` (`tests/http/support/app_wiring.rs`) build the pipeline with `AgentRegistry::default()`, and `ActionPipeline::with_backends` does not add a default Quantifier. `run_post_generation_agents` therefore returns `QuantifierResult::default()` for every turn in those apps. This covers 22.1, 22.2, 22.3, 22.4 (all four re-impersonate/user-regen scenarios), 9.1, 9.3, 9.4, 1.5, 1.13, the whole `retrigger.rs` HTTP file, `narrator_mode.rs`, and `options_app()`-based tests that omit a quantifier. **Inferred consequence:** in these apps the quantifier *cannot move the player and cannot add an NPC*, so "the player has not moved" is vacuously true, the tail's `quantifier_confidence` is `Some("Low")`, and a System Message always trails the Narration.
2. **Seeded `with_prompt_responses`.** This is the form the movement/trigger/NPC assertions rely on (9.2, 9.5, 9.8, 10.2, 10.4, 10.5, 10.6, 10.7, 10.8, 11.8, 11.10, 14.2, 22.5, 1.4, 1.6, 1.14, 24.x). The seeded answers are per-call and hand-ordered, so the tests assert the *effect of a specific quantifier answer*, never a real one.

Defects these answers hide (all **inferred**, from the research 04/06/07 mechanism):

- A quantifier pass that runs when it should not (re-impersonate, event retry, Retrigger) is invisible whenever its seeded answer says "no movement" — which is exactly 22.1/22.2's shape. The three-way tests 10.2 and 13.2 avoid the trap by seeding a *tripwire* answer (`destination: "room_1"` / `"room2"`), and they are the two tests that can actually fail for the wrong-quantifier defect.
- A save that restores the pre-quantifier world instead of the post-quantifier world is invisible in every test whose quantifier answer happens to equal the pre-turn world (i.e. no movement) — 36.1, 36.2, 9.1, 22.1–22.4.
- The `"[System] NPC detection uncertain"` fallback row, which the absent-quantifier configuration always produces, is what makes a swipe switch impossible after any such turn (the last Message is the System notice). No test asserts that interaction.
- `QuantifierResult::default()` has `movement: MovementParseResult::default()` (all `None`) — **known** from `src/domain/model/quantifier.rs` — so "no movement" is the double's *default*, not a decision any test states.

### 2c. SCENARIO 22.1 specifically (`docs/specs/swipe_new.md`)

- **The tag is carried by** `test_retry_re_impersonate_generates_input_swipe_http` (`tests/http/swipe_new.rs`), **known**.
- The test builds its app with `app_with_narrator(narrator)`, which passes `AgentRegistry::default()` — an **empty registry, no Quantifier agent at all** (**known**; `tests/http/support/app_wiring.rs` + `ActionPipeline::with_backends`). This is *more* than the seed's inference ("its mock quantifier returns no movement"): there is no quantifier double to answer at all, so the tail runs against `QuantifierResult::default()`.
- The scenario's Gherkin line `And the player has not moved (the quantifier did not re-run)` has **no assertion behind it**: the test body contains no `current_room_id`, no `get_debug_state`, no room check of any kind. It asserts `fetch_generating_status == "idle"` (**display**), the Input count, the Swipe count, the active text and `impersonated()` (**state**).
- Verdict: **display-only for the "not moved" claim** — in fact *no assertion*, which is weaker than display-only. 22.2 has the identical shape.
- Therefore the seed's prediction is **tested and confirmed**: the scenario passes regardless of whether the quantifier re-ran. It would pass even if the redo moved the player to another room.

## 3. Harness gaps (no test reaches these)

1. **Arrival narration in the integration harness — the seed's claim is CONFIRMED, with one qualification.**
   - **Known:** no file under `tests/` or `src/test_support/` mentions `arrival`, `ArrivalTaskContext` or `spawn_arrival_task_if_needed` at all (`grep -rn "arrival" tests/ src/test_support/` → no matches; `grep -rn "ArrivalTaskContext\|spawn_arrival_task_if_needed" src/` → only `src/bootstrap/run.rs`, `src/bootstrap/init_game.rs` and `src/application/arrival_service_tests.rs`).
   - **Known:** `TestAppBuilder` / `build_app_graph_for_tests` never call `spawn_arrival_task_if_needed`; `tests/http/support/app_wiring.rs`'s `app_with_production_graph` calls `build_app_graph`, which also does not spawn arrival (only `bootstrap::run::prepare_state` does).
   - **Known:** the only arrival coverage is `src/application/arrival_service_tests.rs`, which calls `ArrivalTaskContext::run()` directly, single-threaded, with no slot, no concurrent turn and no game switch.
   - **Qualification:** the unit tests do assert arrival's own state outcome (`Idle`, the Narration row, the forensics row), so `Sa` is not wholly uncovered — what is uncovered is the *interleaving*.
   - No test covers: arrival concurrent with a turn (research 05 pair 1), arrival's save landing after the turn's, arrival's `Error` overwriting a turn's status, arrival on **every** boot of a scenario-less World, or a game switch during the arrival LLM call.
2. **`/status/reset-generating` has no state-asserting test.** The route is hit only by `tests/test_utils/wait.rs::wait_for_status_ready` (a timeout-recovery side effect, no assertion) and by `test_reset_generating_handler` in `tests/http/requires_migration/fragment.rs`, which asserts only that the response body contains `"reset"`. `reset_persisted_status` itself is covered at the unit tier (`test_reset_generating_status_sets_idle`). The "frees the slot under a live task, then a second claim succeeds" race (research 05 pair 5) has no test.
3. **`handle_cancellation`'s cross-game write** is asserted only incidentally, through the four cancellation tests, and each frames it as the *cancelled* generation's status. No test asserts that the **flipped-to** game is the one written, nor that a live turn in that game is disturbed.
4. **The `S0` and `S4` save sites**: the two tests named after them (`test_pre_main_snapshot_saved_before_narration`, `test_pre_event_snapshot_saved_before_continuation`) read only `load_latest_snapshot().db_id.is_some()`. **No test asserts a pre-main or pre-event Snapshot's content, its position in the save order, or that no pointer was set.**
5. **`check_retry_anchor`'s event-branch mismatch** (research 07 Open point 1): no test makes the anchor's pointer valid and the target's pointer missing, which is the shape where the guard passes and the redo then fails.
6. **The `/debug/state` and `/debug/is_generating` routes**: only `tests/http/requires_migration/debug.rs` (quarantine; asserts JSON **field presence and types**, never a value) and `tests/http/dashboard.rs` 39.3 (`/debug/is_generating`'s boolean vs the registry). No test reads a restored room or pointer through `/debug/state` as a *checked value* outside `swipe_new.rs`/`retrigger.rs`.
7. **Concurrent writers** (research 05 §3 pairs 1, 2, 4, 5, 6, 7, 8, 9): no test. Only pair 3 has tests.
8. **`delete_last` leaving Snapshot rows behind** (research 07 Open point 5): no test.
9. **`edit_history` / `delete_last` on a `System` Message** (research 07 §2): no test deletes or edits a trailing System notice, although that is where the rendered Delete control sits.
10. **A main Retry followed by Retrigger** (the stale `last_trigger`, research 06 D8): no test.

## 4. The seed's "would go red" tests

### 4a. "A swipe switch after a retry whose quantifier moves the player (cover swipe 0 and the retry swipe)"

**Does not exist.** Nearby tests, and what each leaves uncovered:

- `test_retry_reruns_quantifier_moves_player_http` (`tests/http/swipe_new.rs`, tag 9.2) — the quantifier **does** move on the retry, and the test asserts the room through `get_debug_state`. It performs **no swipe switch**.
- `test_retry_swipe_without_movement_reports_no_room_http` (tag 9.8) — a swipe switch **after** a retry, but the retry's quantifier answers "no movement", and the only post-switch assertions are `Message::location_header()` values read through `load_messages`. It never asserts `get_debug_state().current_room_id` after either switch. **This is the test with the "requirement shape" but the wrong quantifier answer and a display-only verdict.**
- `test_switch_swipe_leaves_the_game_idle_http` and `test_switch_swipe_drops_the_offered_option_set_http` (`tests/http/swipe_switch.rs`, tags 36.1, 36.2) — `options_app()`'s quantifier answers `{"npcs_in_room": []}` for **every** call, so neither the Action nor the retry ever moves; neither test reads the room.
- `test_switch_swipe_restores_a_settled_snapshot` (unit, `message_service_tests.rs`) — hand-seeds **both** Swipes with the **same** Snapshot and asserts status/phase/options only, so it cannot see a wrong-world restore at all.

Conclusion: the seed's predicted red test is absent. The nearest existing test would not fail for the seed §1 reason (its quantifier never moves; its assertions are display-only).

### 4b. "An event retry whose Trigger needs an NPC the main quantifier detected"

**Does not exist.** Nearby tests:

- `test_quantifier_npc_fires_trigger_http` (tag 1.4) and `test_trigger_continuation_reruns_quantifier_detects_new_npc_http` (tag 1.6) — a main quantifier detects an NPC and a Trigger fires, with `npcs_in_area` / `times_met` asserted (tags 1.4 asserts the event row; 1.6 asserts `npcs_in_area` and `times_met`). No retry.
- `test_event_retry_does_not_carry_the_previous_swipes_encounter_http` (tag 10.5) — an event that met an NPC, then an **event retry**, asserting `times_met == 0` and `npcs_in_area` empty. This is the closest, and it asserts the *opposite* direction: the retry must **not** inherit the main quantifier's NPC. It does not test a Trigger that *needs* the NPC the main quantifier detected.

Conclusion: absent. Note that 10.5's assertion and the seed's proposed test pull in opposite directions — 10.5 pins that the event redo starts from `S5`-derived, reconciliation-owned NPC state, not from the main quantifier's scene.

## 5. Spec coverage of the three defected behaviours

| Defect | Spec that states the correct behaviour | Scenario | Test tagged to it | Can that test fail for the defected reason? |
|---|---|---|---|---|
| **Seed §1** — a Swipe's Snapshot is set before the quantifier runs | `docs/diataxis/explanation/storage_design.md` §Per-swipe state binding (an explanation doc — no scenarios, so no tag); `docs/specs/swipe_switch.md` and `docs/specs/swipe_new.md` 9.8 are the HTTP-observable surface | `swipe_new.md` **9.8**; `swipe_switch.md` **36.1**, **36.2** | `test_retry_swipe_without_movement_reports_no_room_http`; `test_switch_swipe_leaves_the_game_idle_http`; `test_switch_swipe_drops_the_offered_option_set_http` | **No.** 9.8's redo does not move and its post-switch assertions are `location_header` (display). 36.1 asserts a registry-backed status. 36.2 asserts the dock, which is rebuilt from `current_options` — a switch that restored the wrong *world* would still clear the option set, so 36.2 cannot fail. **None of the three reads the restored room, NPC set or Snapshot id.** |
| **Seed §2** — re-impersonate re-runs the quantifier, contrary to I.6 / 22.1 | `docs/specs/swipe_new.md` **I.6** and **22.1** | 22.1, 22.2 | `test_retry_re_impersonate_generates_input_swipe_http`, `test_retry_user_regen_generates_input_swipe_http` | **No.** Both run with an empty `AgentRegistry`, so the reported failure (a quantifier re-run) has no observable footprint; the "has not moved" line is unasserted. The behaviour *is* pinned in the opposite direction at the unit tier by `test_retry_flow_impersonate_mode_runs_full_tail` (asserts `quantifier_confidence.is_some()`), which is **untagged** and therefore invisible to `validate_feature_spec.py`. So the spec and the code disagree, both are "covered", and only the untagged unit test can fail |
| **Location header** (research 08) | `docs/specs/swipe_new.md` **10.8** ("the event continuation must not carry the room the main Narration arrived in"), **9.8**, **1.14** | 10.8, 9.8, 1.14 | `test_event_retry_keeps_the_room_name_off_the_event_swipe_http`, `test_retry_swipe_without_movement_reports_no_room_http`, `test_moving_narration_reports_the_room_it_arrives_in_http`; the unit `test_event_retry_redo_swipe_names_no_room_of_its_own` | **Yes, for the header itself** — these are genuine display assertions on `Swipe::location_header` and they would fail if the header moved. They are **display-only** in the sense that no test checks whether the *Snapshot restored* matches the header the switch shows. `test_moving_narration_reports_the_room_it_arrives_in_http` (1.14) asserts the header but never `current_room_id`, so a header that reports the wrong room's *name* while the state is right (or vice versa) is caught only by the header assertion |

Additional spec/test mismatches found while building the matrix (facts only):

- **`docs/specs/swipe_new.md` 10.2's tripwire is real** — the seeded quantifier answer would move the player back to `room_1` if it were consulted, and `test_event_retry_does_not_rerun_quantifier_http` compares `room_before`/`room_after` through `get_debug_state`. This is the one HTTP test that can fail for a wrongly-run quantifier. Its twin, 13.2, does the same for Retrigger. By contrast 22.1/22.2 have no tripwire.
- **`docs/specs/swipe_switch.md` has exactly two scenarios (36.1, 36.2), both display-flavoured**, and no scenario states what world a switch restores. The seed's §1 table (main Retry gets `S0`, event retry gets `S3`) is contradicted by research 04's Correction and research 07's table; no spec scenario states either pointer, so nothing is tagged to a scenario about pointer targets.
- **`docs/specs/actions.md` 1.4** is tagged to `test_quantifier_npc_fires_trigger_http`, which asserts the event row and the status — but its sibling assertion in the same file, `test_trigger_continuation_reruns_quantifier_detects_new_npc_http` (tag 1.6), is the one that reads `npcs_in_area` and `times_met`.
- `tests/http/swipe_new.rs` SCENARIO **10.5**'s `test_event_retry_does_not_carry_the_previous_swipes_encounter_http` is the only place where the *event retry's own* reconciliation state is asserted; every other event-retry test seeds `seed_event_flow`'s hand-built pointers.

## 6. Report summary

- Matrix rows: **79** across §1a–§1d (16 generation states, 18 Snapshot states/pointers, 19 retry forms, 26 writer rows), plus 4 harness-fixture rows in §1e that are not coverage rows.
- Rows with at least one **state-asserting** test: **66 of 79** (1a 11/16, 1b 15/18, 1c 17/19, 1d 23/26). The 13 rows with no state-asserting test are: the `Generating` status written at claim and `GenerationPhase::Options` (no test at all); `S0`, `S4` and `S2b` (weak existence-only or run-completes assertions); the `Idle`-after-every-generation row and the error-endpoint row (display only); `Si`'s pointer and the `init_game::load_game_state` boot writer; the concurrent-writer row; and the Retrigger-guard-after-a-main-Retry row (absent).
- Harness gaps: arrival interleaving; `/status/reset-generating` at HTTP; `handle_cancellation`'s cross-game target; `S0`/`S4` content; the `check_retry_anchor` event-branch mismatch; `/debug/state` values; eight of the nine concurrent-writer pairs; `delete_last`'s orphan Snapshot rows; edit/delete of a System notice; a main Retry followed by Retrigger.
- Masking doubles: the **absent quantifier** (`AgentRegistry::default()` in `app_with_narrator`) and the **unseeded `MockBackend`** (`[Continuation: …]`, non-JSON → `QuantifierResult::default()`: no movement, no NPCs, Low confidence, plus a trailing `[System] NPC detection uncertain` row). Plus the hand-seeded pointer fixtures `seed_main_narration_flow`, `seed_event_flow`, `seed_swipe_with_stored_inputs`, `insert_message_with_swipe`.
- Unresolved in scope: none blocking. Two items are **undetermined** and a reader would need a targeted run to settle them: (a) whether any HTTP `fetch_generating_status == "idle"` assertion could ever observe the *persisted* status rather than the registry — read as no, from `generating_status_handler`, but not executed; (b) whether `should_fail()`'s effect on the quantifier branch (2a) is exercised anywhere by design or is incidental — no test comment states either way.
