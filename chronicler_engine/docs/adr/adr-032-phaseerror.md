# ADR-032: PhaseError — replace `ActionOutcome` with errors-only enum

**Date:** 2026-07-13
**Status:** Accepted

## Context

The action pipeline had a binary `ActionOutcome { Completed, Cancelled }` returned from every phase method. The `Completed` variant was redundant with `Ok(())` — a `from_pipeline_result` translator existed solely to convert `Ok(())` → `Completed`. Every existing `ActionOutcome::` reference in the codebase checked only `Completed` or `Cancelled`; the type was a vestigial wrapper.

Two orchestrators (`run_from_input` in `pipeline.rs`, `retry_last_response_impl` in `retry.rs`) had to mix five error styles (early-return, bool, `tracing::error!`, `save_retry_error`, `map_cancelled` wrapper) because phase methods couldn't express rich failure modes. Tickets 03 + 04 collapse those styles into a linear `match PhaseError` — but they cannot land until phase methods return a unified error type.

Grilling session (ticket 01, decision **D2**) chose to **replace** `ActionOutcome` rather than extend it, for three reasons:

1. `ActionOutcome`'s contract — "binary terminal state for external callers" (`actions.rs:13`) — is a real abstraction. Coupling external callers to internal narration/persistence error variants would leak phase internals.
2. Phase-internal errors (`NarratorFailed`, `PersistFailed { label }`, `TriggerMissing`, `SnapshotMissing`) have different recovery semantics (LLM error → set status; persist error → log + abort; missing trigger → inject error message; missing snapshot → fail-loud). The unified enum names them explicitly.
3. The translator (`from_pipeline_result`) was a pure rename of `Ok(())` — it carried no policy and only existed to satisfy the type alias.

## Decision

Replace `ActionOutcome { Completed, Cancelled }` with `PhaseError`, errors-only, defined in `application/action_pipeline/phase_error.rs`:

```rust
pub enum PhaseError {
    Cancelled,
    NarratorFailed(String),
    PersistFailed { label: &'static str, source: EngineError },
    TriggerMissing,
    SnapshotMissing,
    FetchFailed(String),
}
```

Success is `Ok(())`. `Completed` dies. `from_pipeline_result` dies with the enum. `PipelineResult<T>` type alias dies; callers use `Result<T, PhaseError>` directly. `handle_cancellation` returns `PhaseError::Cancelled` instead of `ActionOutcome::Cancelled`. `map_cancelled` is inlined into its two call sites (`run_from_input`, `phase_trigger_continuation_with_cancel_handling`).

Mechanical churn: 4 caller renames + 3 test assertions + 1 comment + deletion of `ActionOutcome` / `PipelineResult` / `from_pipeline_result` / `map_cancelled` / `persist_snapshot_failed` (bool → replaced by `persist_snapshot_or_err` returning `Result<(), PhaseError>`).

`save_retry_error` stays alive in this ticket; ticket 04 deletes it.

### Why `Ok(())` over a `Completed` variant

`Ok(())` is the universal Rust success indicator. `ActionOutcome::Completed` was redundant — the translator existed solely to convert one to the other. Eliminating the variant removes a layer of indirection, makes the success path match the rest of the Rust ecosystem, and forces callers to think in `Result` (where error variants carry meaning) rather than in a custom enum (where `Completed` carries none).

### Why `PersistFailed { label: &'static str, source: EngineError }` carries a label

The `label` distinguishes the call site that failed (`"pre-main snapshot"`, `"pre-event snapshot"`, `"post-trigger snapshot"`, `"post-engine snapshot"`). Today, this string is only used in `tracing::error!` and `GenerationStatus::Error(format!(...))` messages. The label lives in the enum so:

1. Orchestrators can surface which persist step failed without re-deriving the string from a stack trace.
2. A future `match PhaseError` block in tickets 03 + 04 can dispatch on the label for differential recovery (e.g., pre-main snapshot failure → retry-able; post-trigger failure → terminal).

We deliberately do NOT provide a `From<EngineError> for PhaseError` blanket impl: every `PersistFailed` construction must be intentional, with the label chosen at the call site.

### Why `FetchFailed` was added in ticket 04

Retry precondition fetches (game/world/persona/npc bundle in `retry_event_continuation`) wrap an underlying `EngineError` from the persistence/storage layer. We give them a distinct `FetchFailed(String)` variant — not `NarratorFailed` — because the recovery policy differs:

- `FetchFailed` is a precondition check: the orchestrator could not even assemble the inputs required to call the narrator. Terminal, surface to user via `GenerationStatus::Error`.
- `NarratorFailed` is a mid-flow LLM failure: the LLM was called and returned an error or empty response.

Both end up funneled to `finalize_phase_error` and produce `GenerationStatus::Error`, but the seam deserves to know whether the failure was a storage lookup or an LLM call — future diagnostics or differential recovery will dispatch on the variant. The `String` payload is `EngineError::to_string()` so display-shaped failure messages like `Game not found: 999` flow through unchanged (locked in by `retry_records_canonical_game_not_found_when_game_missing`). This was intentionally deferred from ticket 02's baseline variant set because ticket 02 did not touch the retry body.

### Why no `std::error::Error` or `Display` impl

No cross-boundary bubble today. Orchestrators consume variants inline; the HTTP layer never sees a `PhaseError`. If a downstream caller needs to render one (e.g., to JSON for a debug endpoint), add the impl then.

## Consequences

### Positive

- Phase method return types become honest: `Result<PhaseOutput, PhaseError>` names what can go wrong, where `PhaseOutput` names what succeeded.
- Tickets 03 + 04 can now collapse the 5-style error mixing into one linear `match PhaseError` per orchestrator.
- `map_cancelled` ad-hoc pattern (polish-25) is gone — the cancel-handling logic is one match arm at each call site.
- `persist_snapshot_failed` (polish-18: name lies, returns bool) becomes `persist_snapshot_or_err` returning `Result<(), PhaseError>`. The name matches the signature; the return type is honest.
- Translator layer (`from_pipeline_result`, `PipelineResult<T>` type alias) removed — one less indirection.

### Negative

- `run_from_input` propagates `Err(PhaseError::PersistFailed { ... })` for pre-event and post-trigger snapshot failures where it previously absorbed to `Ok((state, empty))`. This is a deliberate behavior change at two sites (tickets 03 + 04 will decide whether `phase_finalize` runs on `PersistFailed`).
- `phase_pre_main_snapshot` propagates `Err(PhaseError::PersistFailed { ... })` via `?` where it previously early-returned `Ok(state)` after `phase_finalize`. Same rationale: 03 + 04 will linearize.
- Test `test_trigger_continuation_save_post_trigger_error` was updated to accept the new `PersistFailed` variant in its `Err` arm. The test's intent ("snapshot failure produces empty text OR a clean error") is preserved.
- `error_return` signature changed to `Result<_, PhaseError>` but the body still returns `Ok((state, empty, empty, empty))` — preserving the "narrator failure → `GenerationStatus::Error` + `Ok(())` from `run_from_input`" contract that `test_pipeline_returns_error_on_narration_failure` locks in. **Surface:** ticket 02 spec asked for the body to return `Err(PhaseError::NarratorFailed(msg))`; the test contract took precedence (ticket author flag this as a behavior-preservation decision for ticket 03 to revisit).

### Trade-offs

- Chose replacement (`PhaseError` enum, `Ok(())` = success) over extension (keep `ActionOutcome`, add error variants) — eliminates the `Completed` redundancy and matches the rest of Rust's `Result` convention.
- Chose explicit `label: &'static str` over a generic `String` — labels are static for the lifetime of the call site; allocation-free.
- Chose to NOT propagate `NarratorFailed` from `error_return` (kept body returning `Ok(...)`) — preserves today's error model where the UI polls on `GenerationStatus::Error` via `get_generating_status` and the pipeline returns `Ok(())` so the orchestrator can call `phase_finalize` and clear the registry slot. Ticket 03 may revisit this when linearizing `run_from_input`.
- Chose to fold all type churn into ticket 02 (rebaselined from 2 SP to 5 SP per ticket note) — splitting type deletion from caller renames left a non-compiling intermediate state.

## Related ADRs

- [ADR-014: Action Pipeline Architecture](./adr-014-action-pipeline.md) — original phase-based pipeline design.
- [ADR-027: Hexagonal Architecture Migration](./adr-027-hexagonal-architecture-migration.md) — pipeline lives in `application/`, ports/traits collapsed.
- Ticket 01 — G3 decision (D2): [`.scratch/t4-phase-error/issues/01-g3-phaseerror-bridge-location.md`](../../.scratch/t4-phase-error/issues/01-g3-phaseerror-bridge-location.md).

## History

- **2026-07-13**: Initial acceptance (ticket 02 of T4 PhaseError wayfinding).
- **2026-07-14**: Amended by ticket 04 — added `FetchFailed(String)` variant for retry precondition fetch failures.
- **2026-08-01**: Post-execution clean-up (arch-exec-wiredapp-pipeline tickets 20–23). `ActionPipeline::persist_generation_error` is the single helper that sets `GenerationStatus::Error` and resets phase; `finalize_phase_error` no longer reloads via `load_or_fresh` + `phase_finalize` (double-write removed). `phase_narrate` / `phase_post_generation` / `phase_trigger_continuation_llm_call` / `reconcile_post_trigger_npcs` now return `Result<_, PhaseError>` and route errors through `finalize_phase_error`. `error_return` (ticket 21) now returns `Err(PhaseError::NarratorFailed(msg))` via the `set_error` helper — the earlier "test contract took precedence" note in *Negative* is stale: the change is the unification ticket 22 moved to `Err`. `map_cancelled` is no longer inlined into `run_from_input` and `phase_trigger_continuation_with_cancel_handling`; it is now a `PipelineRun` method used by both call sites after the wrapper was deleted. `prepare_retry_state` persists via `PersistenceGate::save_state` instead of bypassing the gate. `apply_npc_events` and `commit_trigger_narration` take `&mut self` and return `Result<()>`, removing the by-value clones the pipeline used to need.