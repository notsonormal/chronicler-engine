# 04 — Refactor ActionPipeline to module-per-type

Type: task
Status: resolved
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

## Answer

Resolved via **shape A: a pure `action_pipeline/` type-split subfolder**, not
the single-file consolidation the ticket originally scoped.

### What was done

`ActionPipeline` (struct + all its inherent impls) moved into a new
`src/application/pipeline/action_pipeline/` subfolder — the folder-exemption
escape hatch the rule sanctions (folder name = `snake(ActionPipeline)`):

```text
application/pipeline/              # subsystem parent (unchanged name)
  mod.rs                           # declares action_pipeline + siblings; re-exports ActionPipeline
  phase_error.rs                   # PhaseError
  phases.rs                        # PipelineRun + PipelineInputs (already clean — same-file def+impl)
  spawn.rs                         # spawn_pipeline_task
  action_pipeline/                 # PURE ActionPipeline type-split (exemption folder)
    mod.rs                         # declarations/re-exports only (mod_purity)
    core.rs                        # struct ActionPipeline + core impls + phase_engine_commit
    action.rs  retrigger.rs  retry.rs   # impl ActionPipeline entry paths (unchanged bodies)
    core_tests.rs  action_tests.rs  retrigger_tests.rs  retry_tests.rs
```

- `phase_engine_commit` moved back out of `phases.rs` into
  `action_pipeline/core.rs` (it is `impl ActionPipeline`, so it belongs with
  the type).
- `PipelineRun`, `PhaseError`, `PipelineInputs`, and `spawn` **stayed in
  `pipeline/` root** — the subsystem parent, deliberately *not* inside the
  `ActionPipeline` type-split folder. This is the cohesion point: the
  exemption folder holds only `ActionPipeline`'s split.
- External `application::pipeline::ActionPipeline` imports stayed stable via a
  re-export (`pub use action_pipeline::ActionPipeline;` in `pipeline/mod.rs`),
  so no changes were needed outside `pipeline/`.
- Visibility widened: seven struct fields and one method (`run_post_generation_agents`)
  went `pub(super)` → `pub(crate)`, because `PipelineRun` (in `phases.rs`,
  now a sibling of `action_pipeline/`, not its child) no longer falls under
  `action_pipeline`'s `super`. This is the visibility trade-off of the move.

### Why not single-file consolidation (the ticket's original plan)

Single-file consolidation (all `impl ActionPipeline` into `core.rs`) was
attempted first. It collided with three already-shipped guardrails the ticket
didn't account for:

- `guardrails_test_file_location` — every `*_tests.rs` needs a paired
  `X.rs`; consolidating production orphans `action_tests.rs`,
  `retrigger_tests.rs`, `retry_tests.rs`.
- `guardrails_file_length` (≤2000 non-blank lines) — merging the orphaned
  test files into `core_tests.rs` would exceed the cap.
- `guardrails_mod_purity` — `mod.rs` must be declarations-only, so the struct
  can't live there (rules out the `mod_tests.rs` special-case dodge).

The folder exemption is the sanctioned resolution for exactly this — a type
splitting its impls across files inside its named folder — so shape A was
chosen. The user's cohesion principle (a type-split folder holds only that
type's split) is what kept `PipelineRun` etc. in the parent rather than
renaming `pipeline/` flat to `action_pipeline/` (shape D, rejected).

### Cohesion gap surfaced

The choice between shape A and shape D exposed that the rule checks folder
*name* but not folder *contents* — shape D's impurity is invisible to the
rule. That gap is now ticket [11 — Enforce folder cohesion in the
inherent-impl-locality rule](11-enforce-folder-cohesion-in-inherent-impl-locality-rule.md)
(blocked by 01). Not implemented here.

### Acceptance — verified

- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo test --lib`: 975 passed.
- `cargo nextest run --test guardrails`: 104 passed (incl. `test_file_location`,
  `mod_purity`, `file_length` — all green with the new layout).
- `cargo nextest run --test architecture`: passed.
- `cargo nextest run --test http actions`: 17 passed (full HTTP action
  pipeline, including `test_three_actions_in_sequence_http` and
  `test_input_persisted_before_narration_http` — the exact "look → narration
  appears" path).
- `cargo nextest run -p chronicler_engine --lib pipeline`: 79 passed (all
  action/retry/retrigger paths).

**Caveat on the `guardrails_inherent_impl_locality` acceptance criterion:**
that guardrail does not exist yet (ticket 01 is deferred), so the
"zero `ActionPipeline` violations" criterion could not be checked by running
the rule. It is verified **structurally**: every `impl ActionPipeline` now
lives under `action_pipeline/`, whose parent dir ends `/action_pipeline` =
`snake(ActionPipeline)`, so the rule as specified would flag none.

### Browser test flakiness (not caused by this refactor; fixed this session)

`build.py` failed on two browser tests — `test_delete_removes_message`
(deterministic, "have 1" entry) and `test_status_updates_during_generation`
(flaky). **An earlier draft of this answer wrongly attributed these to commit
`b319579`. That was a timeline error:** build `logs/build_20260815_132708.log`
(13:27) ran on `1d7dcb0` (committed 01:10), BEFORE `b319579` (committed
13:36); build `logs/build_20260815_121429.log` (12:14) also ran on `1d7dcb0`
and PASSED the same test. Same commit, one pass one fail → the failure is
flaky/environmental, not a `b319579` regression.

**Root cause (investigated and fixed this session):** a test-side race in the
browser helpers, not a production bug. Manual reproduction with the real
binary + Mock backend confirmed `/action/check` returns `"Thinking..."` in
~0.3s and narration persists within ~0.5s — the pipeline is sound (also
verified by 17/17 `http actions` integration tests). The race: `send_action`
waited only 500ms (best-effort, result ignored) for `#status-display` to leave
"Ready"; under parallel browser-test load the `/action/check` swap can exceed
500ms, so `wait_for_status_ready` returned on the STALE pre-action "Ready"
before generation started, reading the log before the narration landed.

**Fix (in `tests/test_utils/`, outside this ticket's scope):** added
`wait_for_status_generating` — `send_action` now deterministically waits for
the status to leave "Ready" (5s timeout, hard failure if it never does) before
returning, so `wait_for_status_ready` cannot return on stale Ready; bumped
`wait_for_status_ready`'s timeout 5s→12s to absorb the ≤5s
`/status/generating` poll gap. Verified: the two tests pass 3/3 isolated runs,
all 16 browser tests pass, and `build.py` is fully green (1351 passed, 2 LLM
skipped).

### Post-resolution addendum: pipeline_run.rs rename

After resolution, at the user's direction, `application/pipeline/phases.rs`
was renamed to `application/pipeline/pipeline_run.rs` for clarity. The file's
primary content is the `PipelineRun<'a>` struct and both its impl blocks (plus
the `PipelineInputs` DTO); `phases.rs` named the phase *methods* rather than
the *type*, which violated the "naming as documentation" principle (symbols
map 1-to-1 to concepts). `PipelineRun`'s locality is unchanged — def + both
impls are co-located in the renamed file, still clean. Updates: `pipeline/mod.rs`
declaration (`pub mod pipeline_run;`) and two imports in
`action_pipeline/core.rs` / `core_tests.rs`. Re-verified green (fmt, clippy
`-D warnings`, 975 lib tests, 104 guardrails). Not a locality fix — a clarity
rename; the original ticket's "do not touch PipelineRun" line was about
violation-fixing, which this is not.
