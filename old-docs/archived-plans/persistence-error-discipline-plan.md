# Plan: Propagate mid-pipeline persistence errors through `PhaseError::PersistFailed`

**Date:** 2026-04-18
**Status:** Planned
**Goal:** Remove the mid-pipeline log-and-continue swallow-sites in `action_pipeline` by routing them through the existing `PhaseError::PersistFailed` seam. End-of-pipeline swallow-sites stay, with explicit best-effort justification.

**Origin:** Ticket 01 (`.scratch/move-refactor-review-debt/issues/01-atomic-persistence.md`) — thermo-nuclear review finding #10.

**Scope rule:** Atomicity / single-transaction persistence is OUT OF SCOPE. The atomicity gap (`save_message_and_snapshot` non-transactional writes; `Storage::persist_swipes` swallowing per-swipe errors) pre-exists the branch and is recorded in the map's Out-of-scope section. This plan only fixes the error-discipline regression the review correctly called out.

---

## Overview

The `action_pipeline` currently has 5 swallow-sites where `save_message_and_snapshot` / `save_state` errors are logged and silently dropped. One of them (`persist_snapshot_or_err`) already propagates via `PhaseError::PersistFailed { label, source }` — that's the existing disciplined seam. The other four leak partial state:

| Site | Location | Shape | Cluster |
|------|----------|-------|---------|
| `PipelineRun::persist` | `phases.rs:57-62` | `error!` + drop | end-of-pipeline |
| `phase_narrate` pre-quantifier | `phases.rs:157-164` | `warn!` + continue | **mid-pipeline** |
| `phase_post_generation` pre-save | `phases.rs:180-187` | `warn!` + continue | **mid-pipeline** |
| `phase_trigger_continuation_llm_call` error branch | `phases.rs:258-264` | `error!` + continue (already in error path) | end-of-pipeline |
| `run_from_input` post-quantifier | `pipeline.rs:101-109` | `warn!` + continue | **mid-pipeline** |

The mid-pipeline sites allow the pipeline to continue against partially persisted state: snapshot saved, message not saved; or quantifier runs against stale snapshot. End-of-pipeline sites (`phase_finalize`'s `persist`; trigger error-state save already inside an error return) have no upstream to propagate to — swallowing there is defensible.

---

## Architecture Decisions

1. **Reuse the existing seam.** `PhaseError::PersistFailed { label, source }` already exists in `phase_error.rs` and is already used by `persist_snapshot_or_err` and the `phase_engine_commit` call site in `pipeline.rs:107-111`. No new error variant, no new abstraction.

2. **Propagate mid-pipeline sites; annotate end-of-pipeline sites.** The 3 mid-pipeline sites convert to `return Err(PhaseError::PersistFailed { label, source })`. The 2 end-of-pipeline sites keep swallowing but add a `// best-effort: pipeline already terminal` comment naming why. This matches the grilling-agreed split.

3. **No signature changes.** Every mid-pipeline site is already in a `Result<_, PhaseError>` function, so propagation is direct (`?` or explicit `return Err`). `phase_post_generation` returns `QuantifierResult`, not `PhaseError` — but its only caller (`pipeline.rs:99`) already reads the `state.narrative.input_buffer.status` mutation; the continuation-of-flow on mid-pipeline failure in that phase is handled by setting `GenerationStatus::Error` and short-circuiting with a default `QuantifierResult` (the existing pattern inside that fn). No signature change, matches the function's existing internal error shape.

4. **No transaction story.** This plan does NOT introduce `rusqlite::Transaction`. The atomicity gap is pre-existing, out of scope for this map. If pursued later it earns its own effort.

5. **No change to `persist_swipes` swallowing.** Same reason — pre-existing, out of scope. Recorded in map Out-of-scope.

6. **Test seam preserved.** `with_backend_mut` checks `TestOverride` per-method. Existing failure-injection tests for `save_message_and_snapshot` keep working; no new override surface needed.

---

## Implementation

### Step 1: `phase_narrate` pre-quantifier narration save (site at `phases.rs:157-164`)

**Before:**
```rust
if let Err(e) = self
    .pipeline
    .persistence
    .save_message_and_snapshot(&mut state)
{
    tracing::warn!("Failed to save pre-quantifier narration: {e}");
}
```

**After:**
```rust
if let Err(source) = self
    .pipeline
    .persistence
    .save_message_and_snapshot(&mut state)
{
    return Err(PhaseError::PersistFailed {
        label: "pre-quantifier narration",
        source,
    });
}
```

`phase_narrate` already returns `Result<_, PhaseError>` — direct propagation. `pipeline.rs:87-94` already handles `Err(e) => Self::finalize_phase_error(&run, e)` for this call site.

### Step 2: `phase_post_generation` pre-save (site at `phases.rs:180-187`)

`phase_post_generation` returns `QuantifierResult`, not `PhaseError`. Its caller (`pipeline.rs:99`) does not match on an `Err` arm. To propagate without changing the signature, embed the error in `input_buffer.status` and return a default `QuantifierResult` — matches the existing shape inside this function (see the `npc_ids.is_empty() && !confidence.is_high()` branch at `phases.rs:217-228` which mutates `input_buffer` + returns a modified `QuantifierResult`).

**Before:**
```rust
state.narrative.input_buffer.phase = GenerationPhase::Quantifying;
if let Err(e) = self.pipeline.persistence.save_message_and_snapshot(state) {
    tracing::warn!("Failed to save pre-quantifier phase update: {e}");
}
```

**After:**
```rust
state.narrative.input_buffer.phase = GenerationPhase::Quantifying;
if let Err(e) = self.pipeline.persistence.save_message_and_snapshot(state) {
    tracing::error!("Failed to save pre-quantifier phase update: {e}");
    state.narrative.input_buffer.status = GenerationStatus::Error(format!(
        "Failed to save pre-quantifier phase update: {e}"
    ));
    return QuantifierResult::default();
}
```

Caller in `pipeline.rs:99-109` proceeds to `phase_engine_commit`, reads `state.narrative.input_buffer.status`, and if it's `Error`, `phase_engine_commit` / `phase_finalize` will surface it. The follow-on `save_message_and_snapshot` at `pipeline.rs:106` will continue to warn-and-swallow as before (or, per Step 3 below, propagate).

**Ponytail lite note:** `QuantifierResult::default()` IS a partial-state continuation (caller proceeds to engine commit with empty quantifier result), which is a leak under a strict reading. The clean fix is changing the signature to `Result<QuantifierResult, PhaseError>`. Choosing the embed-and-default path to keep the diff minimal and match the function's existing error shape; revisit if callers need the strict signal. `# ponytail: embed-in-status preserves signature; promote to Result<QuantifierResult, PhaseError> if caller needs strict propagation`.

### Step 3: `run_from_input` post-quantifier save (site at `pipeline.rs:101-109`)

**Before:**
```rust
if let Err(e) = run
    .pipeline
    .persistence
    .save_message_and_snapshot(&mut state)
{
    tracing::warn!("Failed to save post-quantifier metadata: {e}");
}
```

**After:**
```rust
if let Err(e) = run
    .pipeline
    .persistence
    .save_message_and_snapshot(&mut state)
{
    Self::finalize_phase_error(
        &run,
        PhaseError::PersistFailed {
            label: "post-quantifier metadata",
            source: e,
        },
    );
    return Ok(());
}
```

`run_from_input` returns `Result<(), PhaseError>` and treats persist failures as terminal (same shape as the `phase_engine_commit` error handling at `pipeline.rs:104-112`). Option B would be `return Err(PhaseError::PersistFailed { ... })` — but `run_from_input`'s contract is "return `Err` only for `Cancelled`" (see `pipeline.rs:90-92` `Err(PhaseError::Cancelled) => return Err(run.handle_cancellation())`); other errors go through `finalize_phase_error` + `Ok(())`. Picking the existing shape.

### Step 4: Annotate end-of-pipeline sites

**`phases.rs:57-62` — `PipelineRun::persist`:**
```rust
pub(super) fn persist(&self, state: &GameState) {
    // best-effort: pipeline already terminal — called by phase_finalize and error-return paths
    if let Err(e) = self.pipeline.persistence.save_state(state) {
        tracing::error!("Failed to persist state: {e}");
    }
}
```

**`phases.rs:258-264` — trigger error-state save** (already inside an error-returning branch):
```rust
if let Err(e2) = self
    .pipeline
    .persistence
    .save_message_and_snapshot(&mut state)
{
    // best-effort: pipeline already terminal — trigger narration failed, we're returning anyway
    tracing::error!("Failed to persist trigger error state: {e2}");
}
```

### Step 5: Verification

1. `cargo test --test guardrails` — must not regress.
2. `cargo clippy --all-targets -- -D warnings` — clean.
3. `python3 build.py` at repo root — all tests pass.
4. Targeted test: any existing test in `pipeline_tests.rs` covering failure-injection on `save_message_and_snapshot` should still pass; add a regression test if none exists, pinning that a `PersistFailed` on pre-quantifier narration aborts the run (not silently continues).
5. Grep audit: `rg 'tracing::(warn|error).*Failed to save' chronicler_engine/src/application/action_pipeline/` shows only the 2 end-of-pipeline annotations plus `persist_snapshot_or_err`'s own internal `error!`.

---

## Pitfalls

- **`phase_post_generation`'s default-return leaks partial state.** The ponytail comment marks this. If a caller ever needs a strict signal, promote to `Result<QuantifierResult, PhaseError>` and update `pipeline.rs:99`.
- **`persist_swipes` swallowing remains.** Out of scope. If a future transaction effort touches it, that effort owns the cleanup.
- **Test injection.** `with_backend_mut`'s `TestOverride` is keyed by method name string. The propagated sites use the same `save_message_and_snapshot` / `save_state` methods — existing overrides apply.

---

## Verification

- [ ] Mid-pipeline `warn!` + continue sites at `phases.rs:157-164`, `phases.rs:180-187`, `pipeline.rs:101-109` are gone.
- [ ] End-of-pipeline sites carry `// best-effort: pipeline already terminal` annotation.
- [ ] `cargo test --test guardrails` green.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `python3 build.py` green.
- [ ] `rg 'tracing::(warn|error).*Failed to save' chronicler_engine/src/application/action_pipeline/` shows only expected sites.
