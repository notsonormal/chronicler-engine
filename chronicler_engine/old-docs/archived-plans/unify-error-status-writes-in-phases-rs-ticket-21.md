# Unify error-status writes in `phases.rs` (ticket 21)

## Summary
Refactor `chronicler_engine/src/application/pipeline/phases.rs` so every phase-level failure path sets `GenerationStatus::Error` and returns `PhaseError`, letting the orchestrator route it through `finalize_phase_error` / `persist_generation_error`. Update callers in `pipeline.rs`, add one focused test, and document the one deliberate persistence swallow.

## Key Changes
- Add a shared `set_error(&mut state, msg)` helper on `PipelineRun` that writes `GenerationStatus::Error`, persists, and returns `PhaseError::NarratorFailed`. Refactor the existing `error_return` to use it.
- `phase_narrate`: replace the pre-quantifier `warn!`-and-continue `save_message_and_snapshot` failure with `persist_snapshot_or_err`.
- `phase_post_generation`: return `Result<QuantifierResult, PhaseError>`; use `persist_snapshot_or_err` for the pre-quantifier phase-update save; move the post-quantifier metadata save from `run_from_input` into this method as a documented best-effort warning.
- `phase_trigger_continuation_llm_call`: return `Err` for LLM failure, empty response, and `commit_trigger_narration` failure via `set_error`.
- `reconcile_post_trigger_npcs`: return `Result<GameState, PhaseError>` and propagate `apply_npc_events` failures as `PhaseError::NarratorFailed`.
- `pipeline.rs` callers: match the new `Result` return types and route errors through `finalize_phase_error` / `handle_retry_outcome`.
- Tighten `test_trigger_continuation_save_post_trigger_error` to expect only `Err(PhaseError::PersistFailed { label: "pre-event snapshot", .. })`.

## Implementation

### Phase 1: Refactor phase errors

- [ ] #### Task 1.1: Claim and execute the refactor (5 SP)
  - Claim ticket 21 by updating `.scratch/arch-exec-wiredapp-pipeline/issues/21-unify-error-status-writes-phases.md` to `Status: in-progress`, `Assignee: pi`.
  - Dispatch to a `general-purpose` subagent:
    - In `chronicler_engine/src/application/pipeline/phases.rs`:
      - Add `set_error(&mut state, msg)` and refactor `error_return` to use it.
      - Convert `phase_narrate` pre-quantifier save to `persist_snapshot_or_err`.
      - Convert `phase_post_generation` to `Result<QuantifierResult, PhaseError>`; use `persist_snapshot_or_err` for the pre-quantifier save; move the post-quantifier metadata save from `run_from_input` into this method with a comment explaining why it is best-effort.
      - Convert `phase_trigger_continuation_llm_call` error branches to `Err` via `set_error`.
      - Convert `reconcile_post_trigger_npcs` to `Result<GameState, PhaseError>`.
    - In `chronicler_engine/src/application/pipeline/pipeline.rs`:
      - Adjust `run_from_input` and `retry_event_continuation` to match the new `Result` return types and route errors through `finalize_phase_error`.
      - Remove the now-inlined post-quantifier metadata save from `run_from_input`.
  - Subagent validation: `cargo check --all-targets --all-features` must be green.

### Phase 2: Verify and close

- [ ] #### Task 2.1: Test and land (3 SP)
  - Tighten `test_trigger_continuation_save_post_trigger_error` to expect `Err(PhaseError::PersistFailed { label: "pre-event snapshot", .. })`.
  - Add a focused test in `pipeline_tests.rs` that forces the pre-quantifier narration save to fail (e.g., override `save_snapshot`) and asserts `run_from_input` returns `Ok(())` with `GenerationStatus::Error` set.
  - Run `python build.py` in `chronicler_engine/` and verify green.
  - Update the ticket file to `Status: resolved` and append the resolution to the map's `Decisions so far`.

## Test Plan
- Existing pipeline tests continue to pass (`test_pipeline_runs_to_completion`, `test_pipeline_returns_error_on_narration_failure`, `test_pipeline_trigger_complete_failure`, `test_pipeline_trigger_empty_continuation`, `test_trigger_continuation_save_post_trigger_error` after tightening).
- New test covers the unified phase-error path for a main-flow persistence failure.
- `python build.py` green.

## Per Task/Sub Task Validation Steps
- Task 1.1: `cargo check --all-targets --all-features` green.
- Task 2.1: `python build.py` green; no failing tests.

## Assumptions
- The pre-quantifier `warn!`-and-continue saves in `phase_narrate` and `phase_post_generation` are accidental inconsistencies; they are promoted to terminal `PhaseError`.
- The post-quantifier metadata save is the one deliberate best-effort swallow; it moves into `phase_post_generation` and stays a `warn!`.
- Trigger/reconcile failures are represented as `PhaseError::NarratorFailed`; no new `PhaseError` variant is added.
- The shared `set_error` helper persists before returning `Err`, matching the existing `error_return` / `persist_snapshot_or_err` pattern; the resulting double-persist through `finalize_phase_error` is accepted as a known cost of unifying the contract.
