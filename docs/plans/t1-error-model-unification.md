# T1: Error Model Unification

**Parent Plan:** [abstraction-fixes-followup-superplan.md](./abstraction-fixes-followup-superplan.md)
**Status:** Planning — sub-plan scoping decisions pending
**Date:** 2026-06-28
**Depends on:** none (T1 is a root-cause unblocker; R1 in `reliability-and-cancellation-plan.md` sequences after T1)
**Blocks:** T2-ARCH (soft — error shape), R1 (reliability plan — swallow-vs-propagate policy)
**Priority:** P1
**Findings owned:** A1, B2, B3, B9, N3

---

## Summary

Pipeline methods return `PipelineResult<T>` BUT failure is also signalled via `state.narrative.input_buffer.status = GenerationStatus::Error(msg)`. The seam is misplaced (state mutation, not return value). `Ok` does not always mean success — the interface lies. ADR-018 + commit `8e4acf5` deliberately pinned errors onto `GenerationStatus`; revisit because the friction is real.

Three error channels coexist today:
- `EngineError` for storage/IO (`src/error.rs`)
- `GenerationStatus::Error(String)` for pipeline narration errors (`state/generation_status.rs`)
- `ActionOutcome` for pipeline cancellation (`action_pipeline/pipeline.rs`)
- Helper `error_return(&self, state, msg) -> PipelineResult<...>` returns `Ok` while writing `state.status = GenerationStatus::Error(msg)` (`phases.rs:53-61`). Each caller checks `status.error_message().is_some()` after.

## Key Changes

1. Pick a single error type at the pipeline boundary: `PipelineError` enum capturing `Cancelled` / `LlmFailed(String)` / `StorageFailed(EngineError)` / `QuantifierFailed(String)` / `TriggerFailed(String)`.
2. Delete the `error_return` helper; replace with explicit `Err(PipelineError::...)` propagation.
3. Retain `GenerationStatus::Error` purely for state-machine UI rendering — NOT for pipeline control flow.
4. Convert every mid-flow `status.error_message().is_some()` check (~5 sites in `pipeline.rs` + `phases.rs`) to `?` propagation.

## Decisions to Lock

- Fold `ActionOutcome::Cancelled` into `PipelineError::Cancelled` or keep separate as an exhaustiveness aid?

## Out of Scope

- Removing `GenerationStatus::Error` variant entirely (UI status rendering depends on it).
- Swallow-vs-propagate policy on `save_message_and_snapshot` warn paths — R1 in reliability plan.
- `run_from_input` state-machine rewrite (B3 deferral, locked per Phase 6.1 Issue 9 constraint).

## Blast Radius

~3–5 files in `application/action_pipeline/`. Storage layer untouched.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- Integration test coverage for new error paths (cancelled, LLM failed, storage failed, quantifier failed, trigger failed) — required before merge (structural track).
- Verify `architecture/system.md:49` + `system/game_flow.md` "Error Model" section still match new shape; update if not.

## Pre-Implementation Checklist

- [ ] Run `improve-codebase-architecture` skill ADR-conflict check vs ADR-018 + commit `8e4acf5` before writing code.
- [ ] List every mid-flow `status.error_message().is_some()` check (3+ sites in `pipeline.rs`, more in `phases.rs`).
- [ ] Confirm `AGENTS.md` plan-adherence rule: any deviation from super-plan scope requires user approval.
