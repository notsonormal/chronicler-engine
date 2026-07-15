---
diataxis: reference
title: Message Model
---

> **Diátaxis mode:** Reference. This document describes the message aggregate as it is: how `Message`, `Swipe`, and `MessageHistory` relate, the read/write contract for content fields, and the encapsulation rules that prevent bypass. The problem it solves for the reader is *look-up*: where do content fields live, how does one mutate them, and which surface is intentional. Accessor signatures live in `src/domain/model/message.rs`; persistence schema lives in `src/adapters/driven/storage/db.rs` and `./data_layer.md`.

## Overview

The message aggregate has three pieces:

- **`Message`** — a single narrative unit. Holds the message identity plus a `Vec<Swipe>` and an `active_swipe_index`.
- **`Swipe`** — one alternative generation for a `Message`. Holds the actual content fields (`text`, `location_header`, `event_header`, `snapshot_id`); `Message` itself carries none of these.
- **`MessageHistory`** — the ordered collection that owns `Vec<Message>`. Exposes intent-named methods to prevent `.push()` bypass.

## Component Locations

- **`Message` + `Swipe`** — `src/domain/model/message.rs`. One `Message` owns its `Vec<Swipe>` and `active_swipe_index`; one `Swipe` holds the content fields.
- **`MessageHistory`** — `src/domain/model/message_history.rs`. Owns `Vec<Message>`; exposes intent-named methods only.
- **`MessageType` + `MessageEntry`** — `src/domain/model/state/message_types.rs`. `MessageType` discriminates the four message kinds (`Narration`, `Dialogue`, `System`, `Input`); `MessageEntry` is the view-model DTO that templates consume, built via `From<&Message>`.

## Read and Write Contract

Reads use `Message::text()`, `Message::location_header()`, `Message::event_header()`, `Message::snapshot_id()`, and `Message::swipe_count()`. Each routes through the active swipe via the private `active_swipe` accessor. The empty-history case returns an empty string or `None` as appropriate.

Writes use `set_active_swipe(index)`, `update_active_swipe_text(new)`, and `set_snapshot_id(sid)`. Each mutates `swipes[active_swipe_index]` directly. `update_active_swipe_text` is the only sanctioned way to change a message's text in place.

`iter_mut()` is the one method on `MessageHistory` that allows callers to mutate `Message` directly. It exists for swipe navigation (`set_active_swipe`) and inline edits (`update_active_swipe_text`). Every other history method either returns immutable views or wraps the mutation in an intent-named method.

## MessageHistory Encapsulation

`MessageHistory` owns `Vec<Message>` and exposes intent-named methods. The public surface is wide enough that some methods bypass the per-method FIFO cap (see "FIFO Cap" below).

The intent-named methods cover the operations the engine actually performs: `append` (with eviction), `edit` (by id), `delete_last`, `last_input_text`, `last_ai_response_index`, `is_last_ai_response_event_continuation`, and view-DTO conversion via `to_message_entries`. Bulk operations are explicit: `from_messages` and `replace` exist for hydration and wholesale replacement respectively.

## Retry and Swipe Behaviour

Retry replaces the last AI message's active swipe with a new swipe carrying the new generation. The old swipe is preserved (non-destructive). Swipe navigation (`set_active_swipe`) on the last message lets the user compare generations.

Swiping is limited to the **last message**. Each `Swipe` carries its own `snapshot_id`; switching swipes restores the `GameStateSnapshot` that produced that swipe's text, with no mismatch between displayed text and underlying state.

Event independence: narration swipes and event swipes are completely separate. Retriggering an event does not affect the narration. The retry flow distinguishes by `is_event = last().event_header().is_some()`.

## FIFO Cap

The message history is bounded by a FIFO cap. The `append` operation enforces it: if the history is at the cap, evict the head before pushing the new message. This is the FIFO eviction policy.

**Cap bypass.** `from_messages` and `replace` do not enforce the cap. Storage loaders use `from_messages` to hydrate from the database; if the database ever contains more entries than the cap (for example, after a downgrade + upgrade cycle), `from_messages` accepts them all. Soft-delete restoration does not truncate.

## Persistence Notes

The schema-level storage lives in `src/adapters/driven/storage/db.rs`; table relationships and the non-FK `snapshot_id` rationale are documented in `./data_layer.md`. The load-bearing split — identity fields (`id`, `sender`, `message_type`, `timestamp`, `active_swipe_index`, `is_deleted`) live on `messages`, content fields (`text`, `location_header`, `event_header`, `snapshot_id`) live on `message_swipes` — mirrors the "Swipe is the Sole Holder of Content Fields" invariant above. The per-row DDL is not restated here.

Two message-specific observations the schema does not say directly:

- **`Swipe::snapshot_id` is nullable.** The initial message (before any snapshot was saved) has no snapshot. Nullable at the schema level.
- **Soft deletes preserve rows + swipes.** `is_deleted = true` keeps the row for retry restoration. Hard deletes happen via the storage purge path after a successful retry.

## Document References

- [ADR-017: Message Swipes](../../docs/adr/adr-017-message-swipes.md) — swipe rationale, snapshot-per-swipe, event independence.
- [ADR-008: SQLite Snapshot Persistence](../../docs/adr/adr-008-sqlite-snapshot-persistence.md) — `GameStateSnapshot` referenced by each swipe.
- [`../explanation/message_swipe_model.md`](../explanation/message_swipe_model.md) — why the swipe model is shaped this way and which tradeoffs it encodes.
- [`./data_layer.md`](./data_layer.md) — `messages` and `message_swipes` schema, FK relationships, and the non-FK `snapshot_id` rationale.
- [`./game_flow.md`](./game_flow.md) — `add_message` + `state.narrative.history.append(message)` call sites in the phase pipeline.
- [`./triggers.md`](./triggers.md) — event headers on continuation messages; `event_header` field semantics.
