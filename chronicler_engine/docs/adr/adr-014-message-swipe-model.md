# ADR-014: Message + Swipe Domain Model

## Status

**Accepted** — Implemented 2026-05-15

**Supersedes**: [ADR-012: Turn + Swipe Domain Model](adr-012-turn-swipe-model.md)

## Context

ADR-012 introduced a `Turn` + `Swipe` model where one `Turn` grouped a player input with **all** its AI responses (narration, event continuation, dialogue). While this solved the emergent-turn-identity bug, it created a new problem: every AI output within a turn shared the same swipe set. If a trigger fired after the main narration, the event continuation and the main narration were locked together in the same turn's swipe. The user could not retry the narration without also deleting/regenerating the event.

SillyTavern and Marinara Engine solve this by making each AI output its own independent message with its own swipes. Chronicler adopts the same conceptual model.

## Decision

We will replace `Turn` and `Swipe` with `Message` and `MessageSwipe`:

1. **`Message`** = one narrative unit (input, narration, dialogue, or system). Every `add_log()` call creates a new `Message`.
2. **`MessageSwipe`** = one generation attempt for a specific message. Each swipe contains the text content of that attempt.
3. **`Message.id`** = a monotonically increasing `u64`. Used for log entry identification and edit targeting.
4. **`Message.turn_id`** = a stable UUID generated when the user submits input. All messages created between user inputs share the same `turn_id`, preserving snapshot correlation.
5. **`NarrativeState.history()`** = derived `Vec<LogEntry>` view that maps active message swipes to `LogEntry` structs. All rendering, prompt building, and status checks continue to use this view.
6. **`Checkpoint`** = still a bookmark referencing `(turn_id, swipe_index)` stored in a dedicated SQLite table.

### Key Changes from ADR-012

| Area | ADR-012 (Turn) | ADR-014 (Message) |
|------|----------------|-------------------|
| History storage | `Vec<Turn>` | `Vec<Message>` |
| Turn grouping | `Turn { input, swipes }` | Messages share `turn_id`; each is independent |
| AI outputs per swipe | Multiple `LogEntry` items (narration + event + dialogue) | Single `text` per `MessageSwipe` |
| Retry scope | Entire turn (all AI outputs) | Last message only |
| Delete scope | Entire turn | Last message only |
| Swipe creation | `Turn.create_swipe(index)` | `Message.create_swipe(text)` |
| Swipe switching | Per-turn (all messages in turn switch together) | Per-message (last message only) |

## Consequences

### Positive

- **Independent retry per AI output**: The user can retry the last message (narration, event, or dialogue) without affecting earlier messages in the same turn.
- **Layered deletion**: Deleting removes one message at a time, peeling back layers. To retry narration when an event exists, delete the event first.
- **Simpler swipe model**: Each swipe is just a text string, not a `Vec<LogEntry>`.
- **No rendering changes**: `history()` derived view preserves all existing template and prompt builder behavior.
- **Snapshot correlation preserved**: `turn_id` on each message ensures `pre-main:{turn_id}` and `pre-event:{turn_id}` snapshots still correlate correctly.

### Negative

- **No backward compatibility**: Existing SQLite databases are not migrated. This is acceptable per project conventions (DBs are recreated on fresh runs).
- **More messages in history**: Where the old model had one turn with 3 entries, the new model has 3 separate messages.
- **UI swipe navigation is per-message**: The action area swipe counter now reflects the last message's swipes, not the whole turn's.

## Alternatives Considered

1. **Keep Turn model, add per-message swipes inside Turn**: Rejected because it overcomplicates the domain. If each message needs its own swipes, making Message first-class is cleaner.
2. **Fully relational SQLite schema (messages + message_swipes tables)**: Rejected because `NarrativeState` is already serialized as JSON in snapshots. Keeping snapshot JSON intact minimizes migration risk.
3. **Flat `Vec<LogEntry>` with swipe arrays on each entry**: Rejected because `LogEntry` is the atomic rendering unit; adding swipe metadata would pollute the rendering contract.

## References

- `src/model/message.rs` — `Message`, `MessageSwipe` structs
- `src/model/state.rs` — `NarrativeState` with `Vec<Message>`
- `src/model/state_snapshot.rs` — `GameStateSnapshot` with `turn_id`
- `src/engine/game_service/retry.rs` — Message-level retry logic
- `src/server/fragments/history.rs` — Message-level mutation handlers
- `docs/system/game_flow.md` — Updated retry flow diagram
