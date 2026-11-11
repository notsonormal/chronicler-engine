# Migrate tier-1 tests to observe through legal read seams

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Apply the domain-outcome rule from [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md): a tier-1 test observes the response plus stored state through a read seam (an HTTP GET, or a method on an application read service/port). It may not dereference a field of `GameState` or read a storage row.

Migrate the existing violations now:

- **45 raw `GameState` field reads** in `tests/http/`, almost all `narrative.input_buffer.status`. Sites: `actions.rs` (13), `swipe_new.rs` (17), `retrigger.rs` (9), `games_create.rs` (1), plus `tests/http/support/http_requests.rs` (1). List: [raw_gamestate_reads.txt](../assets/test-audit/raw_gamestate_reads.txt). Status reads use `GET /status/generating`; where a test needs more, `GameViewQuery::get_generating_status` is an application read seam. Synchronization waits (`wait_idle`) may keep reading state.
- **Storage observation reads** in `tests/http/`: `get_settings` (1, `prompt_presets.rs`), `get_preset` (1, `prompt_presets.rs`), `get_game` (1, `games_delete.rs`), `load_message_rows` (1, `games_create.rs`), `list_latest_llm_messages` (4, `llm_messages.rs` ×2, `swipe_new.rs`, `narrator_mode.rs`). Replace with `GameViewQuery::list_latest_llm_messages()`, `MessageService::load_messages`, `PromptPresetService::get_preset`, or the matching `GET /fragment/...`.
- **Three persistence facts** have no read seam and reduce to storage-tier assertions: `load_latest_snapshot` and `count_swipes_for_message` (`games_create.rs`), `require_active_swipe_index` (`swipe_new.rs`). Move each to `tests/storage/` or drop the assertion as covered there.

`storage.current_game_id()` (14 sites) stays where it identifies a game for setup. The two assertions on it (`games_delete.rs`, `games_switch.rs`) read through a seam.

## Done when

- No `tests/http/` test dereferences a `GameState` field for an assertion (excluding synchronization waits).
- No `tests/http/` test reads storage for an observation.
- `python build.py` is green, including `spec-coverage`. The user reviews the diff. Commit after approval.

## Answer

All violations named by the audit list migrated, plus sites the list omitted but the Done-when covers.

- **Raw `GameState` reads.** Status reads now go through `GET /status/generating` (a new `fetch_generating_status` beside `wait_idle` in `tests/http/support/http_requests.rs`); room, scene, and encounter observations go through `GameViewQuery::get_debug_state`; message and event observations through `MessageService::load_messages`. `wait_idle` stays as the allowed synchronization wait. The 45 sites in the audit list are covered, along with the `options.rs` helper and the second `get_settings` / extra `get_preset` sites.
- **Storage observation reads.** `list_latest_llm_messages` → `GameViewQuery::list_latest_llm_messages`; `get_settings` → `SettingsService::get_settings`; `get_preset` → `PromptPresetService::get_preset`; `load_message_rows` → `MessageService::load_messages`; `get_game` → `GET /fragment/games` plus `GameCatalogue`; `current_game_id` assertions → `GameCatalogue::current_game_id()`; `require_active_swipe_index` → `Message.active_swipe_index` through `load_messages`.
- **The three persistence facts.** The scenario message's swipe is asserted in `tests/http/games_create.rs` through `load_messages` (`!scenario_msg.swipes.is_empty()`). The initial snapshot is asserted at the application tier by the new `message_service_tests::test_save_message_and_snapshot_persists_a_snapshot` — a snapshot row is not a tier-1 read surface. Skipping the `write_snapshot` call in `save_message_and_snapshot` fails that test, verified by temporary mutation.
- No production read seam was added; production code is unchanged.

Tier: tier 1 (HTTP E2E) for the migrated tests, unit for the snapshot assertion. The quarantined `tests/http/requires_migration/fragment.rs` keeps one `wait_idle`-style status poll, which the ticket's exception covers. `python build.py` is green, including `spec-coverage`.
