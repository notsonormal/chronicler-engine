# Changelog

## 2026-05-31

### Fixed

- **Fixed scenario messages losing content on game reset/new game**
  - Root cause: `create_game()` and `reset()` in `src/application/game_lifecycle.rs` were inserting messages but not their swipes
  - Messages appeared in database with no text content (swipe records missing)
  - Added `insert_swipe()` calls matching the pattern in `bootstrap/run.rs` lines 254-256
  - Also set `swipe.snapshot_id` before insertion to maintain consistency
  - Verified: Fresh game creation now properly persists scenario introduction with text and location header
  - All existing tests pass; no behavioral changes except fixing the message persistence bug

### Changed

- **Migrated from log/env_logger to tracing crate**
  - Replaced all `log::info!`, `log::debug!`, `log::warn!`, `log::error!` calls with `tracing::info!`, `tracing::debug!`, `tracing::warn!`, `tracing::error!`
  - Updated logging initialization in `bootstrap/logging.rs` to use `tracing_subscriber` with `tracing-appender`
  - File logging with daily rotation to `logs/chronicler_YYYYMMDD.log`
  - Non-blocking writer with proper guard management for application lifetime
  - Removed `log = "0.4"` and `env_logger = "0.11"` dependencies from Cargo.toml
  - Added `tracing-appender = "0.2"` dependency
  - All 944 tests pass; clippy clean; full validation passes
  - Updated 22 source files across application, bootstrap, server, narrative, model, engine, and storage tiers

- **Extracted `build_request_payload`, `configure_request`, and `handle_response` from `call_chat_completions`**
  - Refactored 160-line god-function into composable pure functions
  - `build_request_payload()` — Pure JSON construction (no side effects)
  - `configure_request()` — Pure RequestBuilder construction with conditional headers
  - `handle_response()` — Focused response parsing, delegates to `parse_chat_response()`
  - `call_chat_completions()` reduced to ≤30 lines of clear happy-path orchestration
  - Added 3 unit tests for `build_request_payload()` (empty system prompt, non-empty system prompt, max_tokens serialization)
  - All 947 tests pass; clippy clean; build.py passes
  - Updated docs/architecture/system.md and docs/system/llm_processing.md to reflect modular structure

- **Removed thin abstraction `TriggerContinuationRequest`**
  - Deleted identity wrapper struct around `StoredTriggerContext` that added zero semantic value (4 lines in `src/engine/action_processing.rs`)
  - Updated `commit_trigger_narration()` to accept `&StoredTriggerContext` directly instead of wrapper
  - Updated `phase_trigger_continuation()` and `build_trigger_request()` signatures to work with `StoredTriggerContext` directly
  - Removed wrapper construction at 10 call sites (4 production, 6 tests)
  - Saves developers from learning `.stored` accessor pattern for zero benefit
  - All 947 tests pass; clippy clean; build.py passes
- **Split `llm_client.rs` into modular directory structure**
  - Refactored 314-line single file into directory module with clear separation of concerns
  - Created `src/narrative/llm_client/` with `mod.rs`, `request.rs`, `response.rs`, `client.rs`
  - `request.rs` (72 lines): `REQUEST_COUNTER`, `next_request_id()`, `ChatCompletionResult`, `build_request_payload()`, `configure_request()`
  - `response.rs` (166 lines): `extract_content_from_response()`, `parse_chat_response()`, `handle_response()`
  - `client.rs` (114 lines): `call_chat_completions()`, `call_openrouter_with_model()`, `call_ollama()`
  - Split tests into `tests/request_tests.rs` (45 lines, 3 tests), `tests/response_tests.rs` (140 lines, 10 tests), `tests/integration_tests.rs` (244 lines, 20 tests)
  - Maintained 100% backward compatibility — all external callers unchanged
  - All 947 tests pass; clippy clean; `python build.py` passes
  - Updated docs/system/llm_processing.md to reflect new module structure
- **Refactored `handle_movement` to split mixed responsibilities**
  - Extracted `attempt_movement()` — handles semantic walk + dynamic room creation on failure
  - Extracted `update_npc_encounters_on_room_change()` — pure function for NPC meeting state updates
  - Extracted `log_movement_completion()` — pure function for narrative pending location
  - Refactored `handle_movement()` to compose helpers in linear flow (attempt → update NPCs → log completion)
  - Each helper has single responsibility, testable in isolation
  - No behavioral changes — all 947 tests pass; clippy clean; build.py passes
  - Updated docs/architecture/system.md to reflect new function structure
## 2026-05-30

