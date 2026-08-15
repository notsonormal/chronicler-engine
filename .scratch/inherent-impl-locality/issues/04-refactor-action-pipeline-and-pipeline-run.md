# 04 — Refactor ActionPipeline to module-per-type

Type: task
Status: ready-for-agent
Blocked by: (none)

> **Scope narrowed and re-pathed.** The `PipelineRun` half of this ticket is
> done (resolved by commit `4c704bd`, see `## What changed since this ticket was
> written`), and the folder-exemption question that used to shadow it is moot.
> What remains is `ActionPipeline` only. Paths updated from `action_pipeline/`
> to `pipeline/`.

## Question

Refactor `ActionPipeline` so its inherent impls satisfy `guardrails_inherent_impl_locality`.

Current state on `main` (folder renamed `action_pipeline/` → `pipeline/` by commit `4c704bd`):

- `ActionPipeline` defined in `application/pipeline/core.rs`
- `impl ActionPipeline` blocks spread across four files in `application/pipeline/`:
  - `core.rs` (def + impls)
  - `action.rs`
  - `phases.rs`
  - `retry.rs`
  - `retrigger.rs`

Folder `pipeline/` does **not** match `snake(ActionPipeline) == "action_pipeline"`. The old folder-exemption edge case (folder named `action_pipeline/` matching the type) is gone — the rename made `ActionPipeline` an unambiguous violation. There is no longer a folder-exemption question to resolve; every off-`core.rs` `impl ActionPipeline` block is a plain violation.

Target shape — single-file consolidation:

```text
application/pipeline/
  core.rs    # struct ActionPipeline + ALL impl ActionPipeline blocks
  ...        # action.rs / phases.rs / retry.rs / retrigger.rs keep only
             #   non-ActionPipeline code (entry-path fns, PipelineRun, etc.)
```

If `core.rs` would exceed 2000 lines (file_length guardrail) after consolidation, switch to a folder shape:

```text
application/pipeline/
  action_pipeline/
    mod.rs                # struct ActionPipeline
    <method-group>.rs     # split impl ActionPipeline (allowed: folder = snake)
  core.rs                 # re-export or empty
```

Constraints:
- `build.py` green at every landed step.
- Preserve all `pub`/`pub(crate)`/`pub(super)` visibility signatures exactly.
- Do NOT touch `PipelineRun` — it is already clean (def + both `impl<'a> PipelineRun<'a>` blocks in `phases.rs`).
- Do NOT touch trait impls.
- Do NOT touch `DefaultApplicationService` (deleted — see ticket 05, out of scope).

Acceptance:
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `ActionPipeline` violations.
- Full `build.py` green.
- No new `guardrails_*` failures.

## What changed since this ticket was written

- **`PipelineRun` is clean.** Commit `4c704bd` split the monolithic `pipeline.rs` into `core.rs` plus entry modules (`action.rs`, `retrigger.rs`, `retry.rs`). `PipelineRun`'s def and both its `impl<'a> PipelineRun<'a>` blocks now all live in `phases.rs` (`impl_path == def_path`). The four `impl PipelineRun` methods that used to live in `pipeline.rs` were relocated. Nothing left to do for `PipelineRun`.
- **Folder exemption question is moot.** The folder was renamed `action_pipeline/` → `pipeline/`, so `snake(ActionPipeline) == "action_pipeline"` no longer matches the parent dir. `ActionPipeline` is now flagged by the rule as specified, with no formula-tightening needed. The map's old "Not yet specified" fog item about rule-formula-vs-feature is cleared.
- **`DefaultApplicationService` reference is stale.** The original ticket told the agent not to touch `DefaultApplicationService` impls in `retry.rs` / `message_editing.rs`. That type is deleted (ticket 05); those impl blocks no longer exist. `retry.rs` in `pipeline/` now holds `ActionPipeline` and/or `PipelineRun` code only.
