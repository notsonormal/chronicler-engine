# Changelog

## Unreleased

### Added
- **Marinara-Style Prompt Rules** - Overhauled `SYSTEM_PROMPT_TEMPLATE` with battle-tested patterns from Marinara Engine
  - Free will framing: "you have your own free will, intellect, and emotional intelligence"
  - Anti-repetition rule with concrete example ("Gooner?" → "What type of question is that?")
  - Anti-GPTism ban on generic structures and clichés ("jaws working", "physical punches")
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
  - Validated on `mradermacher/gemma-4-26b-a4b-it-abliterated:iq2xs`: 2048 tokens all-reasoning → ~211 tokens of narrative content
  - Non-Gemma models are completely unaffected

### Fixed
- **Gemma 4 suffix corruption** — Fixed malformed thinking suffix that was causing `<channel|>` prefixes and `<thought>` blocks in output
  - Removed erroneous leading `<turn|>` line from suffix (now matches SillyTavern preset exactly)
  - Scoped suffix to Ollama backend only; OpenRouter's native chat template was fighting the injected raw tokens
  - Added `sanitize_llm_output()` to strip leaked thinking artifacts from all responses
- **Marinara-Style Prompt Architecture** - Refactored prompt construction to plain-text instructions + XML-wrapped data only
  - System prompt (Layer 0) is now plain text — removed `<SystemPrompt>`, `<Role>`, `<CoreRole>`, etc.
  - PHI layer (Layer 7) is now plain text — removed `<AuxiliaryInstructions>` wrapper
  - Quantifier prompt instructions are now plain text — removed `<QuantifierTask>` and `<Query>` wrappers
  - XML tags remain only for external data: `<GameState>`, `<KnownNpcs>`, `<ConversationHistory>`, `<CurrentRoom>`, etc.
  - Fixes Gemma 4 reasoning-loop bug where self-referential XML triggered meta-analysis instead of execution
- **Per-Connection Context Windows** - Added `max_context_tokens` to `Connection` settings
  - Defaults: 8192 for Ollama, 32768 for OpenRouter/DeepSeek, 4096 for Mock
  - Optional field — existing `settings.json` loads without modification
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
  - Phase is a secondary display concern only — all phases use unified `.thinking` CSS class
  - `/status/generating` endpoint returns phase names (`narrating`, `quantifying`, `generating-event`)
  - Frontend maps endpoint values to human-readable text via `onStatusPoll()`
  - Optimistic "Thinking..." still shown immediately on form submit before first poll
  - Pipeline phases:
    - `Narrating` — During main LLM narration (Phase 4)
    - `Quantifying` — During post-narration quantifier analysis (Phase 4.5)
    - `GeneratingEvent` — During trigger continuation narration (Phase 5), only when a trigger actually fires

### Changed
- **Trigger evaluation simplified** — Only the first matching trigger is processed per action
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
- **Added PhiMode** - ~~New enum controlling PHI layer (Layer 7) behavior: Narration vs Continuation~~ (removed in later refactor — PHI is now universal)
- **Quantifier Backend Trait** - Refactored quantifier to use trait for enable testing
  - New `QuantifierBackendTrait` interface with `quantify_room()` method
  - `RealQuantifierBackend` - Production LLM-based implementation
  - `MockQuantifierBackend` - Test implementation returning High confidence with configurable NPCs
  - Set `LLM_BACKEND=mock` env var to use mock for testing

- **action_processing.rs** - Extracted pure functions from fragments.rs for unit testing
  - `get_static_npcs()` - Returns NPCs for current room
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

## 2026-04-26

### Added
- **NPC Event Layer** - Quantifier now tracks NPC enter/leave events
  - New types: `NpcEventType` (Entered, Left), `NpcEvent`, `NpcEventList`
  - `compute_npc_events()` function compares previous vs current NPC presence
  - `QuantifierResult` now includes `npc_events: NpcEventList`
  - `times_met` only increments on `Entered` events (new encounters)
  - `currently_meeting` set to true on `Entered`, false on `Left`
  - This addresses the TODO: tracking NPC movement, not just player movement

### Fixed
- **Trigger evaluation timing bug** - Triggers not firing because `times_met` was incremented BEFORE trigger evaluation
  - `TimesMet Eq 0` triggers would never fire because the counter was already 1
  - Fixed: Evaluate triggers BEFORE incrementing `times_met` in `fragments.rs`
  - Now triggers see `times_met = 0` when evaluating, allowing `TimesMet Eq 0` conditions to fire

- ** Gabriella not detected** - Second quantifier now runs after main narration to detect NPCs in generated text
  - Added two-stage quantifier: first before action (movement), second after narration (NPC detection)
  - This catches NPCs like Gabriella who appear dynamically in the narration

- **Multiple NPCs checked** - Triggers now evaluate ALL NPCs, not just `npcs_in_area`
  - Changed `evaluate_triggers` to iterate `state.npcs.values()` instead of `state.npcs_in_area`
  - This catches NPCs who appear in narration but weren't in the initial room config

- **Trigger order** - Main narration now appears BEFORE trigger continuation
  - Fixed reordering in `fragments.rs`: add_log → then trigger evaluation

- **NPC name prefix** - Trigger narration no longer includes NPC name prefix
  - Changed sender from `Some(npc.sheet.name.clone())` to `None`

### Added
- **Unit tests for new trigger behavior**:
  - `test_currently_meeting_tracks_encounters` - Tests the currently_meeting flag
  - `test_increment_times_met_always_increments` - Tests increment behavior
  - `test_character_state_initializes_with_starting_room_npcs` - Tests starting room NPCs
  - `test_evaluate_triggers_fires_for_npc_not_in_area` - Tests ALL NPCs get evaluated

- **Integration test for second quantifier flow**:
  - `test_second_quantifier_detects_room_npcs` - Tests movement to room with configured NPCs triggers detection

- **Mock backend support for trigger continuation** - `LLM_BACKEND=mock` now works for trigger narration

### Changed
- **times_met semantics** - Counter increments on room entry, not on trigger fire
- **TimesMet conditions** work correctly now that evaluation happens before increment

## 2026-04-20

### Added
- **Reactive Auto-Trigger Movement** - Character-state-based NPC trigger system
  - New `model/trigger.rs` module: `Trigger`, `TriggerCondition`, `TriggerAction`, `NpcEncounterState`, `CharacterState`
  - `TriggerCondition` supports `TimesMet(Eq|Lt|Gte, u32)` comparisons
  - `CharacterState` tracks per-NPC encounter counts in-memory
  - `NpcCard.triggers: Vec<Trigger>` field with `#[serde(default)]` for backward compatibility
  - `GameState.character_state: CharacterState` for persistent in-session NPC encounter tracking

- **Trigger Evaluation Engine** - Pure evaluation functions for NPC triggers
  - New `engine/trigger_eval.rs`: `evaluate_triggers`, `check_condition`, `increment_times_met`, `mark_trigger_fired`
  - Returns matching `(NpcCard, Trigger)` tuples for NPCs in current room
  - Handles missing character state gracefully (defaults to `times_met = 0`)
  - Non-repeatable triggers tracked and skipped after first fire

- **Continuation Prompt Builder** - Second LLM prompt for trigger narration
  - New `narrative/continuation.rs`: `build_continuation_prompt`
  - System prompt instructs LLM to continue scene without repetition
  - User prompt includes first narration + room context + trigger text
  - Token budgeting: truncates first narration to fit `MAX_CONTEXT_TOKENS`

- **Recursive Auto-Trigger Flow** - Integration in `fragments.rs` process_action
  - After first LLM narration + quantifier movement detection
  - `evaluate_triggers` called on successful room transition
  - Second LLM call per trigger with continuation prompt (max 3 to prevent runaway)
  - `is_generating` stays true through ALL LLM calls (no reset between narrations)
  - Failed second calls: first narration still displays, error logged, state resets
  - Trigger narrations marked non-movement (quantifier skipped for them)
  - `times_met` incremented and non-repeatable triggers marked after each fire

- **Trigger Tests** - Comprehensive mock LLM test suite
  - New `tests/trigger_tests.rs`: 5 integration tests (requires Playwright + mock LLM server)
  - Tests: first encounter fires, second encounter skipped, multiple triggers, non-repeatable behavior, LLM failure graceful degradation

### Documentation Updated
- `architecture/system.md` - New Model tier (Trigger, CharacterState), Engine tier (trigger_eval), Narrative tier (continuation)
- `reference/data_schemas.md` - Trigger, NpcEncounterState, CharacterState schemas; NpcCard updated with triggers field
- `system/narration_engine.md` - Continuation narration flow, trigger evaluation, `is_generating` behavior
- `system/navigation.md` - Auto-trigger phase after movement, quantifier skip for triggers
- `system/game_flow.md` - Phase 3.5: Trigger Evaluation, dual LLM call in Phase 4

### Data Updated
- `data/worlds/redmist_estate/characters/gabriella.json` - First-encounter trigger example
- `data/worlds/test/characters/shopkeeper.json` - `TimesMet Eq 0` trigger (non-repeatable)
- `data/worlds/test/characters/ranger.json` - `TimesMet Lt 3` trigger (repeatable)
- `data/worlds/test/characters/bartender.json` - No triggers (control case)

## 2026-04-18 (continued)

### Added
- **Scene Quantification (Dual-LLM Architecture)** - Dynamic NPC presence detection
  - New `quantifier.rs` module with `QuantifierBackend`, `QuantifierPromptBuilder`, and response parser
  - Secondary LLM model via `QUANTIFIER_MODEL` env var (defaults to free model)
  - Quantifier prompt includes: room info, previous room NPCs, last 4 history entries, player action
  - Response parsing: JSON-first → text fallback → validation against known NPCs
  - Confidence levels: High (JSON), Medium (text fallback), Low (use static NPCs)
  - Automatic fallback to static `room.npcs` when quantifier fails
  - Integration in `fragments.rs` WalkTo action handler

- **Quantified NPCs Sidebar** - Persistent NPC list for visual sidebar
  - New `npcs_in_area: Vec<NpcCard>` field in `GameState` for storing quantifier results
  - Visual sidebar now reads from stored quantifier result instead of static room.npcs
  - Re-quantification triggers after EVERY LLM generation (LLM decides NPC presence)
  - Fallback to static room.npcs when quantifier unavailable or npcs_in_area empty
  - Added 4 tests for npcs_in_area field and sidebar behavior

- **OpenRouter Client Enhancement** - Dual model support
  - Added `call_openrouter_with_model()` for flexible model selection
  - Added `get_llm_model()` and `get_quantifier_model()` helper functions
  - Original `call_openrouter()` refactored to use the new helper

- **FreeAction NPC Fix** - Fixed empty NPC list in free actions
  - `fragments.rs` FreeAction handler now correctly fetches static NPCs from room

## 2026-04-18

### Changed
- **Visual Sidebar Images** - Improved NPC portrait visibility
  - Changed NPC grid from 2-column to single column layout
  - Images now display at approximately double the previous width
  - Makes character portraits more visible and easier to identify

### Added
- **Headshot Image Support** - Enhanced character and room image handling
  - New fields in `CharacterSheet`: `profile_image` and `headshot_image` (both Optional<String>)
  - Room images use existing `image_path` field in map JSON
  - Visual sidebar now displays NPC portraits in 2-column grid (per UI spec)
  - Images in visual sidebar are clickable to toggle sidebar expand/collapse
  - CSS added: cursor:pointer, hover states for images
  - Integration tests added for world data loading with image paths

### Changed
- **Visual Sidebar Layout** - NPCs now displayed inside visual sidebar (20% column) with grid layout
  - Removed separate character-headshots section that was blocking game text
  - NPCs use headshot_image with fallback to image_path
  - Grid: 2 columns desktop, responsive breakpoints

## 2026-04-17

### Added
- **PromptContext Refactoring** - Unified context for LLM calls
  - New `PromptContext` struct in `prompt.rs` containing all prompt fields
  - `PromptBuilder::from_context()` method creates context from game state
  - Simplified `LlmBackend` trait to use `PromptContext`
  - All 3 LLM methods now take `&PromptContext` instead of individual fields
  - Cleaner backend implementations (OpenRouter, Mock, DeepSeek)

- **NPC Prompt Structure** - All characters now included in LLM prompts
  - New fields in PromptBuilder: `all_npcs` and `npcs_in_area`
  - Two output sections: `<KnownNpcs>` (condensed roster of all characters) and `<NpcsInRoom>` (room-specific full cards)
  - Presence status shows "(in room)" or "(elsewhere)"

### Changed
- **OpenRouter Client** - Enhanced content extraction
  - Robust fallback chain: content → reasoning → reasoning_content
  - Added is_non_empty() helper to check both null and empty string
  - Added logging to show which extraction path was used

## 2026-04-16

### Changed
- **Location Display** - Location now shown in story log as inline header
  - Template modified: location entries rendered as "Room Name - HH:MM" inline
  - Added `is_location` field to `LogEntryView` for detection (sender + empty text)
  - Location removed from header template to story log
  - Green color (#4ade80) with bold styling
  - CSS classes `.location-header` and `.location-timestamp` added

### Changed
- **Game Start Flow** - Simplified startup
  - Removed "Welcome to..." and "Logged in as..." system messages
  - Scenario text directly shows without extra system entries
  - Location entry created when using WalkTo action

### Fixed
- **Tests Updated** - Updated tests for new location display behavior
  - `tests/template_tests.rs` - check connection-status instead of location in header
  - `tests/spec_tests.rs` - check `.location-header` in story log
  - `tests/flow_mock_tests.rs` - check `.location-header`
  - `tests/ui_tests.rs` - check `.location-header`

### Added  
- **build.py** - New build script in `scripts/build.py`
  - Runs: cargo build, cargo clippy, cargo test, cargo llvm-cov

## 2026-04-14

### Added
- **Starting Scenarios** - Configurable narrative introductions that play at game start
  - New `src/model/scenario.rs` with `StartingScenario` struct
  - New `scenarios` field in `WorldManifest` (in `world.json`)
  - Template variable `{{user}}` substituted with player name
  - Scenario text replaces LLM call for first response
  - Backward compatible (worlds without scenarios use LLM fallback)
  - Example scenarios added to `redmist_estate` and `test` worlds
  - Auto-selects first scenario in array

### Added
- **Integration Test Infrastructure** - Dynamic port allocation and config-based LLM backend
  - New `tests/test_config.json` with port range (3010-3030) and backend settings
  - New `TestConfig`, `get_available_port()`, `get_config_port()` in test_utils.rs
  - All 6 test files now use dynamic port allocation
  - Config-based LLM backend selection with test-specific overrides
  - Backward compatibility with LLM_BACKEND env var

### Changed
- **Room Entry LLM** - Room entry now shows LLM-generated narration
  - WalkTo shows minimal header (room name) instead of static description
  - LLM auto-triggers on first game load
  - NPCs from target room now included in LLM prompts

## 2025-04-14

### Added
- **System Prompt XML Refactor** - Converted prompt sections from `=== HEADER ===` to XML-wrapped format
  - All 8 prompt sections now use `<Header>content</Header>` format
  - Updated `src/narrative/prompt.rs` with opening and closing XML tags
  - All 108 tests pass
  - Sections: SystemPrompt, GameState, NpcPresence, PlayerCharacter, WorldLore, ConversationHistory, PlayerInput, AuxiliaryInstructions

### Changed
- **LLM Context Pipeline** - SillyTavern-style layered prompt system
  - New `src/narrative/prompt.rs` module with `PromptBuilder`
  - 8-layer prompt construction (System, Game State, NPC Cards, Player, World Info, History, User Input, PHI)
  - Token budget management with hard truncation
  - Prompt injection sanitization
  - Updated `src/narrative/llm.rs` to use PromptBuilder with full history
  - Support for OpenRouter and DeepSeek backends
  - 30+ unit tests for prompt building and sanitization
  - See `docs/architecture/system.md` and `docs/system/llm_processing.md` for specs

### Added
- **Askama template migration (pilot)** - Migrated header template from manual `format!` strings to Askama
  - New `src/server/templates.rs` with compile-time validated `HeaderTemplate`
  - New `tests/template_tests.rs` with fast unit tests (<1ms vs ~5s for integration tests)
  - Compile-time validation: missing field = compiler error
  - Added `askama = "0.12"` to dependencies

### Changed
- `src/server/fragments.rs` now uses `HeaderTemplate` instead of manual string formatting

### Added (Full Migration)
- **Full Askama migration** - All 4 templates now use Askama (complete)
  - `StoryLogTemplate` - Renders narration history with auto-escaped text
  - `VisualSidebarTemplate` - Renders room image + NPC portraits
  - `ActionAreaTemplate` - Renders command form with state-aware disabled
  - 12 unit tests in `src/server/templates.rs` (all pass)
  - Rust 2024 compatible (avoided reserved words in CSS)

## 2025-04-12

### Added
- Multi-world support with CLI arguments (`--world`, `--port`, `--list-worlds`)
- Data organized under `data/worlds/<world_id>/`
- Test world at `data/worlds/test/` for UI tests
- UI tests spawn self-managed server on port 3001
- Auto-kill existing process when port is in use
- Static file serving for `/data/images/` and `/assets/`
- Image endpoint route `/data/images/:file` for serving character images
- UI tests for image loading and NPC image visibility
- `run_background.ps1` script for manual testing

### Changed
- Migrated from Ratatui TUI to HTMX web dashboard
- Server added with Axum + WebSocket for real-time updates
- Added `crate::server::*` module, removed `crate::ui::*`
- Fallback service now serves from `assets` directory
- Use `unpkg.com` CDN for HTMX and WS extension (jsdelivr issues on Windows)

### Fixed
- Static image 404s by adding explicit routes and services for `/data/images/`
- Server not binding on Windows (use `Start-Process -WindowStyle Hidden`)
- WS extension not loading (CDN issue - switched to older version 2.0.3)
