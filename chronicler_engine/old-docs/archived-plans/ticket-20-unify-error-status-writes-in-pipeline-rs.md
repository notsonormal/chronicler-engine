# Ticket 20: Unify error-status writes in `pipeline.rs`

## Summary
Make `ActionPipeline::retry_persist_error` the single error-persistence helper by adding a phase reset and renaming it to `persist_generation_error`. Route every error path in `pipeline.rs` through it, fix the `finalize_phase_error → phase_finalize` double-write, and stop `prepare_retry_state` from bypassing `PersistenceGate`.

## Key Changes
- Rename `ActionPipeline::retry_persist_error` → `persist_generation_error`.
- Upgrade its body to reset `phase` to `GenerationPhase::default()` after setting `status = Error`, using `PersistenceGate::save_state` and logging persistence failure.
- Update all existing callers of `retry_persist_error` (~6 sites in `retry_last_response`) to use `persist_generation_error`.
- Rewrite `finalize_phase_error` to map `PhaseError` variants to a message string and call `persist_generation_error`; remove its own `load_or_fresh` + `phase_finalize` double-write.
- Rewrite `run_from_input` load-world-bundle error arm to call `persist_generation_error` and return `Ok(())`; remove inline `Error` write + `phase_finalize`.
- Change `prepare_retry_state` to persist via `self.persistence.save_state(&game_state)` instead of `self.persistence.storage().save_snapshot(...)`.
- Keep `handle_retry_outcome` `Cancelled` arm unchanged; `Err` arm already reaches the helper via `finalize_phase_error`.

## Implementation

- [ ] #### Task 1: Make `retry_persist_error` the unified error helper (3 SP)
  - [ ] Claim ticket 20 (`Status: claimed`) before editing.
  - [ ] Rename `retry_persist_error` to `persist_generation_error` and add phase reset.
  - [ ] Update all `retry_last_response` call sites to the new name.
  - [ ] Rewrite `finalize_phase_error` to map `PhaseError` → `String` and call `persist_generation_error`.
  - [ ] Rewrite `run_from_input` load-world-bundle error arm to call `persist_generation_error`.
  - [ ] Change `prepare_retry_state` snapshot persistence to `self.persistence.save_state`.
  - [ ] Add/update a unit test only if no existing test directly asserts the helper persists `Error` + default phase.

## Test Plan
- `cargo check --all-targets --all-features` green.
- `python build.py` green.

## Per Task/Sub Task Validation Steps
- After edits: `cargo check --all-targets --all-features`.
- After check: `python build.py`.
- On green build: set ticket `Status: resolved`, append answer with helper location and double-write fix, and add pointer to map's `Decisions so far`.

## Assumptions
- `phases.rs` internal error-write patterns are out of scope; ticket 21 owns them.
- `GenerationPhase::default()` remains `Narrating`.
- Helper persistence failure is logged, not propagated, preserving existing behavior.
