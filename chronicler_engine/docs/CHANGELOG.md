# Changelog

## 2026-04-27

### Added
- **Trigger continuation unified** - Trigger narrations now use full 8-layer sillytavern prompt via `PromptBuilder` with `PhiMode::Continuation`
- **Removed continuation.rs** - Functionality migrated to unified prompt system
- **Added PhiMode** - New enum controlling PHI layer (Layer 7) behavior: Narration vs Continuation
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
  - Excluded `fragments.rs`, `mod.rs`, `openrouter_client.rs` from coverage
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
  - Two output sections: `<Npcs>` (all characters with presence) and `<NpcsInRoom>` (room-specific)
  - Presence status shows "(IN ROOM)" or "(elsewhere)"

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
