# Plan: Propagate post-quantifier metadata persistence failure

**Date:** 2026-08-12  
**Status:** Planned  
**Goal:** Remove the last mid-pipeline `save_message_and_snapshot` swallow site in `phase_post_generation` by routing it through `PhaseError::PersistFailed`.

## Why
`src/application/pipeline/phases.rs:phase_post_generation` currently logs and continues after a failed post-quantifier metadata save:

```rust
if let Err(e) = self.pipeline.message_service.save_message_and_snapshot(state) {
    tracing::warn!("Failed to save post-quantifier metadata: {e}");
}
```

This is the only remaining mid-pipeline persistence swallow. The pipeline already has `persist_snapshot_or_err` for the pre-quantifier save, and the caller in `pipeline.rs` already handles `PhaseError` via `finalize_phase_error`. Propagating here removes silent partial-state risk without changing any signatures.

## Scope

### Task 1: Propagate the post-quantifier save failure
- In `src/application/pipeline/phases.rs`, replace the `warn!` + continue block with:

  ```rust
  if let Err(source) = self.pipeline.message_service.save_message_and_snapshot(state) {
      return Err(PhaseError::PersistFailed {
          label: "post-quantifier metadata",
          source,
      });
  }
  ```
- Delete the preceding `// Best-effort: quantifier metadata...` comment.

### Task 2: Add regression test
- Add `phase_post_generation_returns_persist_error_on_post_quantifier_save_failure` in `src/application/pipeline/pipeline_tests.rs` (near the existing pre-quantifier failure test).
- Inject a `TestOverride` on `save_message_and_snapshot` or `persist_swipes` so the post-quantifier save fails.
- Assert the function returns `Err(PhaseError::PersistFailed { label: "post-quantifier metadata", .. })`.
- Assert `state.narrative.input_buffer.status` becomes `GenerationStatus::Error` after the caller runs `finalize_phase_error`.

### Task 3: Verify no new swallow sites
- Grep `src/application/pipeline/` for `warn!.*save|error!.*save` and confirm only terminal/best-effort sites remain.

## Out of scope
- Changing `PhaseError` enum.
- Adding transactions.
- The end-of-pipeline `PipelineRun::persist` and `finalize_phase_error` snapshot saves are terminal; they may stay best-effort with a short comment if desired, but are not required for this plan.

## Acceptance criteria

- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo test --lib` and `cargo nextest run --test pipeline` pass.
- New regression test fails before the fix and passes after.
- `rg 'warn!.*Failed to save post-quantifier'` returns no matches.
