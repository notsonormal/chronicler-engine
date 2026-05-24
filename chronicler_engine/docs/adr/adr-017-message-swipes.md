# ADR-017: Message Swipes

## Status

**Accepted** — Implemented 2026-05-24

**Supersedes**: ADR-013 simplification (swipes reintroduced with dedicated table)

## Context

ADR-013 removed per-message swipes in favor of a simpler snapshot-rollback model. While this worked for basic retry, it had two problems:

1. **Destructive retry**: Old generations were permanently lost. Users could not compare different narrations or return to a previous generation without regenerating.
2. **No event independence**: Events and narrations were tightly coupled. A user could not retry a narration without also losing the event that followed it.

SillyTavern and Marinara both preserve old generations as "swipes" — alternate versions of the same message that the user can switch between. This allows non-destructive retry and A/B comparison of generations.

## Decision

Reintroduce per-message swipes as a dedicated `message_swipes` table, with each swipe storing its own `snapshot_id`. Only the **last message** is swipeable.

### Architecture

1. **`Message.swipes: Vec<Swipe>`** — each AI-generated message has its own swipe set.
2. **`Swipe.snapshot_id: Option<u64>`** — references the `GameStateSnapshot` that produced this swipe's text. Nullable because the initial message (before any snapshot was saved) has no snapshot.
3. **`Message.active_swipe_index: usize`** — the currently displayed swipe. Active fields (`text`, `location_header`, `event_header`, `snapshot_id`) are hydrated from the active swipe at load time.
4. **`message_swipes` table** — dedicated SQLite table with `ON DELETE CASCADE`.
5. **Soft deletes** — during retry, messages after the anchor are soft-deleted. If the pipeline fails, they are restored. If it succeeds, they are hard-deleted.
6. **No turn grouping** — messages remain independent units. Narration and event swipes are completely separate.

### Retry Behavior

- **Main retry**: Soft-delete messages after the last input, preserve the old narration as a swipe, load the input's snapshot, rerun the full pipeline. Old swipes migrate to the new message.
- **Event retry**: Soft-delete event messages, preserve the old event as a swipe, load the pre-event snapshot, rerun trigger continuation.
- **Swipe navigation**: Left/right arrows on the last message switch between swipes. Restoring a swipe loads its snapshot and rewinds world state.

### Retrigger Event

When a narration swipe is restored and its snapshot contains `last_trigger`, a "Retrigger Event" button appears. Clicking it runs `pipeline.run_trigger_continuation()` from that snapshot state, generating a new event continuation without rerunning the main narration.

### Why Only the Last Message?

Swiping a non-last message would require deleting all messages after it (since they depend on the state that the swipe changes). This is equivalent to retry, which already exists. Limiting swipes to the last message avoids history truncation complexity while preserving the core value: comparing alternate versions of the most recent generation.

## Consequences

### Positive

- **Non-destructive retry**: Old generations are preserved as swipes. Users can switch back at any time.
- **State-consistent swipes**: Each swipe stores its own snapshot. Switching swipes restores the exact world state that produced that text.
- **Event independence**: Narration and event swipes are completely separate. Retriggering an event does not affect the narration.
- **Soft-delete safety**: Retry failures do not lose data. Soft-deleted messages are restored on pipeline failure.

### Negative

- **More complex storage**: Two tables (`messages` + `message_swipes`) with JOIN-like loading logic.
- **Migration complexity**: SQLite cannot drop columns, so v6 recreates the `messages` table.
- **Only last message swipeable**: Users must delete subsequent messages to swipe earlier ones.

## Alternatives Considered

1. **Keep ADR-013 snapshot-only model**: Rejected because destructive retry loses user data and prevents comparison.
2. **Graph snapshots (Marinara-style)**: Rejected because it overcomplicates the engine. Per-message swipes with snapshot IDs give the same state consistency without a graph.
3. **Turn grouping with per-turn swipes**: Rejected because it recouples narration and event. Independent message swipes keep them separate.

## Key Changes from ADR-013

| Area | ADR-013 | ADR-017 |
|------|---------|---------|
| Message storage | Flat `messages` with inline text | `messages` + `message_swipes` table |
| Retry mechanism | Snapshot rollback + replay | Snapshot rollback + swipe preservation |
| Old generations | Lost (destructive) | Preserved as swipes |
| Event retry | Deletes event, regenerates | Soft-deletes event, preserves as swipe |
| Swipe navigation | None | Left/right arrows on last message |
| State per swipe | Single `snapshot_id` on message | Each `Swipe` has its own `snapshot_id` |

## References

- `src/model/message.rs` — `Message` and `Swipe` structs
- `src/storage/db.rs` — v6 migration (`message_swipes` table)
- `src/storage/snapshot_storage.rs` — SQLite swipe loading and storage
- `src/application/action_pipeline/retry.rs` — Soft-delete + swipe migration logic
- `src/server/templates.rs` — Swipe controls and retrigger button rendering
- `assets/index.html` — `submitNewSwipe()`, `switchSwipe()`, `submitRetrigger()`
