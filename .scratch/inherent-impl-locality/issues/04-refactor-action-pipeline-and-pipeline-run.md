# 04 — Refactor ActionPipeline and PipelineRun to module-per-type

Type: task
Status: ready-for-agent
Blocked by: (none)

## Question

Refactor `ActionPipeline` and `PipelineRun` so their inherent impls satisfy `guardrails_inherent_impl_locality`.

Current state:
- `ActionPipeline` defined in `application/action_pipeline/pipeline.rs`
- `impl ActionPipeline` in `pipeline.rs` + `phases.rs` (4 methods in phases.rs: `phase_pre_main_snapshot`, `phase_trigger_continuation_with_cancel_handling`, `phase_finalize`, `handle_cancellation`)
- `PipelineRun<'a>` defined in `application/action_pipeline/phases.rs`
- `impl PipelineRun` spread across `phases.rs` + `pipeline.rs` (one method: `phase_engine_commit` in phases.rs is `impl ActionPipeline`)

Folder `action_pipeline/` matches `ActionPipeline`'s name, not `PipelineRun`'s.

Target shape:
```text
application/
  action_pipeline/
    mod.rs                   # ActionPipeline definition + impl ActionPipeline (all methods here)
    action_pipeline.rs → merge into mod.rs OR keep as single file?
    pipeline_run/
      mod.rs                 # struct PipelineRun<'a>
      <phase methods>.rs     # split impl PipelineRun (allowed: folder named after type)
  phase_error.rs             # pub enum PhaseError (stays where it is)
  retry.rs                   # impl DefaultApplicationService — see ticket 05
  retry_tests.rs             # #[cfg(test)] — unchanged location
  pipeline_tests.rs          # #[cfg(test)] — unchanged location
```

OR simpler — single-file consolidation:
```text
application/action_pipeline/
  pipeline.rs    # ActionPipeline only (def + all impls)
  phases.rs      # PipelineRun only (def + all impls) — rename file to pipeline_run.rs for clarity?
```

The single-file version has `PipelineRun` def in `pipeline_run.rs` (or `phases.rs` renamed), with all impls in that one file. This is cleaner — no folder needed for a type that fits in one file.

If `pipeline.rs` would exceed 2000 lines (file_length guardrail), switch to the folder shape.

Constraints:
- `build.py` green at every landed step.
- Preserve all `pub`/`pub(crate)` visibility signatures exactly.
- Preserve `PipelineInputs` (in `phases.rs`) — co-locate with `PipelineRun`.
- Preserve `phase_error.rs` location (already a self-contained type).
- The `ActionPipeline::phase_engine_commit` method currently in `phases.rs` — move to `pipeline.rs` (or wherever `ActionPipeline` lands).
- The 4 `impl PipelineRun` methods currently in `pipeline.rs` — move to wherever `PipelineRun` lands.
- Do NOT touch trait impls.
- Do NOT touch `DefaultApplicationService` impls (those are in `retry.rs` and `message_editing.rs` — ticket 05).

Acceptance:
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `ActionPipeline` and `PipelineRun` violations.
- Full `build.py` green.
- No new `guardrails_*` failures.
- File renaming is allowed (e.g. `phases.rs` → `pipeline_run.rs`) — update `mod.rs` declarations accordingly.
