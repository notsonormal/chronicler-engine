# chronicler_engine/tests/

## Responsibility

Integration tests for the Chronicler Engine, covering component logic, end-to-end browser flows, LLM interactions, trigger evaluation, and navigation. Uses `playwright_rs` for browser automation and a `TestServer` harness for server lifecycle management.

## Design

**Test organization:**
- `component_tests.rs` — Merged from `template_tests.rs` and `fragment_tests.rs`. Contains:
  - Template rendering tests (`HeaderTemplate` with XSS security verification)
  - HTTP endpoint tests (fragment endpoints: `/fragment/header`, `/fragment/story-log`, `/fragment/visual-sidebar`, `/fragment/action-area`, `/fragment/character-headshots`)
  - Action handler tests (`/action`, `/hints`, `/status/ready`, `/status/generating`, `/status/reset-generating`, `/history/:id`, `/retry`)
  - Integration tests loading real world data from JSON (`test` and `redmist_estate` worlds)
  - Tests for `npcs_in_area` field in GameState (initialization, population, clear, replace)
- `e2e_tests.rs` — Merged from `spec_tests.rs`, `behavior_tests.rs`, `ui_tests.rs`, `layout_tests.rs`. Contains:
  - UI structure tests (page load, header display, connection status indicator)
  - Action area tests (form elements, input validation, form submission)
  - Story log tests (populated, scrollable)
  - Layout tests (no horizontal overflow, element positioning)
  - Visual sidebar and action hints tests
  - Static shell test (form stays in DOM after submission)
  - CSS tests (file loads, CSS variables, responsive breakpoints, scrollbar styling)
  - NPC portrait layout tests (horizontal layout, fixed width)
  - Edit/Retry UI tests (edit button existence, edit mode activation, cancel restores original, polling pauses during edit, retry button on last AI message, textarea height matching)
- `flow_llm_tests.rs` — Real LLM API call tests through the full game loop:
  - `test_llm_generates_narration_for_free_action` — Verifies real LLM generates narrative
  - `test_llm_narration_appears_via_polling` — Verifies HTMX polling catches LLM responses with status transitions
  - `test_llm_handles_arrival_narration` — Verifies navigation triggers LLM narration
  - Helper functions: `send_action()`, `get_story_log_summary()`, `wait_for_narration_increase()`
- `flow_mock_tests.rs` — Mock LLM backend tests (fast, no API key needed):
  - Initial load tests (location display, story log content, status ready)
  - Command submission tests (look command shows thinking status)
  - Uses shared browser pattern (`get_shared_browser()`) for efficiency
- `trigger_tests.rs` — Trigger evaluation and continuation narration:
  - `test_first_encounter_trigger_fires` — Verifies trigger fires when `times_met == 0`
  - `test_second_quantifier_detects_room_npcs` — Verifies RoomConfiguredNpcs triggers
  - `test_no_trigger_for_npc_without_triggers` — Control case for NPCs without triggers
  - `test_second_encounter_does_not_refire` — Verifies non-repeatable triggers don't re-fire
  - Regression tests: FreeAction without/with movement
  - Helper functions: `send_action()`, `get_status()`, `count_log_entries()`, `wait_for_log_entries()`
- `test_utils.rs` — Shared test infrastructure:
  - `TestServer` — RAII server wrapper (starts on `new()`, kills on `Drop`)
  - `get_available_port()` — Dynamic port allocation with exponential backoff and lock files
  - `wait_for_server()` — Polling loop for server readiness
  - `wait_for_llm_idle()` — Polls `/status/generating` endpoint until LLM finishes
  - `wait_for_story_log_entries()` / `wait_for_location_change()` / `wait_for_story_log_change()` — Browser polling helpers
  - `wait_for_more_messages()` / `wait_for_non_loading_value()` / `wait_for_element_class()` / `wait_for_element_children()` / `wait_for_element_text()` — Additional polling utilities
  - `wait_for_status_ready()` / `wait_for_status_not_thinking()` — Status checking functions
  - `goto_with_connection_check()` — Navigation with explicit connection error handling
  - `TestConfig` — JSON-based test configuration (port ranges, per-test backend selection)
  - `launch_chrome()` — Playwright browser launch
  - Port locking mechanism with PID tracking for stale lock cleanup

**Patterns:**
- Server-per-test isolation with dynamic port allocation and lock files
- RAII cleanup via `TestServer::Drop`
- Browser polling with timeout (20 iterations × 500ms = 10s typical)
- Mock vs real LLM backend selection per test via `TestConfig`
- Shared browser pattern in flow_mock_tests for efficiency

## Flow

1. Test starts → `TestServer::from_config()` or `TestServer::new()` → spawns `cargo run`
2. `wait_for_server()` polls port until ready
3. Playwright launches Chrome → navigates to `http://127.0.0.1:{port}`
4. Test interacts with page (click, type, submit)
5. Polling helpers wait for DOM changes (story log, status, location)
6. `TestServer::Drop` kills process, waits for port release

## Integration

- **Tests**: `src/engine/`, `src/model/`, `src/narrative/`, `src/server/`
- **External**: Playwright (browser automation), reqwest (HTTP polling)
- **Config**: `test_config.json` (port ranges, backend selection)