# Clean up `run_from_input` state flow (ticket 22)

## Summary

Refactor `ActionPipeline::run_from_input` and `phase_narrate` in `chronicler_engine/src/application/pipeline/` to remove variable shadowing, clarify the post-commit state variable, delete a redundant `last_trigger` write, and unify cancellation handling. No observable pipeline behavior changes except one fewer duplicate persist round-trip.

## Key Changes

- `phases.rs`: `phase_narrate` takes `&mut GameState` and returns `(String, String, String)` instead of `(GameState, String, String, String)`.
- `phases.rs`: `error_return` takes `&mut GameState` and returns the new tuple type.
- `pipeline.rs`: rename `next_state` to `post_commit_state` throughout `run_from_input`.
- `pipeline.rs`: delete the pipeline-level `last_trigger` write after `build_trigger_request`.
- `pipeline.rs`: add `PipelineRun::map_cancelled` helper, delete `phase_trigger_continuation_with_cancel_handling`, and route `phase_narrate` + `phase_trigger_continuation_llm_call` through it.

## Plan Review Notes

- **Decision resolved:** delete the pipeline-level `last_trigger` write (Option A). The phase method is the natural owner; the recovery edge (post-engine persist failure before entering trigger phase) is bounded by ADR-032 error-contract.
- **NOT in scope:** changing ADR-032 error contract, adding new unit tests, touching other pipeline smells (#15/#16/#18), or changing phase-internal error-write patterns (#12).
- **What already exists:** `handle_cancellation`, `phase_trigger_continuation_llm_call`, `error_return`; no reusable cancellation helper currently.
- **Failure modes:** non-cancel phase errors still route through `finalize_phase_error`; `Cancelled` is mapped once by `map_cancelled` and then propagated; `post-engine snapshot` persist failure still marks `GenerationStatus::Error`.
- **Unresolved decisions:** none.

## Implementation

- [ ] #### Task 1: Claim ticket 22 (1 SP)
  - Set `Status: claimed` in `.scratch/arch-exec-wiredapp-pipeline/issues/22-clean-run-from-input-state-flow.md`.

- [ ] #### Task 2: Refactor `phase_narrate` signature (1 SP)
  - In `chronicler_engine/src/application/pipeline/phases.rs`:
    - Change `phase_narrate` to `fn phase_narrate(&self, state: &mut GameState, inputs: &PipelineInputs) -> Result<(String, String, String), PhaseError>`.
    - Update `error_return` to `fn error_return(&self, state: &mut GameState, msg: String) -> Result<(String, String, String), PhaseError>`.
    - Remove `state` from the `Ok((...))` return tuple.

- [ ] #### Task 3: Clean `run_from_input` state flow (1 SP)
  - In `chronicler_engine/src/application/pipeline/pipeline.rs`:
    - Rename `next_state` to `post_commit_state` at its declaration and all reassignment/use sites.
    - Delete the `if let Some(trigger) = &trigger_request { next_state.narrative.last_trigger = Some(trigger.clone()); }` block.
    - Call `run.phase_narrate(&mut state, &inputs)` and destructure only the three strings.

- [ ] #### Task 4: Unify cancellation handling (1 SP)
  - In `chronicler_engine/src/application/pipeline/pipeline.rs`:
    - Add `fn map_cancelled<T>(&self, result: Result<T, PhaseError>) -> Result<T, PhaseError>` on `PipelineRun`; on `Err(PhaseError::Cancelled)` it calls `self.handle_cancellation()` and returns `Err(PhaseError::Cancelled)`.
    - Delete `phase_trigger_continuation_with_cancel_handling`.
    - Update `run_from_input` call sites to use `run.map_cancelled(...)`; keep `Err(PhaseError::Cancelled) => return Err(PhaseError::Cancelled)` arms (cleanup already done by helper).
    - Update `phase_trigger_continuation` to call `run.map_cancelled(run.phase_trigger_continuation_llm_call(...))`.

- [ ] #### Task 5: Verify and resolve (1 SP)
  - Run `cargo check --all-targets --all-features` in `chronicler_engine/`.
  - Run `python build.py` in `chronicler_engine/`.
  - If any ticket item proves invalid against current code, note it in the ticket and skip instead of forcing it.
  - Append the resolution under `## Answer`, set `Status: resolved`, and add the decision pointer to the map's `## Decisions so far`.

## Test Plan

- `cargo check --all-targets --all-features` passes.
- `python build.py` passes (cargo-nextest run + guardrails + docs validation).
- Pipeline tests exercising `run_from_input`, narration, trigger continuation, and cancellation still pass.

## Per Task/Sub Task Validation Steps

- Task 2: `cargo check` after `phase_narrate` signature change.
- Task 3: `cargo check` after `run_from_input` rename/redundant write removal.
- Task 4: `cargo check` after cancellation helper wiring.
- Task 5: full `python build.py` green before closing ticket.

## Assumptions

- `phase_narrate` mutating `state` through `&mut` does not introduce borrow issues; the caller owns `state` and `run` only borrows `self`.
- Removing the pipeline-level `last_trigger` write is safe because `phase_trigger_continuation_llm_call` writes and persists it in the pre-event snapshot.
- The `map_cancelled` helper is the minimal unification; it replaces the existing trigger-specific wrapper and the duplicated inline cancel arms.
