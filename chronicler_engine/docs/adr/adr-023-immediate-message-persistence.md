# ADR-023: Immediate Message Persistence

**Date:** 2026-05-31  
**Status:** Accepted  
**Replaces:** N/A  
**Superseded by:** N/A

## Context

The original architecture used a two-phase commit pattern for message persistence:

1. Messages were created with `UNPERSISTED_ID` sentinel (value: 0)
2. Multiple messages could accumulate in `GameState.narrative.history`
3. Periodically, `flush_messages()` was called to persist all unpersisted messages
4. Snapshots had a `committed: bool` flag to track whether they included all messages up to that point
5. `save_committed_state()` saved snapshots with `committed = true`
6. `SnapshotStorage::commit()` marked snapshots as committed

This pattern emerged to support retry semantics: retries needed an anchor snapshot that included all messages up to the retry point.

### Problems

1. **Batching complexity**: Messages and snapshots could diverge, requiring careful coordination
2. **UNPERSISTED_ID sentinel**: Magic value `0` scattered across 5+ files (`context.rs`, `bootstrap/run.rs`, `application_service.rs`, etc.)
3. **Committed flag confusion**: Snapshots could exist in "uncommitted" state, requiring `load_latest_committed()` to find safe restore points
4. **Frontend polling pressure**: To see new messages quickly, frontend had to poll aggressively (1s intervals)
5. **Flush timing**: Deciding when to call `flush_messages()` was error-prone (too early = missing messages, too late = stale snapshots)

## Decision

Persist messages **immediately** when created, alongside their corresponding snapshot. Each message gets exactly one snapshot association. No batching, no `UNPERSISTED_ID`, no `committed` flag.

### Core Invariant

> **When a message is created, it is persisted immediately alongside a snapshot.**

### Implementation

#### Helper Function

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

This function:
1. Creates a snapshot from current state
2. Saves the snapshot, obtaining `snapshot_id`
3. Persists the last (newest) message with `snapshot_id` association
4. Because messages are persisted immediately, there is never more than one unpersisted message at a time

#### Ordering

**Snapshot first, then message**: This allows assigning the `snapshot_id` association in a single pass. The snapshot represents the state when the message was created.

### Changes

#### 1. Remove `committed` Flag

- Delete `committed: bool` from `GameStateSnapshot` struct
- Delete `save_committed_state()` from `context.rs`
- Delete `SnapshotStorage::commit()` from trait and all implementations
- Update database migration to drop `committed` column from `snapshots` table
- Update all tests that assert on `committed` flag

#### 2. Remove `UNPERSISTED_ID` Pattern

- Delete `UNPERSISTED_ID` constant from `model/message.rs`
- Delete `persist_new_messages()` / `flush_messages()` from `context.rs`
- Remove manual `UNPERSISTED_ID` loops in:
  - `bootstrap/run.rs` (scenario injection)
  - `server/fragments/misc.rs` (reset handler)
- Replace `msg.id == 0` checks with `msg.is_unpersisted()` helper method

#### 3. Pipeline Integration

**`application/action_pipeline/pipeline.rs`**:

- `phase_pre_main_snapshot`: After setting status/phase, call `save_message_and_snapshot` (persists input message + saves pre-main snapshot)
- After `phase_post_generation`: If history grew, call `save_message_and_snapshot` (persists system log + saves snapshot)
- After `phase_engine_commit`: Call `save_message_and_snapshot` (persists narration + saves post-engine snapshot with `last_trigger`)
- Inside `phase_trigger_continuation`, after `commit_trigger_narration`: Call `save_message_and_snapshot` (persists event message + saves post-event snapshot)
- `phase_finalize`: Call `save_state()` as final safety-net (saves snapshot only, messages already persisted)

**Error paths**: Any error handler that calls `add_log()` should also call `save_message_and_snapshot` to persist the log message.

#### 4. Bootstrap and Reset Handler

- `bootstrap/run.rs`: After `inject_scenario_logs`, call `save_message_and_snapshot` for each scenario message
- `server/fragments/misc.rs` (reset handler): After `add_log()` for scenario injection, call `save_message_and_snapshot`

#### 5. Frontend Polling

Revert aggressive polling introduced to compensate for batch delays:

- `assets/index.html`: Story log polling back to 2s (from 1s)
- Status polling back to 5s
- Remove idle refresh trigger

## Consequences

### Positive

1. **Simpler mental model**: One message → one snapshot, always in sync
2. **No magic sentinels**: `UNPERSISTED_ID` eliminated, replaced with `is_unpersisted()` helper
3. **Committed flag unnecessary**: No more "uncommitted" snapshots; all snapshots are immediately valid
4. **Retry semantics preserved**: 
   - Main narration retry: anchor = input message → `snapshot_id` = pre-main snapshot → pipeline re-runs from this state ✓
   - Event retry: anchor = narration message → `snapshot_id` = post-engine snapshot (has `last_trigger` set because `phase_engine_commit` evaluates triggers before saving) ✓
5. **Frontend responsiveness**: Messages appear immediately without aggressive polling pressure
6. **No flush timing decisions**: Removed entire class of bugs around "when to flush"

### Negative

1. **More database writes**: Each message persists immediately (previously batched), but this is negligible compared to snapshot writes
2. **Migration cost**: All call sites using `UNPERSISTED_ID`, `flush_messages()`, or `committed` flag must be updated
3. **Breaking change**: Database schema change requires migration (drop `committed` column)

### Files Changed

- `model/message.rs` — Remove `UNPERSISTED_ID`, add `is_unpersisted()` helper
- `model/state_snapshot.rs` — Remove `committed: bool` field
- `application/context.rs` — Add `save_message_and_snapshot()`, remove `flush_messages()`, `save_committed_state()`
- `application/context_tests.rs` — Update tests for new persistence pattern
- `application/action_pipeline/pipeline.rs` — Call `save_message_and_snapshot` at each phase boundary
- `storage/db.rs` — Update schema (drop `committed` column)
- `storage/snapshot_storage.rs` — Remove `commit()` from trait
- `storage/mappers/state_snapshot.rs` — Remove `committed` column mapping
- `test_support/in_memory_storage.rs` — Remove `commit()` implementation
- `bootstrap/run.rs` — Use `save_message_and_snapshot` for scenario messages
- `server/fragments/misc.rs` — Update reset handler
- `assets/index.html` — Revert polling intervals

## Compliance

All message creation MUST:
- Persist messages immediately via `save_message_and_snapshot()`
- Never use `UNPERSISTED_ID` sentinel
- Never call `flush_messages()` or `persist_new_messages()`

All snapshots MUST:
- NOT have a `committed` field
- Be immediately valid for restore operations
- Be saved via `save_message_and_snapshot()` or `save_state()` (safety-net only)
