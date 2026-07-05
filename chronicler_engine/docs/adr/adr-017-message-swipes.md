# ADR-017: Message Swipes

**Date:** 2026-05-24
**Status:** Accepted

## Context

The earlier snapshot-rollback model (removed during ADR cleanup; superseded by this ADR) removed per-message swipes in favor of a simpler snapshot-rollback model. While this worked for basic retry, it had two problems:

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
5. **Soft deletes** — during retry, messages after the anchor are marked soft-deleted via `Storage::soft_delete_message`. If the pipeline fails, they are restored via `Storage::restore_soft_deleted`. If it succeeds, they are hard-deleted via `Storage::purge_soft_deleted`.
6. **No turn grouping** — messages remain independent units. Narration and event swipes are completely separate.

### Retry Behavior

- **Main retry**: Soft-delete messages after the last input, preserve the old narration as a swipe, load the input's snapshot, rerun the full pipeline. Old swipes migrate to the new message.
- **Event retry**: Soft-delete event messages, preserve the old event as a swipe, load the pre-event snapshot, rerun trigger continuation.
- **Swipe navigation**: Left/right arrows on the last message switch between swipes. Restoring a swipe loads its snapshot and rewinds world state.

### Retrigger Event

When a narration swipe is restored and its snapshot contains `last_trigger`, a "Retrigger Event" button appears. Clicking it runs the trigger continuation flow via `ActionPipeline::phase_trigger_continuation()` → `ActionPipeline::reconcile_post_trigger_npcs()` → `phase_finalize()` from that snapshot state, generating a new event continuation without rerunning the main narration.

### Why Only the Last Message?

Swiping a non-last message would require deleting all messages after it (since they depend on the state that the swipe changes). This is equivalent to retry, which already exists. Limiting swipes to the last message avoids history truncation complexity while preserving the core value: comparing alternate versions of the most recent generation.

### Why per-message swipes over snapshot-only model

The snapshot-only model performed destructive retry: old generations were permanently lost, and retrying a narration invalidated subsequent events. Per-message swipes preserve non-destructive retry and keep narration/event retry independent.

### Why per-message swipes over graph snapshots (Marinara)

Marinara-style graph snapshots would provide equivalent state consistency but require general-purpose graph state management. Per-message swipes with per-swipe `snapshot_id` fields give the same state-consistent swiping without that overhead.

### Why per-message swipes over turn grouping

Turn grouping with per-turn swipes would re-couple narration and event swipes, undoing the event independence requirement. Independent per-message swipes keep narration and event flows separate.

## Consequences

### Positive

- **Non-destructive retry**: Old generations are preserved as swipes. Users can switch back at any time.
- **State-consistent swipes**: Each swipe stores its own snapshot. Switching swipes restores the exact world state that produced that text.
- **Event independence**: Narration and event swipes are completely separate. Retriggering an event does not affect the narration.
- **Soft-delete safety**: Retry failures do not lose data. Soft-deleted messages are restored on pipeline failure.

### Negative

- **More complex storage**: Two tables (`messages` + `message_swipes`) with JOIN-like loading logic.
- **Only last message swipeable**: Users must delete subsequent messages to swipe earlier ones.

### Trade-offs
- Chose two-table design over single-table with versioning (clearer soft-delete semantics; JOIN cost acceptable)
- Chose per-swipe snapshot_id over shared snapshot (state-consistent swiping won over storage)
- Chose soft-delete over hard-delete on retry (recovery safety won over storage simplicity)

## Related ADRs

- [ADR-008: SQLite Snapshot Persistence](./adr-008-sqlite-snapshot-persistence.md) — supplies the `GameStateSnapshot` referenced by each swipe.

