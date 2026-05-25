# Plan: Fix Test Coverage Gaps (Test-Police Follow-up)

## Goal
Raise coverage on the three most impactful undertested files to push overall coverage above 85% and eliminate all sub-80% files that are not explicitly expected (bootstrap, external LLM backends).

## Files to Target

### 1. `src/server/fragments/misc.rs` — 62.7% → target 75%+
Currently missing tests for:
- `retrigger_handler` — all error branches (no trigger, no messages, last message not narration, generation in progress)
- `switch_swipe_handler` — error branches (generation in progress, not last message, missing snapshot)
- `reset_handler` — some error branches (delete_game failure, create_game failure, save snapshot failure)

**Approach:** Add component tests in `tests/components/misc.rs` using the existing `create_app_with_storage` pattern and `FailingLoadStorage`/`FailingSaveStorage` wrappers already used in the file.

### 2. `src/application/action_pipeline/retry.rs` — 76.5% → target 80%+
Currently missing tests for:
- `post_retry_swipe_migration` success path with actual pending swipes (existing tests hit error paths or empty swipes)
- `retrigger_event_impl` — the `Cancelled` branch (lines 220–225)
- `retry_main_narration` — not directly exercised (only via `retry_last_response_impl`)

**Approach:** Add unit tests in `src/application/action_pipeline/retry_tests.rs` using existing `make_test_context_with_sqlite` and `FailingMessageStorage` infrastructure.

### 3. `src/application/action_pipeline/pipeline.rs` — 78.9% → target 80%+
Currently missing tests for:
- `run_trigger_continuation` — the cancellation-at-start branch (lines 387–395)
- `phase_trigger_continuation` — `save_message_and_snapshot` error path (lines 277–282)
- `reconcile_post_trigger_npcs` — not directly tested (only via integration)

**Approach:** Add unit tests in `src/application/action_pipeline/pipeline_tests.rs` using the existing `MockPipelineBackend` and cancellation token.

## Verification Steps

1. Run tests: `cargo nextest run --no-fail-fast`
2. Run coverage: `cargo llvm-cov nextest --no-report`
3. Parse report: `python scripts/parse_coverage.py --json target/llvm-cov/coverage.json`
4. Confirm all three files are at or above 80% and total coverage increased.

## Risks & Notes

- Component tests for `retrigger_handler` require setting up a state with `last_trigger` present — use `StoredTriggerContext` directly.
- `switch_swipe_handler` tests need messages with multiple swipes and snapshot IDs already wired up.
- The `retrigger_event_impl` cancelled branch requires cancelling the token before calling the function.
- No changes to production code — this is test-only work.
