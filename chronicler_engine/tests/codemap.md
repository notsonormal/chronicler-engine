# chronicler_engine/tests/

## Responsibility

Integration tests for the Chronicler Engine, covering component logic, end-to-end browser flows, LLM interactions, trigger evaluation, and navigation. Uses `playwright_rs` for browser automation and a `TestServer` harness for server lifecycle management.

## Design

**Test organization:**
- `component_tests.rs` — Unit-level tests for engine parsing, model serialization, trigger evaluation, and state management
- `e2e_tests.rs` — Full browser-based tests: server startup, page load, UI element presence, basic interactions
- `flow_llm_tests.rs` — Real LLM API call tests through the full game loop
- `flow_mock_tests.rs` — Mock LLM backend tests for the game loop (fast, no API key needed)
- `trigger_tests.rs` — Trigger evaluation, condition checking, and continuation narration
- `test_utils.rs` — Shared test infrastructure:
  - `TestServer` — RAII server wrapper (starts on `new()`, kills on `Drop`)
  - `get_available_port()` — Dynamic port allocation with exponential backoff
  - `wait_for_server()` — Polling loop for server readiness
  - `wait_for_llm_idle()` — Polls `/status/generating` endpoint until LLM finishes
  - `wait_for_story_log_entries()` / `wait_for_location_change()` / `wait_for_status_ready()` — Browser polling helpers
  - `goto_with_connection_check()` — Navigation with explicit connection error handling
  - `TestConfig` — JSON-based test configuration (port ranges, per-test backend selection)
  - `launch_chrome()` — Playwright browser launch
- `test_data.rs` — Factory functions for test fixtures:
  - `create_test_world()` / `create_test_player()` / `create_test_npcs()` / `create_test_map()` — Standard 3-room world
  - `create_test_game_state()` — `Arc<Mutex<GameState>>` ready for server tests
  - `create_navigation_test_map()` — 4-room mansion layout
  - `create_simple_test_map()` — Single room for basic UI tests

**Patterns:**
- Server-per-test isolation with dynamic port allocation
- RAII cleanup via `TestServer::Drop`
- Browser polling with timeout (20 iterations × 500ms = 10s typical)
- Mock vs real LLM backend selection per test via `TestConfig`

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
