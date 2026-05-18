# Plan: Consolidate Action Pipeline Tests

## Overview

After the refactor split `game_service` logic into `action_pipeline`, the `tests/game_service/` integration tests have significant overlap with the new `tests/action_pipeline.rs`. This plan moves pipeline-behavior tests to `action_pipeline`, keeps service-boundary tests in `game_service`, and fills gaps.

## Current State

| Test file | Count | What it tests |
|-----------|-------|---------------|
| `src/application/action_pipeline/pipeline_tests.rs` | 8 | Unit tests for `ActionPipeline` orchestration |
| `src/application/action_pipeline/retry_tests.rs` | 18+ | Unit tests for retry logic |
| `src/application/action_pipeline/actions_tests.rs` | 5 | Unit tests for `execute_action_impl` entry point |
| `tests/action_pipeline.rs` | 9 | Integration tests for pipeline functions directly |
| `tests/game_service/basic.rs` | 9 | Integration tests through `GameService` trait |
| `tests/game_service/advanced.rs` | 27 | Integration tests through `GameService` trait |
| `tests/flow_mock/*.rs` | 22 | Workflow/sequence tests through `GameService` trait |

## Goal

- `tests/action_pipeline.rs` owns all pipeline-behavior integration tests (narration, errors, cancellation, quantifier, trigger, retry)
- `tests/game_service/` owns only service-boundary tests (constructors, trait delegation, edge inputs)
- No redundant tests across suites
- All gaps filled

---

## Phase 1: Expand `tests/action_pipeline.rs`

### Task 1.1: Rename shared helpers and add to action_pipeline

The helper file `tests/helpers/game_service.rs` is already shared across `game_service` and `flow_mock` test crates via `#[path]`. Since it will now also be used by `action_pipeline` tests, rename it to something neutral.

**Changes:**
- Rename `tests/helpers/game_service.rs` → `tests/helpers/pipeline_helpers.rs`
- Update declarations:
  - `tests/game_service.rs`: `#[path = "helpers/pipeline_helpers.rs"] mod pipeline_helpers;`
  - `tests/flow_mock.rs`: `#[path = "helpers/pipeline_helpers.rs"] mod pipeline_helpers;`
  - `tests/action_pipeline.rs`: add `#[path = "helpers/pipeline_helpers.rs"] mod pipeline_helpers;`
- Update `use` statements in all affected files (`basic.rs`, `advanced.rs`, `sequence.rs`, `retry_main.rs`, `retry_event.rs`, `action_pipeline.rs`) from `crate::game_service_helpers` to `crate::pipeline_helpers`
- Remove the local `latest_state` helper from `tests/action_pipeline.rs` (use `pipeline_helpers::latest_state` instead)

### Task 1.2: Move pipeline behavior tests from `game_service/advanced.rs`

Move these 18 tests to `tests/action_pipeline.rs`, adapting them to call `execute_action_impl` and `retry_last_response_impl` directly instead of going through `service.execute_action()` / `service.retry_last_response()`.

| Test | What it tests |
|------|---------------|
| `test_execute_freeaction_with_movement_mock` | Quantifier + movement through pipeline |
| `test_cancellation_resets_state_to_idle` | Async cancellation resets status |
| `test_pipeline_cancels_after_main_narration` | Cancellation at post-narration checkpoint |
| `test_pipeline_cancels_during_trigger_continuation` | Cancellation during trigger LLM call |
| `test_empty_llm_response_handled_gracefully` | Empty narration text → error |
| `test_failing_trigger_narration_does_not_crash` | Trigger narration failure recovery |
| `test_delayed_llm_completes_without_deadlock` | Delayed LLM + synchronous execution |
| `test_quantifier_detects_movement` | Quantifier returns movement JSON |
| `test_quantifier_detects_npc_presence_and_fires_trigger` | Trigger fires end-to-end |
| `test_retry_no_snapshot` | Retry with no snapshot is no-op |
| `test_retry_no_input_text` | Retry with no input text is no-op |
| `test_retry_room_not_found` | Retry when room not found |
| `test_retry_llm_error` | Retry with failing LLM |
| `test_retry_empty_narration` | Retry with empty LLM response |
| `test_pre_main_snapshot_saved_before_narration` | Snapshot exists after execution |
| `test_pre_event_snapshot_saved_before_continuation` | Snapshot exists after trigger |
| `test_retry_main_narration_uses_pre_main_snapshot` | Retry restores pre-main snapshot |
| `test_retry_event_continuation_uses_pre_event_snapshot` | Retry restores pre-event snapshot |

### Task 1.3: Add missing pipeline tests

Add these new tests to `tests/action_pipeline.rs` to fill gaps left by removing redundant `game_service` tests:

1. `test_pipeline_phase_transitions` — verify `Narrating` → `default()` on success, `Narrating` stuck on error
2. `test_pipeline_empty_input` — verify empty string input is handled (no panic, appropriate status)
3. `test_pipeline_with_quantifier` — run with `DefaultGameService::with_mock_quantifier` (currently all action_pipeline tests use `AgentRegistry::default()` which has no quantifier)

---

## Phase 2: Clean up `tests/game_service/`

### Task 2.1: Remove redundant tests from `basic.rs`

Delete these 4 tests (pipeline behavior now covered in `action_pipeline.rs`):

- `test_execute_look_action`
- `test_execute_inventory_action`
- `test_retry_with_no_history`
- `test_execute_look_room_not_found`

### Task 2.2: Remove moved tests from `advanced.rs`

Delete the 18 tests listed in Task 1.2.

### Task 2.3: Remove additional redundant tests from `advanced.rs`

These 9 tests are also pure pipeline behavior and are covered by existing `action_pipeline` or unit tests:

- `test_execute_freeaction_immediate_return` — synchronous return is trivial
- `test_execute_freeaction_room_not_found` — covered by `test_pipeline_handles_room_not_found`
- `test_execute_freeaction_state_accessible` — synchronous execution, covered
- `test_execute_freeaction_narration_failure` — covered by `test_pipeline_handles_llm_failure`
- `test_execute_freeaction_with_mock_backend` — covered by `test_pipeline_executes_and_persists_narration`
- `test_retry_with_mock_backend` — covered by `test_retry_finds_last_input_and_runs_pipeline`
- `test_freeaction_phase_starts_narrating` — covered by new `test_pipeline_phase_transitions`
- `test_freeaction_phase_transitions_mock` — covered by new `test_pipeline_phase_transitions`
- `test_retry_last_response_not_ai_generated` — covered by `test_retry_after_llm_failure_succeeds`

### Task 2.4: Collapse `game_service/` into single file

With only 5 tests remaining, the `game_service/` subfolder is unnecessary. Inline the remaining tests directly into `tests/game_service.rs` and delete the subfolder.

**Tests to inline into `tests/game_service.rs`:**
- `test_execute_action_empty_command`
- `test_execute_action_unknown_command`
- `test_default_game_service_default`
- `test_default_game_service_with_backends`
- `test_default_game_service_with_mock_quantifier`

**Changes:**
- Move test bodies + `run_action` helper from `tests/game_service/basic.rs` into `tests/game_service.rs`
- Delete `tests/game_service/basic.rs`
- Delete `tests/game_service/advanced.rs` (already empty from Task 2.2 + 2.3)
- Remove `mod basic;` and `mod advanced;` declarations from `tests/game_service.rs`
- `tests/game_service.rs` now declares: `mod test_data;`, `mod pipeline_helpers;`, plus the 5 tests and helpers

---

## Phase 3: Verify

### Task 3.1: Run full test suite

```bash
cd chronicler_engine && cargo nextest run
```

**Acceptance criteria:**
- All tests pass
- No dead code warnings from removed tests
- `game_service.rs` test count: 5 (down from 36)
- `action_pipeline.rs` test count: ~30 (up from 9)

---

## Files touched

| File | Action |
|------|--------|
| `tests/action_pipeline.rs` | Expand with moved tests + new tests |
| `tests/game_service.rs` | Inline remaining 5 tests, remove submodule declarations |
| `tests/game_service/basic.rs` | **Delete** (tests inlined into parent) |
| `tests/game_service/advanced.rs` | **Delete** (all tests moved or removed) |
| `tests/helpers/game_service.rs` | **Rename** to `tests/helpers/pipeline_helpers.rs` |
| `tests/flow_mock.rs` | Update `mod` declaration to `pipeline_helpers` |
| `tests/flow_mock/*.rs` | Update `use` statements to `crate::pipeline_helpers` |

## Risks

| Risk | Mitigation |
|------|------------|
| Moving async `#[tokio::test]` tests to `action_pipeline.rs` | `tests/action_pipeline.rs` already compiles as integration test crate; tokio is available |
| `pipeline_helpers` shared across test crates | Each integration test crate is independent; `#[path]` include is safe |
| Lost coverage during transition | Move first, delete second; run tests between steps |

## Checkpoint

After Phase 1 + Phase 2 complete:
- [ ] `cargo test --test action_pipeline` passes
- [ ] `cargo test --test game_service` passes
- [ ] `cargo nextest run` passes with expected test counts
