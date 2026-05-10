# Plan: Fix Server-Side Action Race Condition

## Problem

`process_action` in `src/server/fragments/actions.rs` performs `load_state() → add_log → save()` without any lock. Two concurrent requests can read the same snapshot, modify independently, and the second save overwrites the first. This causes `test_double_submit_protection` flakiness.

`GameServiceContext::action_lock` exists for serialization but is created fresh per-request in `as_game_service_context()`, so it does not actually coordinate anything.

## Context from Multi-Agent Architecture Spec

- **Assumption #1**: Single-player only. No concurrent players.
- **Phase 1.7**: Replaced `Arc<Mutex<GameState>>` with `Arc<SnapshotStorage>` + cached world data.
- **Risk table**: "Mutex removal causes races" — mitigation was "No shared mutable state = no races; test concurrent requests." This mitigation is incomplete: SQLite snapshots are shared mutable state at the application level. The read-modify-write cycle in `process_action` is the race site.

## What the Reviews Said (and Didn't Say)

All 8 review documents in `docs/reviews/` were read. **None of them identified this race condition.**

- **Phase 2 (Structural Forces)** and **Holistic Review** praised the old `Arc<Mutex<GameState>>` pattern: "excellent lock-to-unblock ratio," "no deadlock risk," "locks are brief." They were written before/during the snapshot migration and focused on the **old** mutex-based architecture.
- **Defensive Architecture Review** explicitly states: "No Mutex Contention Changes. The `Arc<Mutex<GameState>>` pattern is unchanged." This was accurate at the time but predates Phase 1.7.
- **Evolution Stress Test** discusses multiplayer mutex bottlenecks but assumes the old centralized mutex still exists.

**Key insight:** The old `Arc<Mutex<GameState>>` **did** serialize all state access. When Phase 1.7 removed it in favor of SQLite snapshots, the application-level serialization was lost. The reviews never got updated to account for the new snapshot-based race. This is a **regression introduced by the Phase 1.7 migration**, not a pre-existing bug.

## Root Cause

1. `AppState` has no action lock.
2. `as_game_service_context()` creates a **new** `Arc<Mutex<()>>` each call.
3. Handler does unprotected read-modify-write.
4. Background tasks each have their own lock — no serialization.

## Rethinking the Approach

The user raises two excellent points:

1. **Double-submit protection should start in the UI.** A well-designed web app disables the submit button after click. Server-side protection is a safety net, not the primary defense.
2. **There should be a "Generating" lock.** If the LLM is still narrating the previous turn, the server should reject new actions. This is semantically correct for a text adventure — you can't take a new turn while the GM is still speaking.

These insights point to a **better fix** than a generic action mutex.

## Better Fix: Generation Gate + UI Protection

Instead of serializing all actions behind a mutex, prevent concurrent async actions at the domain level.

### Server-Side: Atomic Generation Gate

Use an `AtomicBool` in `AppState` as a fast, lock-free gate:

```rust
pub is_generating: AtomicBool,
```

In `process_action`:
```rust
// Only FreeAction triggers LLM generation. Sync actions bypass the gate.
if !is_sync {
    if state.is_generating.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("<span class=\"status wait\">Still thinking...</span>"))
            .expect("static response body is valid");
    }
}
```

The background task clears the flag when done (success, error, or cancellation). If the task panics, a timeout or guard should clear it.

**Why this is better than a mutex:**
- Uses domain semantics ("can't act while generating") instead of low-level synchronization.
- Rejects the second request immediately instead of queueing it.
- No lock contention, no poison risk, no lifetime issues.
- Aligns with single-player design: one turn at a time.

### The Remaining Race

Even with a generation gate, two concurrent requests could both pass `compare_exchange` if they execute it simultaneously before either saves. The probability is extremely low (single instruction), but not zero.

To close this: wrap the `load_state → check → set_generating → save` sequence in the **existing** `action_lock` (a brief `std::sync::Mutex` hold). The lock is only needed for microseconds — just long enough to atomically check-and-set.

### Client-Side: Disable Button During Request

The HTMX action form should disable the submit button while a request is in flight. This prevents the vast majority of double-submits before they reach the server.

**Implementation:** Add `hx-indicator` and a small script, or use HTMX's `htmx:configRequest` event to disable the button, and `htmx:afterRequest` to re-enable.

### Changes

1. **`src/server/mod.rs` — `AppState`**: add `pub is_generating: AtomicBool`.
2. **`src/server/fragments/actions.rs` — `process_action()`**:
   - For async actions: `compare_exchange` on `is_generating`. If busy, return "Still thinking...".
   - Proceed with `load → add_log → save` (the micro-race here is acceptable; if hit, both requests see Idle and both spawn, but the generation gate + brief lock prevents this).
3. **`src/engine/game_service/actions.rs`**: clear `is_generating` at all exit points of `execute_action_impl` (success, error, empty response, early return).
4. **UI/templates**: disable submit button during HTMX request.

### Test Impact

`test_double_submit_protection` currently asserts both commands appear. With generation gating, the second request would be rejected. The test should be updated to:
- Assert the second request returns "Still thinking..." or similar.
- Or: assert only the first command's narration appears in the final log.

## Verification

1. `test_double_submit_protection` updated and passes reliably.
2. `build.py` passes clean.
3. Manual check: rapid-clicking the action button shows "Still thinking..." for the second click.

## Decision

Implement the **generation gate + UI protection** approach. It is semantically correct, simpler than a mutex, and fixes the race at the domain level rather than the synchronization level.
