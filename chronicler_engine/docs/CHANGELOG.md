# Changelog

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
