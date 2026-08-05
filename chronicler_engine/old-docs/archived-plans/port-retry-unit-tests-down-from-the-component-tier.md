# Port retry unit tests down from the component tier

## Summary

Ticket 04 (wayfinder map `test-strategy-execution`). Port retry branch-coverage assertions from `tests/integration/application/action_pipeline/retry.rs` (component tier) into `src/application/pipeline/retry_tests.rs` (unit). Add shutdown-guard unit + handler tests for `retry()` / `retrigger()` (ticket 11 branches). Fix mislabeled `test_pipeline_trigger_complete_failure` backend to match S2.4. Delete the component-tier retry file + unwire its mod. Suite stays green, guardrails pass.

Scope: `action_pipeline/retry.rs` only. `flow/retry_main.rs` + `flow/retry_event.rs` belong to ticket 05 — do not touch.

## Key Changes

- `src/application/pipeline/retry_tests.rs` — add 4 port-down tests + 2 shutdown-guard unit tests; strengthen existing `test_retry_no_input` + `test_retry_main_narration_happy_path`.
- `src/application/pipeline/pipeline_tests.rs` — fix `test_pipeline_trigger_complete_failure`: swap `with_fail()` → `with_trigger_narration_fail()`, add main-narration-preserved + System-log assertions to match S2.4.
- `src/adapters/driving/http/chat_window/handlers/chat_window_tests.rs` — replace loose `test_retry_handler` / `test_retrigger_handler` "any 2xx/4xx/5xx" assertions with specific `test_retry_handler_returns_503_on_shutdown` + `test_retrigger_handler_returns_503_on_shutdown` (mirror of `actions_tests.rs::test_action_handler_returns_503_on_shutdown`).
- `tests/integration/application/action_pipeline/retry.rs` — delete.
- `tests/integration/mod.rs` — remove `#[path = "application/action_pipeline/retry.rs"] mod pipeline_retry;` lines.

## Implementation

### Phase 1: Port unit tests + fix mislabeled test

- [ ] #### Task 1.1: Port 4 missing branch-coverage tests into retry_tests.rs (3 SP)
  - [ ] ##### SubTask 1.1.1: `test_retry_recovers_after_llm_failure` — seed input, backend `with_fail_first_n(1)`, call `execute_action` (fail). **Risk: `execute_action` spawns a task; retry must observe the first task's completion before calling `retry_last_response` (else busy gate → `ConcurrentGeneration`).** Before writing, verify the existing no-wait pattern in `test_retry_event_trigger_narration_fails` is reliable (it asserts immediately after `retry_last_response()` with no wait). If the mock is fast enough to yield before `load_or_fresh`, no sync needed; else reuse `MockBackend.narration_started`/`trigger_started` AtomicBool sync primitives or add a short yield loop. After wait, call `retry_last_response`; assert status reaches non-generating + narration count 1. (1 SP)
  - [ ] ##### SubTask 1.1.2: `test_retry_room_not_found_sets_error` — seed input, move `movement.current_room_id` to `"non_existent_room"`, `retry_last_response`; assert `GenerationStatus::Error` containing `"Room not found"`. (1 SP)
  - [ ] ##### SubTask 1.1.3: `test_retry_main_narration_llm_error_sets_error` — backend `with_fail()`, seed input + pre-main snapshot + narration, `retry_last_response`; assert `Error` (main-retry failure path, distinct from existing trigger-fail test). (0.5 SP)
  - [ ] ##### SubTask 1.1.4: `test_retry_main_narration_empty_response_sets_error` — backend `with_empty_response()`, same setup as 1.1.3; assert `Error` msg contains `"empty"`. (0.5 SP)
- [ ] #### Task 1.2: Strengthen existing retry tests (2 SP)
  - [ ] ##### SubTask 1.2.1: Extend `test_retry_main_narration_happy_path` — `retry_main_narration` is sync (`pub(crate) fn`, not async); no `wait_for_generation_complete` / sync flag needed. After the sync call returns, load state, assert final status `Idle` + exactly 1 narration message (pre-main snapshot used, count stays 1). (1 SP)
  - [ ] ##### SubTask 1.2.2: Strengthen `test_retry_no_input` in-place (no new test) — seed System + Narration messages (no Input), `retry_last_response`, assert `history.len()` unchanged (noop). Avoids same-tier overlap with `test_retry_appends_swipe_to_same_message` (which already asserts `narration.id == original_id` + 3 swipes → implies count stays 1). (1 SP)
- [ ] #### Task 1.3: Fix mislabeled `test_pipeline_trigger_complete_failure` in pipeline_tests.rs (1 SP)
  - [ ] ##### SubTask 1.3.1: Swap `MockBackend::default().with_fail()` → `with_trigger_narration_fail()`; add assertions: (a) at least one `Narration` message preserved (main succeeded), (b) at least one `System` message whose text mentions `"Trigger narration failed"`, (c) status `Error` mentioning `"Trigger narration failed"`. Matches actions.md S2.4. (1 SP)

### Phase 2: Shutdown-guard coverage + handler tests + delete component tier

- [ ] #### Task 2.1: Add shutdown-guard unit tests for retry() + retrigger() (1 SP)
  - [ ] ##### SubTask 2.1.1: `test_retry_returns_shutting_down_when_token_cancelled` — `TestAppBuilder::default_test().build_service()`, `state.shutdown_token.cancel()`, call `app.pipeline.retry(&app.generation_gate)`; assert `Ok(ProcessActionResult::ShuttingDown)`. No seeding needed (guard runs before state load). (0.5 SP)
  - [ ] ##### SubTask 2.1.2: `test_retrigger_returns_shutting_down_when_token_cancelled` — same shape for `retrigger(&app.generation_gate)`. (0.5 SP)
- [ ] #### Task 2.2: Replace loose handler tests with shutdown-path 503 tests (1 SP)
  - [ ] ##### SubTask 2.2.1: In `chat_window_tests.rs`, add `test_retry_handler_returns_503_on_shutdown` + `test_retrigger_handler_returns_503_on_shutdown` — cancel `state.shutdown_token`, call handler, assert `StatusCode::SERVICE_UNAVAILABLE` via `IntoResponse` on the `Err(ApplicationError::ShuttingDown)` arm. Mirror `actions_tests.rs::test_action_handler_returns_503_on_shutdown`. (0.5 SP)
  - [ ] ##### SubTask 2.2.2: Delete the loose `test_retry_handler` + `test_retrigger_handler` (the "any 2xx/4xx/5xx" assertions). Keep `test_switch_swipe_handler` (out of scope). (0.5 SP)
- [ ] #### Task 2.3: Delete component-tier retry.rs + unwire mod (1 SP)
  - [ ] ##### SubTask 2.3.1: Delete `tests/integration/application/action_pipeline/retry.rs` and its parent `action_pipeline/` dir if empty. (0.5 SP)
  - [ ] ##### SubTask 2.3.2: Remove the two `#[path = "application/action_pipeline/retry.rs"]\nmod pipeline_retry;` lines from `tests/integration/mod.rs`. (0.5 SP)

## Test Plan

- `cargo nextest run -p chronicler_engine` (or `cargo test -p chronicler_engine`) — full suite green. Baseline per map: 1362 pass after ticket 11. Expect +6 unit tests (+2 strengthened), -9 component-tier tests, net ~1359; exact count verified at run.
- `python tests/infrastructure/guardrails/validate_feature_spec.py` (or repo-equivalent) — 17 declared / 17 covered / 0 gaps / 0 orphans, no SCENARIO-tag placement violations (no new tags added in `src/`).
- Targeted: `cargo nextest run -p chronicler_engine src::application::pipeline::retry_tests` + `chat_window_tests` + `pipeline_tests` — all pass.

## Per Task/Sub Task Validation Steps

- SubTask 1.1.x: new test compiles, passes in isolation, fails if the branch it targets is reverted (e.g. flip `with_fail_first_n` to `with_fail` to confirm the test catches the wrong path).
- SubTask 1.1.1 specifically: if the wait-risk is real, test will flake without sync; verify before committing.
- SubTask 1.2.x: assertion fails when invariant violated (e.g. noop test: if retry appends instead of noop, `history.len()` changes → test fails).
- SubTask 1.3.1: test fails with the old `with_fail()` backend (main narration lost) — proves the mislabel was real. Passes with `with_trigger_narration_fail()`.
- SubTask 2.1.x: removing the top-of-fn `is_shutting_down()` guard from `pipeline.rs` makes the test fail (would proceed past the guard). Do NOT commit that removal — revert after confirming.
- SubTask 2.2.x: handler tests assert exactly `503`, not a range.
- SubTask 2.3.x: build fails if the `mod pipeline_retry;` line is left in place without the file; green after removal.

## Assumptions

- Ticket 01's code changes (R4.3/R4.4 pre-spawn anchor check, R5.3/R5.4 generation gate on retry/retrigger, R4.8 system-log persistence) are landed and covered by existing tests `test_retry_returns_internal_error_when_anchor_has_no_snapshot_id`, `test_retry_returns_internal_error_when_snapshot_row_missing`, `test_retry_returns_concurrent_generation_when_gate_busy`, `test_retrigger_returns_concurrent_generation_when_gate_busy`, `test_retry_event_trigger_narration_fails`. No new tests needed for these — confirmed by grep.
- `test_retry_no_snapshot` drift (item 5 in the diff asset) is resolved by deleting the weaker SQLite version; the in-memory version already asserts `Error` containing "Retry failed: no anchor message" — no new test needed.
- No new spec scenarios added in this ticket (spec updates belong to ticket 05). Branch-coverage unit tests don't carry SCENARIO tags per STRATEGY.md.
- `TestAppBuilder::default_test().build_service_with_storage()` and `make_test_recorder` / `make_test_pipeline_with_mock_quantifier` helpers already used in retry_tests.rs are reused; no new test-support code.
- Branch is `testing-refactor`; work continues on this branch (no branch switch).
- Ponytail lite: lazier alternative would be to skip the 4 port-down tests and rely on the HTTP E2E tests (ticket 05) to cover retry behaviour end-to-end — but ticket 04's acceptance explicitly requires every retry branch to have a unit test, and the diff asset classifies these as "port down" not "move to HTTP E2E", so they stay.

## NOT in scope

- `flow/retry_main.rs` + `flow/retry_event.rs` (ticket 05).
- `docs/specs/retry.md` scenario edits (ticket 05).
- New spec scenarios (unit tests carry no SCENARIO tags).
- Retry storage-error branches already covered by existing tests (`test_retry_event_storage_error_on_pre_event`, `test_retry_main_storage_error_on_pre_main`, `test_retry_event_continuation_returns_ok_on_world_fetch_failure`, `test_retry_event_continuation_returns_ok_on_persona_fetch_failure`, `retry_records_canonical_game_not_found_when_game_missing`) — no new tests.

## What already exists (reuse)

- All retry branch-coverage tests for: no-snapshot, no-input, missing-trigger-context, trigger-narration-fail, event-empty-continuation, main-no-pre-main-snapshot, happy paths, storage errors, cancellation at phase boundary, concurrent-generation gate, anchor-no-snapshot-id, snapshot-row-missing. ~25 tests in `retry_tests.rs`.
- Handler 503-on-shutdown pattern in `actions_tests.rs::test_action_handler_returns_503_on_shutdown` — mirror it.
- `MockBackend` sync primitives (`narration_started`, `trigger_started` AtomicBool).

## Failure modes

- Test doesn't catch bug → validation step (flip backend variant, confirm test fails) per subtask.
- Build break on dangling `mod pipeline_retry;` → fix is the unwire (Task 2.3).
- Race in 1.1.1 → Finding 4 risk; mitigation = reuse `narration_started`/`trigger_started` AtomicBool or short yield loop.
- Mislabel fix (1.3.1) undetected → test fails with old backend proves it; passes with new.

## Unresolved decisions

- None remaining. Findings 1+2 applied (dropped redundant test, folded noop into existing). Finding 3 wording fixed. Finding 4 risk flagged in-task, verified at implementation time.
