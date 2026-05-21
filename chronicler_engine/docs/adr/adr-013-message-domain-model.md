# ADR-013: Message Domain Model

## Status

**Accepted** — Implemented 2026-05-15  
**Simplified** — Per-message swipes removed 2026-05-15  
**Further simplified** — Message-aligned snapshots, 2026-05-16

**Supersedes**: ADR-012 (Turn + Swipe Domain Model, deleted)

## Context

ADR-012 introduced a `Turn` + `Swipe` model where one `Turn` grouped a player input with **all** its AI responses (narration, event continuation, dialogue). While this solved the emergent-turn-identity bug, it created a new problem: every AI output within a turn shared the same swipe set. If a trigger fired after the main narration, the event continuation and the main narration were locked together in the same turn's swipe. The user could not retry the narration without also deleting/regenerating the event.

ADR-013 originally proposed `Message` + `MessageSwipe` to solve this by giving each AI output its own swipe set. After implementation, we discovered that the actual retry system worked via **snapshot rollback** (loading `pre-main:{turn_id}` and re-running the pipeline), not via message-level swipes. The `MessageSwipe` fields were dead code that created architectural confusion and caused a bug where retry accidentally wiped input text.

## Decision

We use a flat `Vec<Message>` encapsulated in `MessageHistory` where each `Message` is a single-version narrative unit. Retry is implemented as **snapshot rollback + replay**, not swipe creation.

1. **`Message`** = one narrative unit (input, narration, dialogue, or system). Every `add_log()` call creates a new `Message`.
2. **`Message.text`** = the single text content. No swipes, no versioning at the message level.
3. **`Message.id`** = a monotonically increasing `u64`. Used for log entry identification and edit targeting.
4. **`Message.snapshot_id`** = references the `GameStateSnapshot` saved after this message was created. Used by retry to restore state.
5. **`NarrativeState.history()`** = derived `Vec<LogEntry>` view for templates and prompts.
6. **`Game`** = a top-level playthrough session. Each game has its own isolated snapshots and messages. Named automatically on creation (`{WorldName}_{Date}_N`).

### Retry Behavior

- **Main retry**: Find the last `Input` message, load the snapshot from its `snapshot_id`, delete messages after it, re-run full pipeline. Old narration is discarded; new narration message is created.
- **Event retry**: Find the last non-event message, load the snapshot from its `snapshot_id`, delete event messages after it, re-run trigger continuation. Old event message is discarded; new event message is created.

This is simpler, more robust, and matches how the engine actually works.

### Key Changes from ADR-012

| Area | ADR-012 (Turn) | ADR-013 (Message) |
|------|----------------|-------------------|
| History storage | `Vec<Turn>` | `MessageHistory` (wraps `Vec<Message>`) |
| Turn grouping | `Turn { input, swipes }` | Messages are independent units; no turn grouping |
| AI outputs per unit | Multiple `LogEntry` items per swipe | One `text` per `Message` |
| Retry scope | Entire turn (all AI outputs) | Last message only |
| Delete scope | Entire turn | Last message only |
| Retry mechanism | Snapshot rollback + replay | Snapshot rollback + replay (no swipes) |

## Consequences

### Positive

- **Independent retry per AI output**: The user can retry the last message (narration, event, or dialogue) without affecting earlier messages.
- **Layered deletion**: Deleting removes one message at a time, peeling back layers.
- **No rendering changes**: `history()` derived view preserves all existing template and prompt builder behavior.
- **Snapshot correlation preserved**: Every message stores `snapshot_id` — the snapshot saved after it was created. Retry loads the previous message's snapshot, no synthetic turn IDs needed.
- **Simpler model**: No dead code, no divergence between `text` and `active_swipe.text`, no silent out-of-bounds fallback.

### Negative

- **No backward compatibility**: Existing SQLite databases are not migrated. This is acceptable per project conventions (DBs are recreated on fresh runs).
- **More messages in history**: Where the old model had one turn with 3 entries, the new model has 3 separate messages.

## Alternatives Considered

1. **Keep Turn model, add per-message swipes inside Turn**: Rejected because it overcomplicates the domain.
2. **Fully relational SQLite schema (messages + message_swipes tables)**: Rejected because `NarrativeState` is already serialized as JSON in snapshots. Keeping snapshot JSON intact minimizes migration risk.
3. **Flat `Vec<LogEntry>` with swipe arrays on each entry**: Rejected because `LogEntry` is the atomic rendering unit; adding swipe metadata would pollute the rendering contract.
4. **Message + MessageSwipe (original ADR-013)**: Implemented then removed because the retry system never actually used per-message swipes. Snapshot rollback is simpler and works correctly.

## References

- `src/model/message.rs` — `Message` struct
- `src/model/state.rs` — `NarrativeState` with `MessageHistory`
- `src/model/message_history.rs` — `MessageHistory` encapsulates message lifecycle
- `src/model/state_snapshot.rs` — `GameStateSnapshot` (standalone, no `turn_id` or `base_snapshot_id`)
- `src/application/action_pipeline/retry.rs` — Snapshot-based retry logic
- `src/server/fragments/history.rs` — Message-level mutation handlers
- `docs/system/game_flow.md` — Updated retry flow diagram
