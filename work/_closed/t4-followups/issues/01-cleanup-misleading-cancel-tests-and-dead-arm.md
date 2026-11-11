Type: task
Story points: 2
Blocked by: (none)

# Ticket 01 — Cleanup misleading cancel-test plumbing + dead arm surfaced by ticket 04 coverage pass

Per [T4 followups map](../map.md): the post-plan-workflow coverage pass on ticket 04 surfaced 3 sharp cleanup items. Since the map was charted, the action-pipeline structure has been refactored: `phases.rs` is now `pipeline_run.rs`, retry modules live under `src/application/pipeline/action_pipeline/`, and the misleadingly-named `phase_trigger_continuation_with_cancel_handling` has been renamed. One of the three items is therefore resolved; this ticket tracks the remaining two.

## Question

What does the focused cleanup of the remaining two items look like, and does it preserve every existing test contract?

## Scope

### Item 1 — ~~`phase_trigger_continuation_with_cancel_handling` name lies~~ RESOLVED by `031cf9b`

**Status:** The misleadingly-named function no longer exists. It was renamed to `phase_trigger_continuation_llm_call` in `src/application/pipeline/pipeline_run.rs` and its callers (`run_from_input` and `retry_event_continuation`) were updated as part of the pipeline refactor.

**Residual gap:** A `///` doc comment naming the α-check contract (game-id mismatch only, not shutdown-token) was **not** added. This is minor polish; the name no longer lies, so the original concern is resolved.

### Item 2 — Misleading cancellation test still passes for the wrong reason

**Today:**
- `src/application/pipeline/action_pipeline/retry_tests.rs::test_retry_event_continuation_cancels_before_llm` (around line 369-396)

`test_retry_event_impl_cancels_cleanly` has been removed, but `test_retry_event_continuation_cancels_before_llm` still claims to verify cancellation while actually relying on `phase_finalize` resetting status to `Idle` after a successful pipeline run. It cancels `app.shutdown_token` and then calls `retry_event_continuation` directly, but `retry_event_continuation` never consults the shutdown token; cancellation is not actually driven. The `Idle` assertion is a no-op that would pass even if cancellation were broken. **This is misleading-documentation test code.**

**Decide:**
- **(A) Rewrite** the test using the `thread::spawn` + `with_trigger_delay` + `app.set_game_id` flip pattern that the ticket-04 coverage tests established (`test_retry_last_response_cancelled_at_phase_boundary`). The rewrite should assert that `status == Idle` AND that the pipeline actually took the Cancelled path (e.g., assert the trigger LLM was NOT called past the cancel point). Preserve the test intent, not the implementation.
- **(B) Delete** the test if the rewrite is too invasive AND there's already sufficient Cancelled-coverage from `test_retry_last_response_cancelled_at_phase_boundary`. If deleting, document in the resolution why the existing test subsumes this one.

**Recommendation: (A) rewrite.** This test lives at a higher level (`retry_event_continuation` directly) than `test_retry_last_response_cancelled_at_phase_boundary` (which drives the orchestrator match). Different test levels catch different regressions. Rewriting preserves the contract; deleting loses the higher-level coverage.

### Item 3 — Dead `None` arm in `retry.rs` (no-input-to-retry)

**Today:** Inside `src/application/pipeline/action_pipeline/retry.rs::retry_last_response`, after reconstructing retry state:

```rust
let input_text = match state.narrative.history.last_input_text() {
    Some((_, text)) => text,
    None => {
        self.persist_generation_error("Retry failed: no input to retry");
        return;
    }
};
```

Coverage analysis: this arm is **unreachable by construction**. `find_retry_anchor` always returns an anchor whose `snapshot_id` resolves to a snapshot whose history contains an Input message, and `reconstruct_retry_state` truncates history to `anchor_idx + 1`, keeping the Input. So `last_input_text()` always returns `Some`. Current coverage shows lines 76–77 (the `None` arm body) are uncovered.

**Decide:**
- **(A) Delete the arm** and replace the `match` with a direct destructure: `let (_, input_text) = state.narrative.history.last_input_text().expect("retry anchor guarantees an Input in history");`.
- **(B) Keep the arm** as a defensive guard (defense-in-depth). The cost is one uncovered branch; the benefit is graceful handling if `find_retry_anchor`'s contract ever loosens.
- **(C) Replace with an invariant assertion** that fires loudly if the assumption ever breaks, instead of silent `persist_generation_error`. E.g., `.unwrap_or_else(|| panic!("invariant violated: retry anchor did not yield an Input message"))`.

**Recommendation: (A) or (C).** (B) keeps the dead code visible at the cost of a permanent coverage hole. If the anchor contract ever loosens, the panic from (C) surfaces immediately during development rather than silently degrading in production. Read `MessageService::find_retry_anchor_msg` and its tests before deciding.

## Out of scope

- **Cancellation-via-token propagation through all phases.** Today only the α-check (game-id mismatch) produces `Err(PhaseError::Cancelled)` from `phase_trigger_continuation_llm_call`. Propagating `app.cancel_token()` through every phase is a separate, larger mechanics effort — not in scope here.
- Anything outside `src/application/pipeline/action_pipeline/retry_tests.rs` (test rewrite) and `src/application/pipeline/action_pipeline/retry.rs` (dead arm removal). `pipeline_run.rs` should only receive the optional doc comment noted in Item 1 if you choose to add it; no behavior change.
- No `run_from_input` changes. No new public API. No `ActionPipeline` surface changes.

## Blast radius

- `src/application/pipeline/action_pipeline/retry_tests.rs` — rewrite or delete the misleading cancellation test. ~30–60 LOC.
- `src/application/pipeline/action_pipeline/retry.rs` — remove or replace dead `None` arm in `retry_last_response`. ~3–5 LOC.
- `src/application/pipeline/pipeline_run.rs` — optional `///` doc comment on `phase_trigger_continuation_llm_call` explaining the α-check contract. ~3 LOC.

## Validation

- `python build.py` green (all 12 steps)
- `cargo llvm-cov nextest --features testing --lib retry_tests` → `retry.rs` coverage stays ≥ 80% (current 86.9%; acceptable transient drop to ≥ 82% if rewrite changes line counts)
- All existing test contracts preserved — specifically the coverage tests `test_retry_last_response_cancelled_at_phase_boundary`, `test_retry_event_continuation_handles_state_without_input_message`, and `test_retry_records_missing_snapshot_id`
- The misleading cancellation test either actually drives `Err(PhaseError::Cancelled)` or is explicitly deleted with rationale
- No new `ActionOutcome` references. No new public API surface.

## Subagent

AFK task ticket. 2 SP → `general-purpose` subagent. Primary agent verifies output + `build.py` + coverage.
