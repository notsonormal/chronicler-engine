# Changelog

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

