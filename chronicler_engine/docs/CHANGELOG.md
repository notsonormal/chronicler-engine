# Changelog

## 2026-05-28

### Changed
- **Unified `Storage` struct replaces 6 traits + 12 repository structs**
  - New `Storage` struct (`src/storage/backend/mod.rs`) with `Backend` enum (`Sqlite`, `InMemory`, `Test`)
  - All `Arc<dyn Trait>` injection points collapsed to `Arc<Storage>`
  - `GameServiceContext` reduced from 5 storage fields to single `storage: Arc<Storage>` + `preset_storage: Arc<Storage>`
  - `Backend::Test` supports dynamic failure injection via `Operation` enum + `TestOverride` + `TestFailureHandle`
  - Deleted: `game_storage.rs`, `snapshot_storage.rs`, `message_storage.rs`, `message_swipe_storage.rs`, `prompt_preset_storage.rs`, `llm_message_storage.rs`, `in_memory_storage.rs`, and all associated `*_storage_tests.rs`
  - ADR-019 guardrail (`guardrails_one_table_per_storage`) removed — no longer applicable with unified struct
  - ADR-019 marked as superseded; new ADR-020 documents the consolidation decision
  - All 859 tests pass; clippy clean; `build.py` clean
- **Split `backend.rs` into directory module by table domain**
  - Converted `src/storage/backend.rs` (1,248 lines) → `src/storage/backend/` directory module
  - `mod.rs` holds `Storage` struct, `Backend` enum, constructors, and shared helpers
  - Table-scoped methods moved to `games.rs`, `snapshots.rs`, `messages.rs`, `swipes.rs`, `presets.rs`, `llm_messages.rs`
  - Renamed `from_db` → `db_preset_to_preset` for naming consistency
  - Replaced `Mutex<Backend>` with `Mutex<Option<Backend>>` to eliminate dummy `InMemoryData` allocation in `add_failure`
  - Zero logic changes; all 894 tests pass; clippy clean; `build.py` clean
- **Removed obsolete DB migrations v1-v8 from `src/storage/db.rs`**
  - Replaced ~270 lines of incremental migration logic with a single idempotent `if version < 9` block
  - Fresh databases now get the final v9 schema directly via `CREATE TABLE IF NOT EXISTS`
  - Kept `run_migrations` function, `PRAGMA user_version` check, and migration pattern template for future schema changes
  - Deleted helper functions: `merr`, `recreate_prompt_presets_table`
  - Updated `docs/reference/data_layer.md` migration policy
  - All 894 tests pass; clippy clean; `build.py` clean

## 2026-05-27

### Changed
- **Storage tier split — one table per storage module (ADR-019)**
  - New `GameStorage` trait (`src/storage/game_storage.rs`) with `SqliteGameRepository` and `InMemoryGameRepository`
  - New `MessageSwipeStorage` trait (`src/storage/message_swipe_storage.rs`) with `SqliteMessageSwipeRepository` and `InMemoryMessageSwipeStorage`
  - `SnapshotStorage` stripped of game CRUD; now owns `game_state_snapshots` only
  - `MessageStorage` stripped of swipe methods; `insert_message` is metadata-only (swipes stored separately)
  - `GameServiceContext` gains `game_storage` and `message_swipe_storage`; adds cross-storage helpers (`load_messages`, `update_message_text`, `migrate_swipes`)
  - All callers migrated from monolithic `snapshot_storage` to `game_storage` for game CRUD
  - Schema v9: `ON DELETE CASCADE` on `game_state_snapshots` and `messages` referencing `games`; restored `DEFAULT 1` on `game_id`
  - `arch-lint` enforces no direct `storage` imports in `server` layer
  - Unit tests added: `game_storage_tests.rs`, `message_swipe_storage_tests.rs`

## 2026-05-26

### Changed
- **`PromptAssembler` replaces `PromptBuilder` — transport/assembly decoupling**
  - New `PromptAssembler` trait in `src/narrative/prompt/assembler.rs` with `assemble(context, preset, global_rules, response_length) -> Result<AssembledPrompt, EngineError>`
  - `LayeredPromptAssembler`: default implementation that renders 7 XML layers and applies token budgets directly from `PromptPreset` sections
  - `AssembledPrompt` struct holds `{ system_prompt, user_prompt, max_tokens }`
  - `DefaultGameService` owns `Arc<dyn PromptAssembler>`; constructed from connection settings in `with_storage()`
  - `ActionPipelineBackend` trait: `assembler() -> &dyn PromptAssembler` replaces `narrate_action(&PromptContext)`
  - `ActionPipeline::phase_narrate()`: loads preset → `make_prompt_context` → `assembler().assemble()` → `service.complete()`
  - `LlmBackend` trait slimmed to pure transport: `complete` and `narrate_continuation` only
  - Removed from `LlmBackend`: `narrate_action`, `narrate_arrival`, `generate_dialogue`
  - Deleted `narrate_from_context` from `OpenRouterBackend` and `OllamaBackend`
  - Deleted `builder.rs` and `builder_tests.rs`
  - Removed `system_prompt` field from `PromptContext`
  - Removed `POST_HISTORY_DELIMITER` and `assemble_split_text()` from `PromptPreset`
  - Removed `active_system_prompt()` from `GameServiceContext`
  - `make_prompt_context` no longer takes `system_prompt` parameter
  - Bootstrap arrival narration uses assembler + `complete()` instead of `narrate_arrival()`
  - `MockBackend` and `DeepSeekBackend` cleaned of narrative methods
  - Documentation updated: `system/prompt_system.md`, `architecture/system.md`, `reference/testing.md`, `reference/system_prompt.md`, `system/llm_processing.md`, `system/narration_engine.md`, `adr-005`

## 2026-05-25

### Changed
- **`PromptContext` requires assembled system prompt — `Option<String>` removed**
  - `PromptContext.system_prompt` and `PromptBuilder.system_prompt` are now `String` (was `assembled_system_prompt: Option<String>`)
  - `make_prompt_context()` takes `String` instead of `Option<String>`
  - `PromptBuilder::render_system_layer()` returns `self.system_prompt.clone()` — no silent empty-string fallback
  - `pipeline.rs`: `phase_narrate()` errors explicitly if `active_system_prompt()` returns `None`
  - `bootstrap/run.rs`: arrival narration assembles the active preset before building `PromptContext`
  - `openrouter.rs` + `ollama.rs`: derivative contexts (dialogue, arrival) propagate parent `system_prompt`
  - Test helpers updated: `make_test_context()` and `make_test_context_with_npc()` include a default test system prompt; `build_test_context()` and `make_test_context_with_sqlite()` seed `InMemoryPromptPresetStorage` with a default preset
  - Documentation updated: `reference/system_prompt.md`

## 2026-05-25

### Added
- **Sectioned XML-wrapped prompt presets — `prompt_text` split into four fields**
  - `PromptPreset` domain model: new `role`, `instructions`, `writing_style`, `output_format` fields (`Option<String>`)
  - `PromptPreset::assemble_prompt_text()` assembles sections into XML-wrapped tags with fixed order: `<role>` → `<instructions>` → `<writing_style>` → `<global_rules>` → `<output_format>`
  - Global rules from `world.json` injected dynamically before `<output_format>`
  - Response length from `AppSettings` appended inside `<output_format>` content
  - DB migrations v7 (add section columns) and v8 (drop `prompt_text` column)
  - `DbPromptPreset` and `SqlitePromptPresetStorage` updated for new schema
  - Seed files restructured: `data/prompt_presets/system/default.json` and `quantifier/default.json`
  - UI updated with four textarea fields per preset (Role, Instructions, Writing Style, Output Format)
  - Startup/activation caching calls `assemble_prompt_text()` with `world.global_rules` and `response_length`
  - Documentation updated: `adr-004` (v4 section), `adr-005` (sectioned refactor), `adr-015` (preset structure table), `reference/system_prompt.md`, `reference/quantifier_prompt.md`, `system/prompt_system.md`, `system/game_flow.md`, `system/llm_processing.md`

### Added
- **`TestAppBuilder` — test infrastructure without `GameState` exposure**
  - New `src/test_support/test_app_builder.rs` with fluent builder API
  - `default_test()` mirrors old `tests/components.rs::create_test_state()` defaults
  - `new(world, player)` for fully custom setups; `map()`, `npcs()`, `room_npc()`, `log()`, `generation_status()`, `settings()`, `snapshot_storage()`, `message_storage()`, `llm_storage()`
  - Internally constructs `GameState` and `AppState`; callers never touch `GameState` directly
  - Replaces `create_app_for_testing`, `create_app_for_testing_with_settings`, `create_app_with_storage` (all removed from `src/test_support/server_helpers.rs`)

- **Layer boundary guardrails** (`tests/guardrails/layers.rs`)
  - `guardrails_server_layer_boundaries`: bans `GameState` references and `.load_state()` calls in `src/server/` (except `mod.rs` and `debug.rs`)
  - `guardrails_test_layer_boundaries`: bans `GameState::new()` construction and `GameState` imports in `tests/components/`

### Changed
- **Eliminated `GameState` from all integration test signatures**
  - Migrated ~114 call sites across 10 `tests/components/*.rs` files to `TestAppBuilder`
  - Moved 4 `npcs_in_area` unit tests from `tests/components/world.rs` to `src/model/state_tests.rs`
  - Removed `create_test_state()` from `tests/components.rs`
  - `src/server/` now has zero `GameState` references
  - `tests/components/` now has zero `GameState` imports/construction
- **Updated agent rules** (`.agents/rules/chronicler_engine.md`) and **architecture docs** (`docs/architecture/system.md`) with layer boundary guidance

## 2026-05-24

### Added
- **ApplicationService — logic firewall between HTTP handlers and domain**
  - New `ApplicationService` trait in `src/application/application_service.rs` with `DefaultApplicationService` implementation
  - Orchestrates all state mutations, persistence, and game-service calls
  - Methods: `process_action`, `retry`, `retrigger`, `switch_swipe`, `edit_history`, `delete_last`, `reset`, `create_game`, `switch_game`, `delete_game`, `list_games`, `get_generating_status`, `reset_generating_status`, `load_state`, `get_current_game_name`, `list_latest_llm_messages`
  - All Axum handlers now reduced to request parsing + `ApplicationService` delegation + HTTP response mapping
  - No handler directly touches `snapshot_storage.save()`, `message_storage.load_messages()`, or `GameStateSnapshot::from_game_state()`

- **View model layer — decoupled templates from domain types**
  - New `src/server/view_models.rs` extracting `SafeHtml`, `LogEntryView`, `PreviewIssueView`, `LlmMessageView` from `templates.rs`
  - New `ActionAreaViewModel` decouples `ActionAreaTemplate` from `GenerationStatus`/`GenerationPhase`
  - New `VisualSidebarViewModel` and `NpcPortraitView` replace raw `(String, String)` tuples in `VisualSidebarTemplate` and `CharacterHeadshotsTemplate`
  - `templates.rs` now focuses purely on HTML; all domain-to-view mapping lives in `view_models.rs`

### Changed
- **Storage repository split — eliminated swipe blast radius**
  - Split `SqliteGameStorage` into `SqliteSnapshotRepository` + `SqliteMessageRepository`
  - Split `InMemoryGameStorage` into `InMemorySnapshotRepository` + `InMemoryMessageRepository`
  - `MessageStorage::insert_message` signature changed from `(&self, &mut Message) -> Result<(), _>` to `(&self, &Message) -> Result<u64, _>` — returns generated ID instead of mutating input
  - Removed `reset()` from concrete storage types (test-only convenience; tests use fresh repos per test)
  - All call sites updated to capture returned ID explicitly
  - Snapshot changes no longer force message code recompilation, and vice versa

- **Arch-lint guardrail**: Added `deny-scope-dep` rule banning `server -> storage` imports; server layer must access storage through `ApplicationService`
- **Removed `AppState::load_state()`**: All loading goes through `ApplicationService::load_state()`
- **Removed dead UI fields from `InputBuffer`**: Deleted `cursor_position`, `scroll_offset`, and methods `push_char`, `pop_char`, `clear_input`

## 2026-05-24

### Added
- **Message swipes — non-destructive retry with state-consistent swipe navigation**
  - New `Swipe` struct: each swipe stores `text`, `snapshot_id: Option<u64>`, `location_header`, and `event_header`
  - `Message` now has `swipes: Vec<Swipe>`, `active_swipe_index: usize`, and `is_deleted: bool`
  - `LogEntry` carries `swipe_count` and `active_swipe_index` for template rendering
  - New `message_swipes` SQLite table with `ON DELETE CASCADE`; v6 migration recreates `messages` table and migrates existing data
  - `MessageStorage` trait expanded: `soft_delete_message`, `restore_soft_deleted`, `purge_soft_deleted`, `insert_swipe`, `update_active_swipe`, `shift_swipe_indices`
  - Retry changed from destructive delete to **soft-delete + swipe preservation**: old text is saved as a swipe before regeneration; on failure, soft-deleted messages are restored; on success, they are purged
  - `post_retry_swipe_migration` shifts old swipes to the new message and sets active index to the latest swipe
  - **Swipe navigation endpoint**: `POST /message/:id/swipe/:index` updates active swipe, restores swipe's snapshot, re-renders story log. Only allowed on the last message (returns 400 otherwise)
  - **Retrigger endpoint**: `POST /retrigger` runs `pipeline.run_trigger_continuation` from the current state when `last_trigger` exists and the last message is a narration (not an event continuation)
  - **UI**: Right arrow on latest swipe triggers new generation (`submitNewSwipe()` → `POST /swipe/new`); left/right arrows navigate between swipes; counter shows `active+1 / swipe_count`; "Retrigger Event" button (♻) appears on narration swipes when trigger context is available
  - Removed separate retry button from template — swipe right arrow replaces it
  - `snapshot_id` made properly nullable (`Option<u64>` in model, `INTEGER` nullable in DB) instead of sentinel `0`
  - New ADR-017 documenting the swipe architecture; ADR-013 updated to show it was superseded
  - All 871 tests pass; clippy clean

## 2026-05-21

### Changed
- **Multi-game support — replaced checkpoints with independent games**
  - Migration v5: added `name TEXT` to `games` table, dropped `checkpoints` table and index
  - New `Game` domain model with auto-generated names (`{WorldName}_{Date}_N`)
  - `SnapshotStorage` and `MessageStorage` traits gained `set_game_id` / `current_game_id` for runtime switching
  - `SqliteSnapshotRepository`/`SqliteMessageRepository` and `InMemorySnapshotRepository`/`InMemoryMessageRepository` filter all queries by active `game_id`
  - Game CRUD: `list_games`, `create_game`, `delete_game`, `get_game`
  - Bootstrap startup: auto-creates game if none exist for world; loads most recent if games exist
  - Server endpoints: `GET /fragment/games`, `POST /games`, `POST /games/:id/switch`, `POST /games/:id/delete`
  - UI: header shows active game name + "Games" dropdown with switch/create/delete
  - **Reset reworked**: deletes current game + creates new one with fresh auto-generated name (full page refresh)
  - **Checkpoints removed entirely**: all model, mapper, storage, server, test code deleted
  - All 877 tests pass; clippy clean

### Fixed
- **Latest-game heuristic**: `find_latest_game_for_world` now orders by most recent message timestamp (with `updated_at` fallback) instead of `updated_at` alone. This ensures the actually-played game is loaded on restart.
- **`delete_game` transaction**: `SqliteSnapshotRepository::delete_game` now wraps its three cascading `DELETE`s in a SQLite transaction so partial failures cannot leave orphaned data.
- **Game naming**: `generate_game_name` simplified from gap-filling loop to `max(N) + 1` algorithm.
- **Code hygiene**: Extracted `game_id()` helper to replace ~25 inline `AtomicU64::load` calls across storage implementations.

## 2026-05-20

### Changed
- **Immediate message persistence — removed `committed` flag and batch persistence**
  - Each message is now persisted immediately alongside a snapshot via `save_message_and_snapshot`
  - Deleted `committed: bool` from `GameStateSnapshot` and `committed` column from DB
  - Removed `SnapshotStorage::commit()`, `save_committed_state()`, and `persist_new_messages()`
  - `save_message_and_snapshot` saves snapshot then persists the last message if `id == 0`
  - Pipeline updated: messages persisted at pre-main, post-generation, post-engine, and post-event phases
  - Event retry fixed: pre-build trigger request before post-engine snapshot so `last_trigger` is included in the restore point
  - All 868 tests pass; clippy clean

### Fixed
- **Reduced UI delay between LLM response and game story-log display**
  - Story-log HTMX poll interval reduced from `every 2s` to `every 1s`
  - Status-display HTMX poll interval reduced from `every 5s` to `every 2s`
  - Added immediate story-log refresh via `htmx.trigger('#story-log', 'htmx:refresh')` when status poll detects `idle`
  - This cuts the worst-case gap between pipeline completion and story-log update from ~4s to ~1s
  - All 870 tests pass; clippy clean

## 2026-05-19

### Fixed
- **Prompt preset review fixes — schema defaults, HTMX nesting, XSS, storage API**
  - Fixed `data/schemas/settings.schema.json` defaults: `"system_default"` / `"quantifier_default"` now match Rust code
  - Fixed HTMX nesting bug in prompt presets panel: changed `hx-swap="innerHTML"` to `hx-swap="outerHTML"` on `.prompt-presets-panel` targets
  - Fixed XSS vulnerability in `PromptPresetsPanelTemplate`: `{{ preset.prompt_text }}` → `{{ preset.prompt_text|escape }}`
  - Moved `html_escape()` helper from `prompt_presets_fragment/template.rs` to shared `server/fragments/renderers.rs`
  - Added single-quote escape (`'` → `&#x27;`) to `html_escape()`
  - Deleted unused `run_server()` from `src/server/mod.rs` — production path is `bootstrap::run` → `run_server_with_config`
  - All 862 tests pass; clippy clean

### Changed
- **Prompt preset domain model refinement — `preset_type` ownership and safer parsing**
  - Added `preset_type: PresetType` to `PromptPreset` domain model (already stored in DB; now carried through domain layer)
  - Simplified `PromptPresetStorage::save()` signature from `save(&preset, type)` to `save(&preset)` — type read from `preset.preset_type`
  - Replaced dangerous `impl From<&str> for PresetType` (silent fallback to `System`) with `impl TryFrom<&str>` returning explicit `Err` for unknown strings
  - Fixed stale cache bug: `update_preset_handler` now updates `AppSettings.active_system_prompt` / `active_quantifier_prompt` when an active preset is edited
  - `InMemoryPromptPresetStorage` simplified from `Vec<(Preset, Type)>` to `Vec<Preset>`
  - All 862 tests pass; clippy clean

## 2026-05-19

### Changed
- **Removed hardcoded system prompt and quantifier templates; renamed PHI → OutputFormat**
  - Deleted `src/narrative/prompt/templates.rs` — system prompt now comes exclusively from DB-loaded preset (`active_system_prompt`)
  - Deleted hardcoded fallback from `src/narrative/agents/quantifier/prompt.rs` — quantifier prompt now comes exclusively from DB-loaded preset (`active_quantifier_prompt`)
  - Added startup preset loading in `src/bootstrap/run.rs` and `src/server/mod.rs` to cache active preset text into settings on boot
  - Moved writing style + "Your only job" instructions from system prompt seed into inline `OUTPUT_FORMAT_TEMPLATE` in `builder.rs`
  - Renamed `render_phi_layer()` → `render_output_format_layer()` throughout builder and tests
  - Updated `docs/reference/system_prompt.md`, `docs/system/llm_processing.md`, `docs/system/prompt_system.md` to replace PHI terminology with Output Format
  - Updated `data/prompt_presets/system/default.json` — removed writing style section
  - All 821 tests pass; clippy clean

### Changed
- **Added invariant contract tests and consolidated testing documentation**
  - New `tests/invariant_contract_tests.rs` with 5 fast regression tests for documented runtime invariants (INV-001, INV-002, INV-004, INV-004b)
  - Simplified `docs/architecture/invariants.md` from 64 lines to 33 lines — stripped implementation prose, linked every testable invariant to its test function
  - Merged `docs/system/testing.md` into `docs/reference/testing.md` — single source of truth for testing docs
  - Removed stale references to unit-test sibling files (`logic_tests.rs`, `text_check_tests.rs`, `state_snapshot_tests.rs`) from integration test tables
  - Added fast iteration section with measured command durations
  - Updated `docs/README.md` and `docs/CHANGELOG.md` links to point to `reference/testing.md`
  - Archived superseded `fast-fail-build-test-localization-plan.md` to `docs/plans/archived/`
  - All 811 tests pass; clippy clean; coverage unchanged

### Changed
- **Split `tests/action_pipeline.rs` into submodules** — Mirrored the source module structure (`actions.rs`, `pipeline.rs`, `retry.rs`)
  - `tests/action_pipeline/` directory with `actions.rs` (8 basic execution tests), `pipeline.rs` (12 advanced behaviour tests), `retry.rs` (10 retry tests)
  - `tests/action_pipeline.rs` is now a module aggregator with shared helpers (`working_backend`, `failing_backend`, `pipeline_helpers`, `test_data`)
  - No test names or assertions changed — pure file reorganisation
  - All 806 tests pass; clippy clean

### Changed
- **Diagnostic signal quality improvements** — Fixed and updated the diagnostic benchmark suite
  - `map_llm_error` already preserved structured error details (HTTP status, network URL/detail, parse format, empty response) — updated benchmark notes and scores to reflect reality
  - Fixed `LowConfidenceQuantifierBackend` in diagnostic tests to return ambiguous non-JSON text, correctly triggering the low-confidence system log path
  - Updated all 12 benchmark scenarios with factually accurate notes; raised `state_visibility` scores for LLM scenarios (debug endpoint now exposes `last_error`)
  - Added `quantifier_confidence` field to `SceneState` and `DebugStateResponse` — debug endpoint now shows High/Medium/Low
  - Added `backend_name` and `model_name` fields to `NarrativeState` and `DebugStateResponse` — tracks which backend served the last narration
  - Fixed cancellation state-reset bugs in `run_trigger_continuation`: trigger retry failures and empty trigger responses now persist `GenerationStatus::Error` instead of leaving state stuck in `Generating`
  - All 806 tests pass; clippy clean

### Changed
- **Consolidated action pipeline integration tests and trimmed redundant `game_service` tests**
  - `tests/action_pipeline.rs` expanded from 9 to 30 tests — now owns all pipeline-behavior integration tests (narration, errors, cancellation, quantifier, trigger, retry)
  - Moved 17 tests from `tests/game_service/advanced.rs` to `tests/action_pipeline.rs`, adapted to call `execute_action_impl` / `retry_last_response_impl` directly
  - Added 4 new pipeline tests: phase transitions, phase-on-error, empty input, quantifier integration
  - `tests/game_service.rs` collapsed from 36 tests across `basic.rs` + `advanced.rs` to 5 focused service-boundary tests (constructors, trait delegation, edge inputs)
  - Deleted `tests/game_service/basic.rs` and `tests/game_service/advanced.rs`; `tests/game_service/` directory removed
  - Renamed `tests/helpers/game_service.rs` → `tests/helpers/pipeline_helpers.rs` — shared across `game_service`, `action_pipeline`, and `flow_mock` test crates
  - Updated `docs/reference/testing.md` to reflect new test organization
  - All 806 tests pass; clippy clean; coverage 83.8%

## 2026-05-18

### Changed
- **Extracted `ActionPipelineBackend` trait + reorganized `application` modules** — Turned the concrete `DefaultGameService` dependency into a real seam and separated concerns across the application tier
  - New `src/application/action_pipeline/` module — owns `ActionPipelineBackend` trait, `ActionPipeline<B>`, `actions.rs`, `retry.rs`, and `retry_tests.rs`
  - New `src/application/context.rs` — `GameServiceContext` + persistence helpers (moved from `game_service/context.rs` and `game_service/helpers.rs`)
  - `game_service/` simplified to service boundary only — `GameService` trait, `DefaultGameService`, and `impl ActionPipelineBackend for DefaultGameService` glue
  - `ActionPipeline` is now generic over `ActionPipelineBackend` instead of taking `&DefaultGameService` concretely
  - `DefaultGameService` implements the 3-method trait (`narrate_action`, `complete`, `run_post_generation_agents`), acting as an adapter
  - Tests can now inject narrow mocks implementing only `ActionPipelineBackend` instead of constructing full `DefaultGameService` instances
  - Zero breaking changes — all existing constructors and `GameService` re-exports remain unchanged
  - All 794 tests pass; clippy clean; architecture guardrails pass; test structure guardrail pass

## 2026-05-18

### Changed
- **Extracted `MessageHistory` from `GameState`** — Encapsulated `Vec<Message>` and all message operations into dedicated `MessageHistory` type
  - New `src/model/message_history.rs` — `MessageHistory` struct owns message lifecycle (append, edit, delete, query, capacity cap at 1000)
  - `NarrativeState.messages: Vec<Message>` replaced with `NarrativeState.history: MessageHistory`
  - `GameState` delegator methods preserved (`add_log`, `edit_log`, `delete_last_log`, etc.) to minimize call-site churn
  - `#[serde(transparent)]` ensures serialization shape unchanged; `NarrativeSnapshot` already excluded messages
  - 20 dedicated unit tests in `message_history_tests.rs`; 100% line coverage
  - All 799 tests pass; clippy clean; architecture guardrails pass

### Changed
- **Extracted `ActionPipeline` module** — Unified action and retry flows into a single pipeline with explicit phase methods
  - New `src/application/game_service/action_pipeline.rs` — `ActionPipeline` struct with `run_from_input` and `run_trigger_continuation`
  - `ActionOutcome` enum captures terminal states: `Completed`, `Error`, `Cancelled`
  - Private phase methods match documented game flow: pre-main snapshot → narrate → post-gen → engine → trigger → continuation → reconcile → finalize
  - `actions.rs` reduced from 417 lines to ~35 lines (thin dispatch layer)
  - `retry.rs` simplified — `retry_event_continuation` delegates to `ActionPipeline::run_trigger_continuation`, eliminating duplicated LLM call + commit + reconcile logic
  - `finish_action` moved to `helpers.rs` (shared by pipeline finalize and retry cancellation)
  - Preserved exact behavior: 3 cancellation checkpoints, dual error handling (early vs late), pre-main and pre-event snapshot timing

### Removed
- **Talk action** — Removed legacy `Action::Talk` variant (should have been removed with Look/Inventory/Quit)
  - `Action` enum now has only `FreeAction(String)`
  - Parser simplified — all input becomes `FreeAction`, no quote-parsing logic needed
  - Removed Talk handler from `actions.rs` and Talk tests from parser tests and integration tests
  - All 779 tests pass; clippy clean; architecture guardrails pass

## 2026-05-18

### Changed
- **Pipeline cancellation checkpoints** — `execute_freeaction_pipeline` now checks `ctx.cancel_token.is_cancelled()` at three stage boundaries
  - After main narration LLM call (prevents wasted quantifier + trigger work)
  - Before trigger continuation LLM call (prevents second LLM call on stale request)
  - After trigger continuation LLM call (prevents committing partial trigger state)
  - New `handle_pipeline_cancellation()` helper resets `GenerationStatus::Idle`, clears phase, and persists state
  - `retry_event_continuation` now also checks cancellation before running the trigger retry LLM call
  - Updated `docs/adr/adr-010-concurrency-generation-gate.md`, `docs/architecture/invariants.md`, `docs/architecture/system.md`, `docs/system/game_flow.md`, `docs/system/llm_processing.md`
  - All 781 tests pass; clippy clean

## 2026-05-17

### Changed
- **Unified lock-poison recovery strategy** — All `Mutex` and `RwLock` sites now recover consistently
  - `try_lock!` macro in `settings_fragment/handlers.rs` no longer returns HTML error on poison; recovers via `into_inner()` + logs warning
  - `AppState::settings()` no longer returns `AppSettings::default()` on poison; recovers actual settings + logs warning
  - `AppState::current_cancel_token()` and `replace_cancel_token()` now log warnings when recovering from poison
  - `openrouter.rs` and `ollama.rs` `response_length()` no longer silently returns default on poison; recovers + logs warning
  - Added `test_settings_recover_from_poisoned_rwlock` and `test_cancel_token_recover_from_poisoned_rwlock` to verify recovery
  - Updated `docs/architecture/invariants.md#INV-005` to document the unified strategy
  - All 781 tests pass; clippy clean

## 2026-05-17

### Added
- **GameStateBuilder** — Structural extensibility for `GameState`
  - Added `GameStateBuilder` in `src/model/state.rs` with required constructor fields (`world`, `map`, `player`, `starting_room`) and optional setters (`with_npcs`, `with_narrative`, `with_scene`, `with_npc_encounter_log`)
  - New fields added to `GameState` get `Default::default()` fallback in `build()`, so existing call sites do not break
  - `GameState::new` now delegates to `GameStateBuilder`
  - Marked `GameState` with `#[non_exhaustive]` to prevent integration tests from constructing with struct literals
  - Refactored all 7 manual `GameState { ... }` struct literal constructions across tests to use `GameStateBuilder` or `GameState::new`
  - Added `#[derive(Default)]` to `SceneState` (required for builder fallback)
  - All 779 tests pass; clippy clean; 84.4% coverage

## 2026-05-17

### Changed
- **Domain vocabulary rename** — Eliminated cognitive tax from three misnamed structures
  - `CharacterState` → `NpcEncounterLog` (encounter log map, not a single character's status)
  - `TriggerAction` → `TriggerEffect` (narrative effect payload, not an executable action)
  - `GenerationState` → `InputBuffer` (UI typing buffer, not LLM inference tracking)
  - Corresponding field renames: `GameState.character_state` → `npc_encounter_log`, `Trigger.action` → `effect`, `NarrativeState.generation` → `input_buffer`
  - Backward compatibility: `#[serde(rename)]` preserves old JSON keys for world data and DB snapshots
  - DB schema updated: `character_state` column → `npc_encounter_log`
  - All 779 tests pass; clippy clean; zero logic changes

## 2026-05-17

### Changed
- **Broke engine↔narrative bidirectional coupling** — Full decoupling of peer tiers
  - Moved `get_current_room` from `engine::logic` to `GameState::current_room()` in `model::state`
  - Added `current_room: Option<&Room>` to `AgentContext`; application layer resolves the room and passes it to agents
  - `determine_npcs_in_room` now accepts `&Room` instead of calling engine logic
  - Created `model::quantifier` module; moved `QuantifierResult`, `QuantifierParseResult`, `MovementParseResult`, `QuantifierConfidence`, `NpcEvent`, `NpcEventType`, `NpcEventList`, and `compute_npc_events` from `narrative::agents::quantifier`
  - Extracted trigger prompt building from `engine/action_processing.rs` to `application/game_service/actions.rs`
  - Simplified `FreeActionContext` to `narration_text` + `quantifier_result` only
  - `TurnResult` now carries `trigger_match: Option<TriggerMatch>` (raw engine data) instead of `trigger_continuation: Option<TriggerContinuationRequest>` (pre-built prompts)
  - Preserved mutation order invariant: trigger evaluation still happens before `apply_npc_events`
  - Added `engine → narrative` denial rule to `arch-lint.toml`
  - Removed dead code `evaluate_and_narrate_triggers` from engine
  - All 779 tests pass; clippy clean; architecture guardrails pass

## 2026-05-17

### Removed
- **Legacy synchronous trigger dead code** — Eliminated `evaluate_and_narrate_triggers()` from `engine/action_processing.rs`
  - This function performed a blocking LLM call inside the state lock; production has used the split architecture since the Application Tier extraction
  - Current pipeline: `execute_freeaction_impl` builds `TriggerContinuationRequest` (no LLM) → LLM call runs async outside the lock → `commit_trigger_narration` applies the result
  - Rewrote `test_evaluate_and_narrate_triggers_adds_event_header` to exercise the production split path (`execute_freeaction_impl` + `commit_trigger_narration`)
  - Updated `docs/architecture/system.md` to remove the obsolete function reference
  - All 782 tests pass; zero logic changes

## 2026-05-17

### Changed
- **Separated DB Models from Domain Models** — Storage layer now has clean internal architecture
  - New `src/storage/models/` — DB row structs (`DbGame`, `DbGameStateSnapshot`, `DbCheckpoint`, `DbMessage`, `DbLlmMessage`)
  - New `src/storage/mappers/` — Conversion logic between DB models and domain models
  - Domain models (`Message`, `Checkpoint`, `LlmMessage`, `GameStateSnapshot`, `NarrativeSnapshot`) moved from `src/model/storage/` to `src/model/`
  - Storage implementations (`SqliteGameStorage`, `SqliteLlmMessageStorage`) use DB models internally, map at the boundary
  - Arch-lint guardrails prevent any non-storage code from importing `storage-models`
  - All 772 tests pass; zero logic changes

## 2026-05-17

### Changed
- **Settings I/O Centralization** — Eliminated scattered `load_settings()` calls across all layers
  - Settings loaded **once** at startup in `bootstrap/run.rs` and passed down via `Arc<RwLock<AppSettings>>`
  - `GameServiceContext` now carries `settings: Arc<RwLock<AppSettings>>`
  - `DefaultGameService::with_storage(storage, settings)` receives settings from caller
  - `AgentRegistry::from_configs_with_storage(configs, storage, settings)` resolves connections without file I/O
  - `QuantifierAgent::from_config_with_storage(config, storage, settings)` no longer loads settings
  - `OpenRouterBackend` and `OllamaBackend` store `Option<Arc<RwLock<AppSettings>>>`; `response_length` read dynamically per-call
  - `build_trigger_prompt_parts` is now pure — takes `response_length`, `max_context_tokens`, `max_tokens` as explicit parameters
  - `get_llm_backend()` removed; all callers use `get_llm_backend_for(connection, storage, settings)`
  - Restart still required for connection changes; only `response_length` and `max_context_tokens` are dynamic at runtime
  - Zero logic changes; all 762 tests pass

### Changed
- **Extracted Application Tier** — Moved `game_service` from `engine/` to new top-level `application/` tier
  - `src/engine/game_service/` → `src/application/game_service/`
  - Clean architecture boundary: `server` → `application` → `engine` → `model`
  - `engine/` now contains only pure domain rules (parser, logic, trigger_eval, action_processing)
  - `application/` contains orchestration (DB I/O, LLM coordination, retry logic)
  - All imports updated across ~25 files; zero logic changes

### Changed
- **Architecture guardrails** — Updated `arch-lint.toml` to enforce layer separation
  - `engine` cannot depend on `application` or `server`
  - `application` cannot depend on `server`
  - `model` cannot depend on `application`

### Changed
- **Documentation** — Updated `docs/architecture/system.md` and `docs/architecture/guardrails.md` to document the new Application Tier

## 2026-05-16

### Changed
- **Unified quantifier backend with `LlmBackend` trait**
  - Deleted `QuantifierBackendTrait`, `RealQuantifierBackend`, `MockQuantifierBackend`, `OllamaQuantifierBackend`
  - `QuantifierAgent` now holds `Arc<dyn LlmBackend>` and calls `backend.complete()`
  - Extracted `wrap_and_save` as `LlmBackend` trait default method; added `model()` to trait
  - Renamed `narrate_action_from_prompt` → `complete` (generic prompt-completion primitive)
  - `MockBackend` gained `per_call_prompt_responses` for quantifier test scenarios
  - All documentation updated to reflect unified backend architecture

### Changed
- **Message-Aligned Snapshots** — Replaced `base_snapshot_id` chain with message-aligned snapshot model
  - Removed `base_snapshot_id` from `GameStateSnapshot` — snapshots are standalone state blobs
  - Replaced `turn_id` with `snapshot_id` on `Message` — references the snapshot saved **after** the message was created
  - Retry: find anchor message → load its `snapshot_id` → delete messages after anchor → apply snapshot → regenerate
  - Main retry anchor: last `Input` message; Event retry anchor: last non-event message
  - `save_state` and `save_committed_state` no longer take `base_snapshot_id`; they tag newly persisted messages with the saved snapshot's ID
  - All snapshot `turn_id`/`swipe_index` keying removed; snapshots use auto-increment `id` only

### Added
- **`games` table** — Top-level game session record scoping all state and messages
  - `games(id, world_name, created_at, updated_at)`
  - Default game row (`id=1`) auto-inserted on migration
  - Foundation for future multi-game support

### Changed
- **Game-scoped storage** — `game_id` added to `game_state_snapshots` and `messages`
  - `SqliteGameStorage::new(pool, game_id)` filters all queries by game
  - `InMemoryGameStorage` tracks `game_id` for test parity
  - `reset()` clears only the current game's data (was global)
  - New indexes: `idx_snapshots_game_latest`, `idx_messages_game_id`

### Added
- **Data layer reference doc** — `docs/reference/data_layer.md` documents all SQLite tables, columns, relationships, and migration policy

### Removed
- **Sync actions (`Look`, `Inventory`, `Quit`)** — All player input is now treated as `FreeAction` and routed through the LLM generation pipeline
  - `Action::Look`, `Action::Inventory`, `Action::Quit` variants removed from `Action` enum
  - Parser no longer recognizes `look`, `inventory`, `quit`, `q`, `exit` as special commands
  - `process_sync_action` and `is_sync` checks removed from `server/fragments/actions.rs`
  - `get_available_exits()` and `process_directional_movement()` removed from `engine/logic.rs` (dead code; movement is quantifier-driven)
  - Action hints UI (`render_action_hints`) now returns empty string — no bottom-left options displayed
  - `ActionAreaTemplate::available_actions` field emptied (was already unused in template)

### Removed
- **Inventory from LLM prompts** — `--- Inventory ---` block removed from `PromptBuilder::render_game_state_layer()`
  - System prompt no longer references "inventory" in state validation rules
  - `reference/system_prompt.md`, `system/prompt_system.md`, `system/llm_processing.md` updated to reflect prompt changes

### Changed
- **Server action handling simplified** — All actions uniformly go through `tokio::task::spawn_blocking`
  - Generation gate (`is_generating` AtomicBool) now applies to all actions
  - `HX-Trigger` header and sync response path removed from `action_confirm_handler`
  - `INV-006` invariant updated: "All Actions Are Async" (no sync paths exist)

### Updated
- **Documentation synced** — `architecture/invariants.md`, `system/game_flow.md`, `system/narration_engine.md`, `adr/adr-010-concurrency-generation-gate.md` updated to remove sync action references
- **Tests updated** — Parser tests, logic tests, browser tests, component tests, flow_mock tests, game_service tests updated for new async-only behavior

## 2026-05-15

### Changed
- **Message Domain Model** — Migrated from `Turn` + `Swipe` to flat `Vec<Message>`
  - New `Message` struct: `id: u64`, `turn_id: String`, `sender`, `text`, `log_type`
  - `NarrativeState.messages: Vec<Message>` replaces `Vec<Turn>`; `current_turn_id` tracks the active turn
  - `add_log()` creates a new `Message` for every call (input, narration, event, dialogue, system)
  - `history()` returns derived `Vec<LogEntry>`; all rendering/prompts unchanged
  - `delete_last_log()` pops the last message (peeling back layers)

### Changed
- **Retry is snapshot-based rollback** — Retry loads a pre-generation snapshot and re-runs the pipeline
  - Main retry: loads `pre-main:{turn_id}` snapshot, re-runs full pipeline
  - Event retry: loads `pre-event:{turn_id}` snapshot, regenerates continuation
  - `swipe_index` on the snapshot tracks retry count; no per-message swipes

### Fixed
- **`delete_last_log` recalculates `current_turn_id`** — Deleting the last input message now correctly resets `current_turn_id` in the model (was only handled in the HTTP handler)
- **Trigger index bug** — `evaluate_triggers` now returns the original trigger index, preventing `mark_trigger_fired` from marking the wrong trigger when some triggers are skipped
- **Pipeline TOCTOU** — `execute_freeaction_pipeline` no longer reloads state from storage after the LLM call; uses the passed state directly

### Removed
- **`MessageSwipe` and per-message swipes** — `swipes`, `active_swipe_index`, `create_swipe`, `switch_swipe` removed from `Message`
- **Swipe navigation UI** — `POST /turn/:id/swipe/:index` endpoint and action-area swipe buttons removed (were dead code)

### Added
- **Documentation updated** — `docs/architecture/system.md`, `docs/system/game_flow.md`, `docs/adr/adr-013-message-domain-model.md`
  - ADR-012 marked as superseded by ADR-014

## 2026-05-14

### Added
- **LLM Messages Tab** — New dashboard tab showing the last 50 LLM calls with full request/response forensics
  - `LlmMessage` model: agent name, backend, model, system/user prompts, raw request/response JSON, parsed response, error, timestamp
  - `llm_messages` SQLite table with `created_at DESC` index and strict 50-row auto-pruning cap
  - `LlmMessageStorage` trait: `save()` + `list_latest(limit)`; `SqliteLlmMessageStorage` (transactional insert+prune) + `InMemoryLlmMessageStorage` (ring buffer for tests)
  - `ChatCompletionResult` in `llm_client.rs`: returns full metadata (text, prompts, raw JSON) from the HTTP client chokepoint
  - `LlmCallResult` in `LlmBackend` trait: wraps `ChatCompletionResult` with `backend_name`, `model_name`, `agent_name`
  - All 4 LLM backends (OpenRouter, DeepSeek, Ollama, Mock) updated to return `LlmCallResult` and pass `agent_name`
  - Quantifier path logs via shared `llm_client.rs` with `agent_name = "quantifier"`
  - `/fragment/llm-messages` endpoint with `LlmMessagesTemplate` (compact expandable list, oldest-first, polled every 4s)
  - `index.html` tab button + `styles.css` styling for LLM Messages panel
  - Agent name constants: `AGENT_NARRATOR`, `AGENT_QUANTIFIER`, `AGENT_TRIGGER`, `AGENT_DIALOGUE`

### Changed
- **Documentation updated** — `docs/architecture/system.md`, `docs/system/dashboard.md`, `docs/system/llm_processing.md`, `docs/reference/testing.md`

## 2026-05-13

### Added
- **Turn + Swipe Domain Model** — Migrated from flat `Vec<LogEntry>` to structured `Vec<Turn>` with swipe-based retry
  - New `Turn` struct: stable UUID `id`, `input: LogEntry`, `swipes: Vec<Swipe>`, `active_swipe_index: u32`
  - New `Swipe` struct: `index: u32`, `entries: Vec<LogEntry>` — one generation attempt per swipe
  - `NarrativeState.history()` returns derived `Vec<LogEntry>` by flattening active swipes; all rendering/prompts unchanged
  - `Turn::create_swipe()` and `Turn::create_swipe_copying_active()` helpers for retry
  - New `Checkpoint` struct with dedicated SQLite `checkpoints` table (not in snapshot JSON)
  - `SnapshotStorage` trait extended with checkpoint CRUD: `save_checkpoint`, `load_checkpoint`, `list_checkpoints`, `delete_checkpoint`

### Changed
- **Snapshot correlation is now structural** — `turn_id` in snapshots matches `Turn.id` instead of random UUIDs
  - Renamed `message_id` → `turn_id` everywhere (`GameStateSnapshot`, `SnapshotStorage`, SQLite schema)
  - Server extracts `turn_id` from `Turn.id` after `add_log(Input)`; engine uses this ID for `pre-main:{turn_id}` and `pre-event:{turn_id}`
  - `delete_turn_snapshots()` cascades to `pre-main:{turn_id}` and `pre-event:{turn_id}` prefixed rows
  - Retry no longer breaks after delete/edit because turn identity is preserved

### Changed
- **History mutation is now turn-level** — Delete removes entire turns, not individual entries
  - `delete_history_handler` calls `delete_last_turn()` and cascades snapshot deletion via `delete_turn_snapshots()`
  - `edit_history_handler` preserves `turn_id` and `swipe_index` on the saved snapshot
  - Returns `400 Bad Request` when no turns exist (same behavior as before)

### Added
- **Swipe-aware Retry** — Retry creates new swipes instead of overwriting the same one
  - Main retry: loads `pre-main:{turn_id}`, creates new empty swipe, sets active, re-runs full pipeline
  - Event retry: loads `pre-event:{turn_id}`, creates new swipe copying main narration from previous swipe, regenerates continuation
  - `swipe_index` increments with each retry; original swipe preserved

### Added
- **Swipe Navigation UI** — Action area shows left/right arrows when a turn has multiple swipes
  - Swipe counter: "2 / 5" display
  - `POST /turn/:id/swipe/:index` endpoint switches active swipe without regeneration
  - Disabled-state bounds checking on navigation buttons

### Added
- **Checkpoint Bookmark System** — Save and restore specific turn+swipe combinations
  - `POST /checkpoint` — creates checkpoint at current turn+swipe
  - `POST /checkpoint/:id/restore` — loads snapshot, sets turn's `active_swipe_index`, re-saves state
  - `POST /checkpoint/:id/delete` — removes checkpoint
  - `GET /fragment/checkpoints` — server-rendered checkpoint list with restore/delete buttons

### Changed
- **Documentation updated** — `docs/architecture/system.md` and `docs/system/game_flow.md` reflect Turn + Swipe model
  - New ADR `docs/adr/adr-012-turn-swipe-model.md` documents rationale and trade-offs
  - Resolved TODO item: retry-after-delete bug fixed by structural turn identity

## 2026-05-12

### Added
- **Granular Retry Logic with Pre-Generation Snapshots** â€” Retry now detects event continuations vs main narration and regenerates with correct scope
  - New `StoredTriggerContext` struct stores trigger metadata (`npc_id`, `trigger_idx`, `trigger_name`, `trigger_repeat`, `trigger_prompt`, `system_prompt`, `user_prompt`, `max_tokens`) in `NarrativeState`
  - `commit_trigger_narration` populates `last_trigger` with stored prompts for exact replay
  - New player input clears `last_trigger` to `None`
  - `pre-main:{uuid}` committed snapshot saved before main LLM call
  - `pre-event:{uuid}` committed snapshot saved before trigger continuation LLM call
  - `is_last_ai_response_event_continuation()` helper detects Event log between last Input and last AI response
  - Event retry: loads `pre-event:{uuid}`, regenerates only continuation using stored prompts via `narrate_action_from_prompt`
  - Main retry: loads `pre-main:{uuid}`, re-runs full `execute_freeaction_pipeline` (narrate â†’ quantify â†’ triggers â†’ event continuation)
  - `execute_freeaction_pipeline()` extracted from `execute_action_impl` for reuse by normal actions and retry
  - First turn fallback: if no `pre-main` snapshot exists, falls back to `GameState::new()`
  - Swipe index increment: retries save with `swipe_index + 1`, preserving original snapshot

### Fixed
- **Story Log Button Visibility** â€” Delete button now only appears on the last message and is hidden when only one message exists. Retry button is also hidden on the first/only message.
  - `StoryLogTemplate` delete button wrapped in `{% if loop.last and entries|length > 1 %}`
  - `StoryLogTemplate` retry button condition changed from `{% if loop.last %}` to `{% if loop.last and entries|length > 1 %}`
- **Location Entry Text Bolding** â€” Removed CSS leak that caused all text in location entries to render bold. Only the location header (`<span class="location-header">`) is now bold.
  - Removed `font-weight: bold` from the `.location` rule in `assets/styles.css`
- **Retry UI Feedback** â€” Retry now shows immediate visual feedback
  - `retry_handler` sets `GenerationStatus::Generating` + `GenerationPhase::Narrating` and saves snapshot before spawning blocking task
  - `submitRetry()` calls `updateToThinking()` before fetch, matching form submission behavior
  - Status poll returns `narrating` within milliseconds of retry initiation

### Added
- **Reset Game Button** â€” UI control for resetting game state
  - "Reset Game" button added to `HeaderTemplate` with danger/red styling (`.reset-btn`)
  - Uses `hx-post="/reset"` with `hx-confirm` confirmation dialog
  - `reset_handler` returns `HX-Refresh: true` with empty body for clean page reload

### Fixed
- **Double-submit race condition** â€” Server now rejects concurrent async actions while generation is in flight
  - New `AppState::is_generating` (`Arc<AtomicBool>`) acts as a fast generation gate
  - `process_action` checks `compare_exchange(false, true)` before accepting async actions; rejects with `"Still thinking..."`
  - `GenerationGuard` (RAII in `src/server/fragments/generation_guard.rs`) ensures `is_generating` is cleared on `spawn_blocking` exit, even on panic
  - Client-side: HTMX `hx-sync="this:drop"` on command form prevents duplicate submissions from reaching the server
  - `saveActionArea()` JS helper now disables the submit button during request flight
  - `test_double_submit_protection` rewritten to verify rejection: first request accepted, second rejected, only first command appears in story log
  - Fixes flaky test caused by Phase 1.7 snapshot migration removing the old `Arc<Mutex<GameState>>` serialization

### Added
- **Agent Trait + Registry + Quantifier Migration (Phase 2)** â€” Migrated quantifier from hardcoded pipeline to `dyn Agent` architecture
  - New `Agent` trait with `name()`, `phase()`, `backend_selector()`, `execute()` methods
  - New `AgentRegistry` loads agents from `AppSettings.agents` config; supports `PreGeneration` and `PostGeneration` phases
  - New `AgentResult` enum: `PromptDirective`, `StatePatch`, `NoOp`
  - New `StatePatch` enum (currently `Scene { npc_ids, movement_destination, confidence }`)
  - New `AgentContext<'a>` with `state`, `main_response`, `player_input`
  - New `BackendSelector` enum: `UseMain`, `UseNamed(String)`
  - New `Confidence` enum: `High`, `Medium`, `Low` (replaces `QuantifierConfidence` in agent interface)
  - `QuantifierAgent` implements `Agent`; runs in `PostGeneration` phase
  - `NarratorAgent` stub implements `Agent`; runs in `PreGeneration` phase (reserved for future use)
  - `DefaultGameService` now owns `AgentRegistry` instead of direct `QuantifierBackendTrait`
  - `DefaultGameService::with_mock_quantifier()` helper for test injection
  - `AppSettings.agents` field with `#[serde(default = "default_agent_configs")]` for backward compatibility
  - Quantifier code moved from `src/narrative/quantifier/` â†’ `src/narrative/agents/quantifier/`
  - All quantifier tests updated to new module path; test logic unchanged

### Added
- **Structured Error Taxonomy** â€” Migrated `EngineError` from plain `String` payloads to structured types
  - New `LlmFailure` enum with variants: `EmptyResponse`, `Http { status, body }`, `Network { url, detail }`, `ParseError { raw_response, expected_format }`, `Timeout`
  - New `NarrativeFailure` enum with variants: `PromptBuild { stage, reason }`, `Generation { stage, reason }`
  - New `InternalError` struct with `invariant` field and `internal_error()` helper
  - `EngineError::Llm`, `Narrative`, `Internal` now wrap structured types via `#[source]`
  - `LlmEmptyResponse` removed â€” replaced by `Llm(LlmFailure::EmptyResponse)`
  - `llm_client.rs` return type changed from `Result<String, String>` to `crate::error::Result<String>`
  - `game_service.rs` `map_llm_error()` now uses structured `match` instead of `msg.contains(...)` string matching
  - Added `From<LlmFailure>`, `From<NarrativeFailure>`, `From<InternalError>` for `?` operator support
  - New documentation: `docs/diagnostics/error_catalog.md` â€” structured reference for every variant
  - Updated `.agents/rules/DEBUGGING.md` error taxonomy table to reference structured variants

### Changed
- **Restrict deletion to last message only** â€” Deleting any message now removes only the last entry in history
  - `delete_log(id: u64)` replaced with `delete_last_log()` which pops the final `LogEntry`
  - `POST /history/:id/delete` endpoint changed to parameterless `POST /history/delete`
  - `deleteMessage()` JavaScript handler no longer takes an `id` argument
  - Returns `400 Bad Request` when history is empty instead of `404 Not Found`
  - Component tests updated: `test_delete_history_handler_success`, `test_delete_history_handler_empty`
  - Unit test: `test_delete_last_log` in `state_tests.rs`

### Changed
- **Inline location and event headers** â€” Location and event metadata moved from separate `LogEntry` records into optional fields on the narration they annotate
  - `LogEntry` gains `location_header: Option<String>` and `event_header: Option<String>`
  - `NarrativeState` gains `pending_location: Option<String>` and `pending_event: Option<String>`
  - `add_log` consumes pending metadata into the new entry's fields
  - `handle_movement` sets `pending_location` instead of calling `add_log` for a standalone location entry
  - `commit_trigger_narration` and `evaluate_and_narrate_triggers` set `pending_event` instead of adding a `LogType::Event` entry
  - `is_last_ai_response_event_continuation` simplified to check `event_header.is_some()` on the last AI response
  - `StoryLogTemplate` renders headers inside the same div as the narration text
  - Browser tests updated to stop skipping `.location` entries (they now have text)

  - Template tests: `test_story_log_template_renders_event_header`, `test_story_log_template_renders_location_header`
  - Engine tests: `test_handle_movement_sets_pending_location`, `test_commit_trigger_narration_adds_event_header_and_narration`, `test_evaluate_and_narrate_triggers_adds_event_header`
  - State tests: `test_add_log_absorbs_pending_location`, `test_add_log_absorbs_pending_event`

### Fixed
- **Settings panel encoding and checkbox spacing** - Fixed UI defects in the settings panel
  - Replaced corrupted UTF-8 em-dash (`Ã¢â‚¬"`) with simple hyphen (` - `) in provider/model display strings
  - Added explicit `.checkbox-label` class to checkbox labels for better browser compatibility
  - Updated CSS to target `.checkbox-label` instead of `label:has(> input[type="checkbox"])`
  - Increased checkbox label gap from `var(--spacing-xs)` (4px) to `var(--spacing-sm)` (8px)

### Fixed
- **Test environment isolation** - Fixed tests that failed when `OPENROUTER_API_KEY` env var is set
  - `settings_tests::test_connection_resolve_api_key` now asserts against the env var value instead of hardcoded `None`
  - `game_service_tests` that relied on `DefaultGameService::new()` having no API key now use `DefaultGameService::with_backends()` with explicit `MockBackend::failing()`
  - Tests are now independent of host environment variables

### Fixed
- **Sequential trigger display** - Main narration and trigger text now appear sequentially instead of simultaneously
  - Split `evaluate_and_narrate_triggers` into three phases: evaluate (lock) â†’ LLM (unlock) â†’ commit (lock)
  - Frontend can now poll and display the main narration while the trigger continuation is still generating
  - `execute_freeaction_impl` returns `Option<TriggerContinuationRequest>` for orchestration in `game_service.rs`
  - New `commit_trigger_narration()` function adds event header + narration logs and marks triggers fired

### Added
- **Spell & Grammar Check Integration** - Pre-flight text checking for player input via harper-core
  - New `narrative/text_check/` module: `HarperBackend`, `CheckResult`, `CheckIssue`, `IssueKind`
  - `TextCheckMode` enum: `Disabled`, `Spell`, `Grammar`, `SpellGrammar`
  - `TextCheckSettings` in `AppSettings` with mode, `enable_auto_check`, and `ignored_words`
  - Merged dictionary strategy: `FstDictionary::curated()` + `MutableDictionary` for user-ignored words
  - `POST /action/check` handler: automatic pre-flight check before LLM submission
  - `POST /check-text` handler: manual on-demand text checking
  - `TextCheckPreviewTemplate` Askama template for original vs corrected comparison UI
  - Player can always choose "Send Original" to bypass corrections
  - Fail-open: if linting fails, original text is forwarded silently
  - Tests: `tests/text_check_tests.rs` with 4 integration tests
  - Documentation: `docs/system/text_check.md`

### Added
- **File Length Guard Rails** - Enforced 2,000 non-blank line limit on all `.rs` files
  - New `tests/guardrails.rs` rules: `guardrails_file_length_src`, `guardrails_file_length_tests`
  - `docs/architecture/guardrails.md` updated with file length policy
- **Test File Extraction** - All inline `#[cfg(test)]` blocks moved to separate `*_tests.rs` files
  - 31 new sibling test files across `src/` (e.g., `logic.rs` â†’ `logic_tests.rs`)
  - Parent `mod.rs` files updated with `#[cfg(test)] mod xxx_tests;` declarations
  - Eliminates file-length violations and improves build parallelism
  - New `scripts/check_test_structure.py` guardrail bans inline test blocks
- **Marinara-Style Prompt Rules** - Overhauled `SYSTEM_PROMPT_TEMPLATE` with battle-tested patterns from Marinara Engine
  - Free will framing: "you have your own free will, intellect, and emotional intelligence"
  - Anti-repetition rule with concrete example ("Gooner?" â†’ "What type of question is that?")
  - Anti-GPTism ban on generic structures and clichÃ©s ("jaws working", "physical punches")
  - Knowledge boundary rules: latecomers ignorant, private conversations stay private, rumors travel slowly
  - Character complexity requirement: opinions, contradictions, boundaries, hypocrisies, judgments
  - Proactive narrative momentum: introduce challenges, resist comfort, no plot armor
  - Internal thought barrier: thoughts via narration are never audible to others
  - Positive framing: "describe what DOES happen, rather than what doesn't"
  - Scattered prohibitions (removed dedicated "Never do" bulleted list)
- **Response Length Setting** - Configurable `response_length` in `AppSettings` / `settings.json`
  - Injected into system prompt via `PromptBuilder::with_response_length()`
  - Default: flexible scene-adaptive guidance (concise for dialogue, longer for transitions)

### Fixed
- **Duplicate `global_rules` removal** - `global_rules` no longer appear in both system prompt and `<WorldLore>` user layer
  - Now injected **only** in `render_system_layer()` (Layer 0)
  - Saves tokens and reduces redundancy

### Added
- **Gemma 4 Thinking-Channel Suffix** - Fixed infinite reasoning loop on Gemma 4 26B models
  - `apply_gemma4_thinking_suffix()` in `llm_client.rs` detects Gemma 4 models by name
  - Appends `<|turn>model\n<|channel>thought\n<channel|>` to Ollama user messages
  - Tells the model the thinking slot is already filled, bypassing the loop
  - Validated on `mradermacher/gemma-4-26b-a4b-it-abliterated:iq2xs`: 2048 tokens all-reasoning â†’ ~211 tokens of narrative content
  - Non-Gemma models are completely unaffected

### Fixed
- **Gemma 4 suffix corruption** â€” Fixed malformed thinking suffix that was causing `<channel|>` prefixes and `<thought>` blocks in output
  - Removed erroneous leading `<turn|>` line from suffix (now matches SillyTavern preset exactly)
  - Scoped suffix to Ollama backend only; OpenRouter's native chat template was fighting the injected raw tokens
  - Added `sanitize_llm_output()` to strip leaked thinking artifacts from all responses
- **Marinara-Style Prompt Architecture** - Refactored prompt construction to plain-text instructions + XML-wrapped data only
  - System prompt (Layer 0) is now plain text â€” removed `<SystemPrompt>`, `<Role>`, `<CoreRole>`, etc.
  - PHI layer (Layer 7) is now plain text â€” removed `<AuxiliaryInstructions>` wrapper
  - Quantifier prompt instructions are now plain text â€” removed `<QuantifierTask>` and `<Query>` wrappers
  - XML tags remain only for external data: `<GameState>`, `<KnownNpcs>`, `<ConversationHistory>`, `<CurrentRoom>`, etc.
  - Fixes Gemma 4 reasoning-loop bug where self-referential XML triggered meta-analysis instead of execution
- **Per-Connection Context Windows** - Added `max_context_tokens` to `Connection` settings
  - Defaults: 8192 for Ollama, 32768 for OpenRouter/DeepSeek, 4096 for Mock
  - Optional field â€” existing `settings.json` loads without modification
- **Context-Aware Token Fitting** - `fit_messages_to_context()` dynamically caps `max_tokens` and trims oldest history first
  - New constants: `SAFETY_MARGIN_TOKENS` (256), `MIN_INPUT_BUDGET_TOKENS` (512)
  - `build_split()` now returns fitted `(system, user, max_tokens)` using the active connection's context window

### Changed
- **Token budget defaults** - `DEFAULT_MAX_TOKENS` increased from 1024 to 2048
- **`build_split()` separation** - System half now contains plain-text instructions only; user half contains all XML-wrapped data + PHI
- **`build()` return type** - Now returns `(prompt, max_tokens)` to include the context-fitted token limit
- **LLM backend trait** - `narrate_continuation` and `narrate_action_from_prompt` now accept optional `max_tokens` to pass fitted limits through

### Added
- **Granular Status Phases** - The UI now shows distinct status messages during each stage of LLM processing
  - New `GenerationPhase` enum with three variants: `Narrating`, `Quantifying`, `GeneratingEvent`
  - Added `phase` field to `GenerationState` alongside existing `status`
  - `GenerationStatus` (Idle/Generating/Error) remains unchanged for backward compatibility
  - `is_generating()` remains the single source of truth for disabling UI elements
  - Phase is a secondary display concern only â€” all phases use unified `.thinking` CSS class
  - `/status/generating` endpoint returns phase names (`narrating`, `quantifying`, `generating-event`)
  - Frontend maps endpoint values to human-readable text via `onStatusPoll()`
  - Optimistic "Thinking..." still shown immediately on form submit before first poll
  - Pipeline phases:
    - `Narrating` â€” During main LLM narration (Phase 4)
    - `Quantifying` â€” During post-narration quantifier analysis (Phase 4.5)
    - `GeneratingEvent` â€” During trigger continuation narration (Phase 5), only when a trigger actually fires

### Changed
- **Trigger evaluation simplified** â€” Only the first matching trigger is processed per action
  - Removed `max_triggers` parameter from `evaluate_and_narrate_triggers`
  - Replaced loop with single `if let Some(...)` for first match only
  - `GeneratingEvent` phase only set when a trigger is found and about to call LLM
  - Removed redundant `get_current_room()` call in trigger evaluation (uses `trigger_context.room` directly)

### Fixed
- **Edit textarea sizing** - Textarea now preserves original text height using `getBoundingClientRect()` + padding/border compensation, with auto-resize on input
- **PHI layer missing from split prompts** - `build_split()` now includes Layer 7 (PHI) in the user message, preserving the same ordering as `build()` where behavioral instructions sit closest to generation
- **Settings UI** - Restored accidentally corrupted `data/settings.json` model entry

### Added
- **Single User Message mode** - Per-connection toggle for models that ignore system prompts
  - New `single_user_message` field on `Connection` struct
  - Checkbox in Add/Edit connection forms
  - When enabled, merges system + user into one user message with `[SYSTEM]\n` prefix
  - Empty system messages are omitted from the API payload
  - Added `merge_single_user_message()` helper and coverage tests
- **OpenRouter header** - Added `HTTP-Referer` header alongside existing `X-Title`

### Changed
- **Prompt system docs** - Updated `prompt_system.md` to document PHI placement in user half of split prompts
- **UI docs** - Updated `ui_design.md` and `dashboard.md` to reflect tab bar, settings panel, connection cards, and edit form
- **Test docs** - Updated `testing.md` with accurate test counts and new test files
- **Game flow docs** - Updated `game_flow.md` with granular status phase documentation

### Added
- **Room-Aware Triggers** - Triggers can now be scoped to specific rooms via `room_id`
  - Added optional `room_id` field to `Trigger` schema
  - Global triggers (no `room_id`) fire anywhere (backward compatible)
  - Room-scoped triggers only fire when `state.current_room_id` matches
  - Gabriella's introduction trigger now scoped to `entrance_hall`
  - Prevents NPC introduction events from firing in the wrong location

### Changed
- **Default backend fixed** - `data/settings.json` now defaults to `OpenRouter` instead of `Mock`
- **Mock backend hidden from UI** - Removed "Mock (Testing)" from the Settings backend dropdown. `Mock` remains available for tests via `DefaultGameService::with_backends()` but is no longer selectable by end users

### Added
- Settings system with tabbed UI for LLM configuration (backend, model, quantifier model, API key)
- `data/settings.json` for persistent configuration
- **Dependency-Injected Backends** - `DefaultGameService` now owns its backends via `Arc<dyn Trait>`, eliminating global state and test flakiness
  - `DefaultGameService::with_backends(llm, quantifier)` constructor for test injection
  - Removed all global test-override atomics (`TEST_BACKEND_OVERRIDE`, `TEST_QUANTIFIER_OVERRIDE`, RAII guards)
  - `FreeActionContext` carries `&dyn LlmBackend` to thread backends through `evaluate_and_narrate_triggers`
  - All 17 `game_service_tests` converted to DI; timeouts reduced to 200ms (no disk I/O races)
- **Coverage Improvement** - `game_service.rs` coverage increased from 58% to 79% (llvm-cov)
  - Extracted `execute_freeaction_impl` to `action_processing.rs` for testability
  - Added 6 new integration tests covering FreeAction success, retry, and movement paths
- **Event Header Entries** - Named triggers now render visual event banners in the story log
  - `TriggerAction` requires a `name` field (e.g., "Gabriella Introduction")
  - New `LogType::Event` variant for event header entries
  - Event headers appear before trigger narration, styled in blue/cyan (`#38bdf8`)
  - Event entries have no edit/retry buttons (same as location headers)
  - Updated all world data (`gabriella.json`, `shopkeeper.json`, `ranger.json`) with trigger names
- **Decoupled Characters and Players from Worlds** - Characters and player personas are now stored outside world directories, enabling sharing across worlds
  - Characters moved from `data/worlds/<world>/characters/` to `data/characters/<group>/`
  - Players moved from `data/worlds/<world>/player.json` to `data/personas/<name>.json`
  - `WorldManifest` now has a `characters_dir` field to specify which character group to load
  - `player_file` in `WorldManifest` now resolves relative to `data/personas/`
  - Map files remain in `data/worlds/<world>/map.json`

## 2026-04-29

### Added
- **Retry Handler Implementation** - The `/retry` endpoint now actually regenerates AI responses
  - Added `replace_last_ai_response(new_text)` method to `GameState`
  - Added `get_history_context_for_retry()` - Returns history excluding AI response being retried
  - Retry now calls LLM with original user input and truncated history
  - Critical: History truncation prevents LLM from repeating old response

### Fixed
- **Retry endpoint** - Was returning stub "Retrying..." without actual LLM call
- **History context** - Retry now properly excludes the AI response being retried from LLM context

## 2026-04-28

### Added
- **History Edit & Retry** - Users can now edit past conversation entries and regenerate the last AI response
  - Added `id: u64` to `LogEntry` for unique identification
  - Added `next_log_id: u64` to `GameState` for auto-increment
  - `edit_log(id, new_text)` method to modify entry text
  - `get_last_input_text()` to retrieve last user input for retry
  - `POST /history/:id` endpoint for editing entries
  - `POST /retry` endpoint to regenerate last response
  - UI Edit button (pencil icon) appears on log entry hover
  - Inline text editing (no modal)
  - Retry button on last AI message (Narration/Dialogue)

### Changed
- **StoryLogTemplate** - Now includes `data-id` and `data-raw-text` attributes on each entry
- **LogEntryView** - Added `raw_text` field to preserve original markdown
- **Edit behavior** - Uses `data-raw-text` attribute to get original text (not HTML)
- **Polling pause** - HTMX polling pauses during edit mode to prevent DOM replacement

### Added
- **Trigger continuation unified** - Trigger narrations now use full 8-layer sillytavern prompt via `PromptBuilder` with continuation context in user message
- **Removed continuation.rs** - Functionality migrated to unified prompt system
- **Added PhiMode** - ~~New enum controlling PHI layer (Layer 7) behavior: Narration vs Continuation~~ (removed in later refactor â€” PHI is now universal)
- **Quantifier Backend Trait** - Refactored quantifier to use trait for enable testing
  - New `QuantifierBackendTrait` interface with `quantify_room()` method
  - `RealQuantifierBackend` - Production LLM-based implementation
  - `MockQuantifierBackend` - Test implementation returning High confidence with configurable NPCs
  - Set `LLM_BACKEND=mock` env var to use mock for testing

- **action_processing.rs** - Extracted pure functions from fragments.rs for unit testing
  - `get_static_npcs()` - Returns NPCs for current room (removed in later refactor; NPC presence now driven entirely by quantifier + scenario init)
  - `handle_movement()` - Processes player movement
  - `apply_npc_events()` - Handles NPC Entered/Left events
  - `evaluate_and_narrate_triggers()` - Evaluates narrative triggers

### Changed
- **fragments.rs** - Now uses trait-based quantifier and action_processing module
  - Selects mock/real backend based on `LLM_BACKEND` env var
  - Delegates to extracted action_processing functions
- **Quantifier timing** - Movement detection now runs AFTER narration generation (from narration text), not before
  - Old: Quantifier ran BEFORE narration to detect movement intent from player action text
  - New: Narration generated first, then quantifier detects movement from generated text
  - This ensures the location header is added at the right time

### Fixed
- **Coverage target** - Now maintains ~87% line coverage (excludes async/server code)
  - Excluded `fragments.rs`, `mod.rs` from coverage
  - Added unit tests for action_processing functions

