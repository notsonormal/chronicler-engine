# 05 — Split MessageService by concern

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to splitting `MessageService` — currently 13 public methods
spanning snapshot, message, swipe, retry-anchor, and fresh-state — into a deep
atomic message+snapshot+swipe module plus focused sibling modules — and if so,
what is the shape of each?

## Background

This is **candidate 4** of the architecture review. See
`assets/architecture-review.html` for the cross-section diagram and evidence.

The friction: `MessageService` (`message_service.rs`, 267 lines) declares 12
public methods (13 on 2026-08-16) across four concerns. Callers use disjoint subsets, so the interface
is wider than any single caller needs. `save_message_and_snapshot`
(starts at `:62`) bundles four persistence operations with domain ordering
(snapshot write, retry-target swipe update, unpersisted message insert, swipe
insert). `retry.rs` (lines 71 and 203) bypasses the service entirely to reach
`storage.load_snapshot_by_id` — a leak across the seam.

The deletion test is *partial*: the atomic write is real domain logic and
would leak into callers if removed; retry-anchor and swipe switching are
largely independent.

## What this ticket resolves

- **Commit or reject.** Does the 13-method interface earn its breadth, or do
  the concerns deserve their own modules?
- **Interface shape.** The deep atomic-write module's interface (the
  snapshot→message→swipe ordering stays internal); the RetryAnchor and
  SwipeSwitcher siblings' interfaces.
- **Seam leak.** Whether `retry.rs` reaching into storage is fixed by the
  split, or needs a separate decision.
- **What survives.** Which tests cross the atomic-write interface unchanged.

## Constraints

- Must preserve the snapshot ordering invariant (CONTEXT.md: every snapshot is
  immediately valid for restore, message-aligned, persisted with its Message).
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`.
- Domain terms: Message, Swipe, Snapshot (CONTEXT.md).
- Related: ticket 10 (arrival narration) — `ArrivalTaskContext` calls
  `add_message` + `save_message_and_snapshot` directly and falls back to
  `build_fresh_initial_state`, so it is a second caller of the atomic write.
- Related: ticket 09 — `message_service.rs` is one of the 8 files that write
  `input_buffer.status` / `.phase` by hand.

## Current state (2026-10-04)

Still present, roughly unchanged. 12 public methods (was 13) spanning
snapshot, message, swipe, retry-anchor and fresh-state.
`save_message_and_snapshot` still bundles the four operations. The retry leak
is still there: `retry.rs:71` and `retry.rs:203` call
`self.storage.load_snapshot_by_id`.
