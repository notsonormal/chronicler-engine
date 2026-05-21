# Plan: Immediate Message Persistence — Direct Implementation

## Core Principle

When a message is created, it is persisted **immediately** alongside a snapshot. No `UNPERSISTED_ID`, no `flush_messages`, no batching. Each message gets exactly one snapshot.

## Ordering

Snapshot is saved first, then message is persisted with `snapshot_id`. This lets us assign the association in one pass.

## Helper Function

Add to `application/context.rs`:

```rust
pub fn save_message_and_snapshot(
    ctx: &GameServiceContext,
    state: &mut GameState,
) -> Result<u64, EngineError> {
    let snapshot = GameStateSnapshot::from_game_state(state);
    let snapshot_id = ctx.snapshot_storage.save(&snapshot)?;
    if let Some(msg) = state.narrative.history.last_mut() {
        if msg.id == 0 {
            msg.snapshot_id = Some(snapshot_id);
            ctx.message_storage.insert_message(msg)?;
        }
    }
    Ok(snapshot_id)
}
```

This saves a snapshot and persists only the last (newest) message. Because messages are persisted immediately, there is never more than one unpersisted message at a time.

## Changes

### 1. Remove `committed` flag and `save_committed_state`
- Delete `committed: bool` from `GameStateSnapshot`
- Delete `save_committed_state()` from `context.rs`
- Delete `SnapshotStorage::commit()` from trait and impls
- Update DB migration to drop `committed` column
- Update all tests that assert `committed`

### 2. Add `save_message_and_snapshot` to context
- New function in `application/context.rs`
- `save_state()` stays as a safety-net (saves snapshot, no-op on messages since they're already persisted)

### 3. Update pipeline to persist messages immediately
**`application/action_pipeline/pipeline.rs`:**

- `phase_pre_main_snapshot`: after setting status/phase, call `save_message_and_snapshot` (persists input + saves pre-main snapshot)
- After `phase_post_generation`: if history grew, call `save_message_and_snapshot` (persists system log + saves snapshot)
- After `phase_engine_commit`: call `save_message_and_snapshot` (persists narration + saves post-engine snapshot with `last_trigger`)
- Inside `phase_trigger_continuation`, after `commit_trigger_narration`: call `save_message_and_snapshot` (persists event + saves post-event snapshot)
- `phase_finalize`: call `save_state` (saves final snapshot, messages already persisted)

Error paths that call `add_log` also need `save_message_and_snapshot`.

### 4. Remove `UNPERSISTED_ID`
- Delete from `model/message.rs`
- Delete `persist_new_messages` from `context.rs`
- Remove manual `UNPERSISTED_ID` loops in `bootstrap/run.rs` and `server/fragments/misc.rs`

### 5. Update bootstrap and reset handler
- `bootstrap/run.rs`: after `inject_scenario_logs`, call `save_message_and_snapshot` for scenario messages
- `server/fragments/misc.rs` (reset_handler): after `add_log` for scenario, call `save_message_and_snapshot`

### 6. Revert frontend polling
- `assets/index.html`: story-log back to 2s, status back to 5s, remove idle refresh trigger

### 7. Full validation
- `cargo fmt`, `cargo clippy`, all tests, coverage ≥ 80%

## Why Retry Still Works

- **Main narration retry**: anchor = input message → `snapshot_id` = pre-main snapshot. Pipeline re-runs from this state. ✓
- **Event retry**: anchor = narration message → `snapshot_id` = post-engine snapshot. This snapshot has `last_trigger` set (because `phase_engine_commit` evaluates triggers before saving). ✓

No `committed` flag needed. No `load_latest_committed` needed.

## Files Touched

- `model/message.rs`
- `model/state_snapshot.rs`
- `application/context.rs`
- `application/context_tests.rs`
- `application/action_pipeline/pipeline.rs`
- `storage/db.rs`
- `storage/snapshot_storage.rs`
- `storage/mappers/state_snapshot.rs`
- `test_support/in_memory_storage.rs`
- `bootstrap/run.rs`
- `server/fragments/misc.rs`
- `assets/index.html`
