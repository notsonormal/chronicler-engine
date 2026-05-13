# ADR-012: Turn + Swipe Domain Model

## Status

**Accepted** — Implemented 2026-05-13

## Context

The engine used a flat `Vec<LogEntry>` for narrative history with an **emergent** turn identity encoded only in snapshot `message_id`. This caused a critical bug where history mutation (delete/edit) broke retry because the correlation between a turn and its pre-generation snapshots (`pre-main:{uuid}`, `pre-event:{uuid}`) was purely conventional.

Both `delete_history_handler` and `edit_history_handler` generated new random UUIDs for snapshots, disconnecting them from the pre-generation snapshots that retry relied on. When a user deleted or edited history and then clicked retry, the engine could not find the matching `pre-main` snapshot because the `turn_id` had changed.

Marinara Engine solves this with a relational model (`messages` + `message_swipes` + `game_state_snapshots` tied to `(messageId, swipeIndex)`). Chronicler adopts the same conceptual model.

## Decision

We will introduce first-class `Turn` and `Swipe` domain objects:

1. **`Turn`** = one player input + all its AI responses. Every player action creates a `Turn`.
2. **`Swipe`** = one generation attempt. Each swipe contains the log entries produced by that attempt.
3. **`Turn.id`** = a stable UUID generated at turn creation time. This UUID is used as `turn_id` in all snapshots for that turn.
4. **`NarrativeState.history()`** = derived `Vec<LogEntry>` view that flattens active swipes. All rendering, prompt building, and status checks continue to use this view.
5. **`Checkpoint`** = a bookmark referencing `(turn_id, swipe_index)` stored in a dedicated SQLite table.

### Key Changes

| Area | Before | After |
|------|--------|-------|
| History storage | `Vec<LogEntry>` | `Vec<Turn>` |
| Turn identity | Emergent (snapshot `message_id`) | Structural (`Turn.id`) |
| Snapshot key | `message_id` | `turn_id` |
| Retry correlation | By UUID convention | By `Turn.id` — compiler-enforced |
| Delete handler | `delete_last_log()` | `delete_last_turn()` + cascade snapshot delete |
| Edit handler | Random snapshot UUID | Preserves `turn_id` and `swipe_index` |
| Swipe creation | Implicit (snapshot `swipe_index` only) | Explicit (`Turn.create_swipe()`) |

## Consequences

### Positive

- **Retry works after delete/edit**: Because `turn_id` is tied to `Turn.id`, deleting or editing a turn does not break the correlation with pre-generation snapshots.
- **Swipe browsing**: Multiple swipes per turn are now real domain objects, enabling UI navigation between generation attempts.
- **Checkpoint support**: Checkpoints can reference stable `(turn_id, swipe_index)` pairs.
- **No rendering changes**: `history()` derived view preserves all existing template and prompt builder behavior.

### Negative

- **No backward compatibility**: Existing SQLite databases are not migrated. This is acceptable per project conventions (DBs are recreated on fresh runs).
- **Slightly more complex state**: `NarrativeState` now has nested structure (`Turn` → `Swipe` → `LogEntry`) instead of a flat vector.
- **Event retry copies narration**: When retrying an event continuation, the main narration is copied from the previous swipe to the new swipe. This is correct behavior but adds a small copy cost.

## Alternatives Considered

1. **Keep flat history, fix UUID correlation in handlers only**: Rejected because it leaves the root fragility unaddressed. The emergent turn identity would still be easy to break accidentally.
2. **Store turn ID in `LogEntry`**: Rejected because it complicates the atomic rendering unit. `LogEntry` should remain a simple message struct.
3. **Use a separate `turns` table in SQLite**: Rejected because `NarrativeState` is already serialized as JSON in snapshots. Adding a relational turns table would require synchronizing two storage formats. The in-memory `Vec<Turn>` with JSON snapshot serialization is simpler and consistent with existing patterns.

## References

- `src/model/turn.rs` — `Turn`, `Swipe` structs
- `src/model/state.rs` — `NarrativeState` with `Vec<Turn>`
- `src/model/state_snapshot.rs` — `GameStateSnapshot` with `turn_id`
- `src/engine/game_service/retry.rs` — Swipe-aware retry logic
- `src/server/fragments/history.rs` — Turn-level mutation handlers
- `docs/system/game_flow.md` — Updated retry flow diagram
