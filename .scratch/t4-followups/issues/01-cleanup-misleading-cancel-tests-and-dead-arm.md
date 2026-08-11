Type: task
Story points: 2
Blocked by: (none)

# Ticket 01 — Cleanup misleading cancel-test plumbing + dead arm surfaced by ticket 04 coverage pass

Per [T4 followups map](../map.md): the post-plan-workflow coverage pass on ticket 04 surfaced 3 sharp cleanup items. The coverage work used a `thread::spawn` + `with_trigger_delay` + `app.set_game_id` flip to drive `Err(PhaseError::Cancelled)` through `retry_event_continuation` — because the apparent cancel-token check in `phase_trigger_continuation_with_cancel_handling` doesn't actually consult `app.cancel_token()`. That made the Cancelled-coverage tests unnecessarily indirect AND exposed that 2 pre-existing tests "pass for the wrong reason".

## Question

What does the focused cleanup of these 3 items look like, and does it preserve every existing test contract?

## Scope

### Item 1 — `phase_trigger_continuation_with_cancel_handling` name lies

**Today:** `phases.rs::phase_trigger_continuation_with_cancel_handling` is named as if it handles cancellation via the `CancellationToken`. It does not. Its only cancel-equivalent path is the in-phase α-check `check_game_unchanged(started_for)` — i.e., a game-id mismatch. The `CancellationToken` is never consulted inside this function. That's why the ticket-04 Cancelled-coverage tests had to use `thread::spawn` + `with_trigger_delay(200)` + mid-pipeline `app.set_game_id` flip to drive `Err(PhaseError::Cancelled)` through the retry path.

**Decide (do NOT silently pick — surface tradeoff in the resolution, but pick one to ship):**
- **(A) Rename** to `phase_trigger_continuation_with_game_change_handling` (honest name; no behavior change). Update its 2 callers (`run_from_input` in `pipeline.rs`, `retry_event_continuation` in `retry.rs`). Add a `///` doc comment naming the α-check contract (game-id mismatch only, not shutdown-token).
- **(B) Make it actually check the token** — consult `app.cancel_token().is_cancelled()` at the entry of the function AND at the post-LLM-call boundary; return `Err(PhaseError::Cancelled)` on either. This is a behavior change with broader blast radius (any other caller expecting only α-check semantics). Surface the test deltas explicitly.
- **(C) Rename + document the gap as a separate ticket** (defer the token-integration work to a future cancellation-mechanics effort).

**Recommendation: (A) rename + doc.** The cancellation-via-token propagation is a larger mechanics change that belongs in its own effort. The rename + doc makes the lie visible in the code today; later cancellation work can pick up the documented gap. Mark (C) as out-of-scope for this ticket once you pick (A) — it's just the deferral of (B), not a separate ticket.

### Item 2 — 2 misleading tests pass for the wrong reason

**Today:**
- `retry_tests.rs::test_retry_event_continuation_cancels_before_llm` (around line 265-298 of the current file — re-locate if line shifted)
- `retry_tests.rs::test_retry_event_impl_cancels_cleanly` (around line 672-748)

Both tests claim to verify cancellation, but actually rely on `phase_finalize` resetting status to `Idle` after a successful pipeline run. They never drive a real `Err(PhaseError::Cancelled)`; the `Idle` assertion is a no-op that would pass even if cancellation were broken. **This is misleading-documentation test code.**

**Decide:**
- **(A) Rewrite** both tests using the `thread::spawn` + `with_trigger_delay` + `app.set_game_id` flip pattern that the ticket-04 coverage tests established. The rewrite should assert that `status == Idle` AND that the pipeline actually took the Cancelled path (e.g., assert the narrator LLM was NOT called past the cancel point — mirror how `test_retry_last_response_impl_cancelled_at_phase_boundary` does it). Preserve the test intent, not the implementation.
- **(B) Delete** both tests if the rewrite is too invasive AND there's already sufficient Cancelled-coverage from the new ticket-04 tests (`test_retry_last_response_impl_cancelled_at_phase_boundary`, `test_retrigger_event_impl_cancelled_at_phase_boundary`). If deleting, document in the resolution why the new tests subsume the old ones.

**Recommendation: (A) rewrite.** The 2 tests live at a higher level (`retry_event_continuation` + `retrigger_event_impl` directly) than the 2 new ticket-04 tests (which drive `retry_last_response_impl` / `retrigger_event_impl` via the orchestrator match). Different test levels catch different regressions. Rewriting preserves the contract; deleting loses the higher-level coverage.

### Item 3 — Dead arm at `retry.rs:79-82` (no-input-to-retry)

**Today:** Inside `retry_last_response_impl`, after the `let snapshot = ...; let mut state = GameState::from_snapshot(&snapshot);` block:

```rust
let input_text = match state.narrative.history.last_input_text() {
    Some((_, text)) => text,
    None => {
        retry_persist_error(app, "Retry failed: no input to retry");
        return;
    }
};
```

Coverage analysis (subagent 2): this arm is **unreachable by construction**. `find_retry_anchor` always returns an anchor whose `snapshot_id` resolves to a snapshot whose history contains an Input message. And earlier in the function `truncated.truncate(anchor_idx + 1)` keeps everything up to and including the anchor — the Input is always in the truncated history. So `last_input_text()` always returns `Some`.

**Decide:**
- **(A) Delete the arm** and replace the `match` with a direct destructure: `let (_, input_text) = state.narrative.history.last_input_text().expect("retry anchor guarantees an Input in history");`. Add a `// SAFETY`-style comment if appropriate (but see AGENTS.md — no "What" comments; rename or document at the anchor/persistence level instead).
- **(B) Keep the arm** as a defensive guard (defense-in-depth). The cost is one uncovered branch; the benefit is graceful handling if `find_retry_anchor`'s contract ever loosens.
- **(C) Replace with an invariant assertion** that fires loudly if the assumption ever breaks, instead of silent `retry_persist_error`. E.g., `.unwrap_or_else(|| panic!("invariant violated: retry anchor did not yield an Input message"))`.

**Recommendation: (A) or (C).** (B) keeps the dead code visible at the cost of a permanent coverage hole. If the anchor contract ever loosens, the panic from (C) surfaces immediately during development rather than silently degrading in production. Pick based on how confident you are in the `find_retry_anchor` contract — read the function and its tests before deciding.

## Out of scope

- **Cancellation-via-token propagation through all phases.** Today only the α-check (game-id mismatch) produces `Err(PhaseError::Cancelled)` from `phase_trigger_continuation_with_cancel_handling`. Propagating `app.cancel_token()` through every phase is a separate, larger mechanics effort — not in scope here. If you pick Item 1 option (A) rename, add this to the map's **Out of scope** section.
- Anything outside `phases.rs` (rename), `retry_tests.rs` (test rewrite), `retry.rs` (dead arm removal).
- No `run_from_input` changes. No new public API. No `ActionPipeline` surface changes.

## Blast radius

- `src/application/action_pipeline/phases.rs` — rename `phase_trigger_continuation_with_cancel_handling` (Item 1 option A) and update the 2 call sites in `pipeline.rs` + `retry.rs`. ~5-10 LOC.
- `src/application/action_pipeline/retry_tests.rs` — rewrite or delete 2 tests. ~50-100 LOC.
- `src/application/action_pipeline/retry.rs` — remove or replace dead arm at lines 79-82. ~3-5 LOC.

## Validation

- `python build.py` green (all 8 steps)
- `cargo llvm-cov nextest --features testing --lib retry_tests` → retry.rs coverage stays ≥ 80% (target: no regression from current 97.18%; acceptable transient drop to ≥ 82% if rewrite changes line counts)
- All existing test contracts preserved — specifically the 5 ticket-04 coverage tests (`test_retry_last_response_impl_cancelled_at_phase_boundary`, `test_retrigger_event_impl_cancelled_at_phase_boundary`, `test_retrigger_event_impl_emits_error_on_world_fetch_failure`, `test_retry_event_continuation_handles_state_without_input_message`, `test_retry_records_missing_snapshot_id`)
- `grep phase_trigger_continuation_with_cancel_handling src/ tests/` → 0 matches if Item 1 option (A) taken; otherwise document that it's still named misleadingly
- No new `ActionOutcome` references. No new public API surface.

## Subagent

AFK task ticket. 2 SP → `general-purpose` subagent. Primary agent verifies output + `build.py` + coverage.
