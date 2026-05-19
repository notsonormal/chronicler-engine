# Plan: Split `action_pipeline.rs` Integration Tests

## Context
`chronicler_engine/tests/action_pipeline.rs` is a single 1072-line integration test file. The source module `chronicler_engine/src/application/action_pipeline/` is already split into:
- `actions.rs` — `execute_action_impl` entry point
- `pipeline.rs` — `ActionPipeline` struct and phase logic
- `retry.rs` — `retry_last_response_impl` entry point

The project already uses the `tests/foo.rs` + `tests/foo/` subdirectory pattern (see `browser.rs`, `components.rs`, `diagnostic.rs`, etc.).

## Goal
Split `tests/action_pipeline.rs` into multiple files under `tests/action_pipeline/`, mirroring the source module structure.

## Guiding Principle
The inline unit tests (`actions_tests.rs`, `pipeline_tests.rs`, `retry_tests.rs`) already map 1:1 to the source files. The integration tests should follow the same structure. Since all integration tests exercise public entry points (`execute_action_impl` or `retry_last_response_impl`), the split is based on **what is being tested**:

- `actions.rs` → basic `execute_action_impl` contract (simple backends)
- `pipeline.rs` → advanced `ActionPipeline` phase behaviour (quantifier, triggers, cancellation, snapshots)
- `retry.rs` → `retry_last_response_impl` behaviour

## Proposed Split

### 1. `tests/action_pipeline/actions.rs` — Basic execution tests
Tests that exercise `execute_action_impl` with standard `working_backend()` / `failing_backend()`:
- `test_pipeline_executes_and_persists_narration`
- `test_pipeline_persists_input_before_narration`
- `test_pipeline_handles_room_not_found`
- `test_pipeline_handles_llm_failure`
- `test_pipeline_clears_last_trigger`
- `test_pipeline_phase_transitions`
- `test_pipeline_phase_stays_narrating_on_error`
- `test_pipeline_empty_input`

### 2. `tests/action_pipeline/pipeline.rs` — Advanced pipeline behaviour tests
Tests that require `with_mock_quantifier()` or test specific pipeline phases (cancellation, triggers, movement, snapshots):
- `test_delayed_llm_completes_without_deadlock`
- `test_quantifier_detects_movement`
- `test_quantifier_detects_npc_presence_and_fires_trigger`
- `test_empty_llm_response_handled_gracefully`
- `test_failing_trigger_narration_does_not_crash`
- `test_pipeline_cancels_when_token_cancelled`
- `test_cancellation_resets_state_to_idle`
- `test_pipeline_cancels_after_main_narration`
- `test_pipeline_cancels_during_trigger_continuation`
- `test_pre_main_snapshot_saved_before_narration`
- `test_pre_event_snapshot_saved_before_continuation`
- `test_pipeline_with_quantifier`

### 3. `tests/action_pipeline/retry.rs` — Retry tests
Tests that exercise `retry_last_response_impl`:
- `test_retry_finds_last_input_and_runs_pipeline`
- `test_retry_with_empty_history_is_noop`
- `test_retry_after_llm_failure_succeeds`
- `test_retry_no_snapshot`
- `test_retry_no_input_text`
- `test_retry_room_not_found`
- `test_retry_llm_error`
- `test_retry_empty_narration`
- `test_retry_main_narration_uses_pre_main_snapshot`
- `test_retry_event_continuation_uses_pre_event_snapshot`

## Why this split is consistent
- **All cancellation tests are together** in `pipeline.rs` — they test cancellation at different pipeline checkpoints.
- **All quantifier/trigger tests are together** in `pipeline.rs` — they need `with_mock_quantifier()` and test specific pipeline phases.
- **All snapshot tests are together** in `pipeline.rs` — they verify snapshot persistence during pipeline execution.
- **`actions.rs` stays focused** on the basic entry-point contract with simple backends.
- **`retry.rs` is unchanged** in scope — all retry scenarios in one place.

## Implementation Steps

1. **Create directory** `chronicler_engine/tests/action_pipeline/`
2. **Refactor `tests/action_pipeline.rs`** into a module aggregator:
   - Keep the `//! [DOC: docs/reference/testing.md]` doc comment
   - Keep shared imports (`std::sync::Arc`, `chronicler_engine::*` re-exports)
   - Keep shared helper functions: `working_backend()`, `failing_backend()`
   - Keep the `pipeline_helpers` and `test_data` module declarations
   - Add `#[path = "action_pipeline/actions.rs"]` etc. submodule declarations
3. **Create `tests/action_pipeline/actions.rs`** with the basic execution tests and their required imports
4. **Create `tests/action_pipeline/pipeline.rs`** with the advanced pipeline tests and their required imports
5. **Create `tests/action_pipeline/retry.rs`** with the retry tests and their required imports
6. **Run `cargo test --test action_pipeline`** to verify all tests still pass

## Shared Code Strategy
The existing pattern in the codebase (`tests/components.rs`, `tests/browser.rs`) places shared helpers in the main `.rs` file. Submodules access them via `super::`. The main file will keep:
- `working_backend()` / `failing_backend()`
- `pipeline_helpers` module declaration
- `test_data` module declaration

Each submodule will import what it needs from `super`.

## Verification
- All existing tests run and pass
- No test names or assertions are changed (pure file move)
- `cargo test --test action_pipeline` passes
