# Message Model

## Status

> Status: Implemented. See [ADR-017](../adr/adr-017-message-swipes.md) for swipe rationale. Current encapsulation work tracked in T6 (MessageHistory Encapsulation); see super-plan Finding State table.

## Objective

The message model defines how conversation history is structured, accessed, and mutated. Two core types: `Message` (a single narrative unit with multiple swipes) and `MessageHistory` (the ordered collection). The single load-bearing rule: **content lives in `swipes[active_swipe_index]`; use accessor methods for reads; never mirror fields onto `Message` directly**. This prevents A4-class duplication bugs (the `Message` mirrors `Swipe` anti-pattern that was "thermo-nucleared" -- deleted in `6a8531e`).

## Components

| Component | File | Purpose |
|-----------|------|---------|
| `Message` | `src/domain/model/message.rs` | Single narrative unit. Owns `Vec<Swipe>` + `active_swipe_index`. Content accessed via `text()`, `location_header()`, `event_header()`, `snapshot_id()`. |
| `Swipe` | `src/domain/model/message.rs` | Single variant of a message: `text`, `snapshot_id`, `location_header`, `event_header`. Persisted as row in `message_swipes` table. |
| `MessageHistory` | `src/domain/model/message_history.rs` | Ordered collection of `Message`. Encapsulates `Vec<Message>`; exposes intent-named methods (`append`, `edit`, `delete_last`, `replace`, `retain`, `iter`, `iter_mut`, `as_slice`, `clear`). |
| `MessageType` | `src/domain/model/state/message_types.rs` | Enum: `Narration`, `Dialogue`, `System`, `Input`. |
| `MessageEntry` | `src/domain/model/state/message_types.rs` | View-model DTO. Decouples templates + view-models from `Message`. Constructed via `From<&Message>`. |
| `MAX_MESSAGES` | `src/domain/model/message_history.rs` | Constant `1000`. Cap enforced by `append` only. |

## Message Accessor Pattern

Every content read goes through an accessor that delegates to the active swipe. The `Message` struct itself holds ONLY non-swipe fields: `id`, `sender`, `message_type`, `timestamp`, `active_swipe_index`, `swipes`, `is_deleted`. There is NO `text: String` field on `Message`. There is NO `location_header: Option<String>` field on `Message`. There is NO `event_header: Option<String>` field on `Message`. There is NO `snapshot_id: Option<u64>` field on `Message`.

```mermaid
flowchart LR
    M[Message struct] -->|active_swipe_index| S[swipes Vec]
    S --> SI[Swipe index 0..N]
    M -->|text()| A[active_swipe.text]
    M -->|location_header()| B[active_swipe.location_header]
    M -->|event_header()| C[active_swipe.event_header]
    M -->|snapshot_id()| D[active_swipe.snapshot_id]
```

Accessors on `Message`:

| Accessor | Reads from | Returns |
|----------|------------|---------|
| `text()` | `swipes[active_swipe_index].text` | `&str` (empty if no active swipe) |
| `location_header()` | `swipes[active_swipe_index].location_header` | `Option<&str>` |
| `event_header()` | `swipes[active_swipe_index].event_header` | `Option<&str>` |
| `snapshot_id()` | `swipes[active_swipe_index].snapshot_id` | `Option<u64>` |
| `swipe_count()` | `swipes.len()` | `usize` |
| `set_active_swipe(index)` | mutates `active_swipe_index` | `()` |
| `update_active_swipe_text(new)` | mutates `swipes[active_swipe_index].text` | `()` |
| `set_snapshot_id(sid)` | mutates `swipes[active_swipe_index].snapshot_id` | `()` |
| `ensure_valid_swipe_index()` | resets out-of-bounds to 0 | `bool` (true if reset) |

Two accessor methods are private: `active_swipe(&self) -> Option<&Swipe>` and `active_swipe_mut(&mut self) -> Option<&mut Swipe>`. These are the only direct paths to a swipe. Every public accessor goes through them, so a future change to "what counts as active" (e.g. lazy loading, soft delete filter) is a one-line edit.

## Why No Mirrored Fields

The pre-A4 `Message` struct had `text`, `location_header`, `event_header`, `snapshot_id` as direct fields. These mirrored the corresponding `Swipe` fields exactly. Every code path that mutated a swipe ALSO had to mutate the mirrored field on `Message`. This is the textbook duplication trap: two sources of truth that must stay in sync.

The accessor pattern eliminates the trap by removing one of the sources. `Message` has no `text` field, so it cannot disagree with `Swipe::text`. State is `Swipe::text`. Period.

## MessageHistory Encapsulation

`MessageHistory` owns `Vec<Message>` and exposes intent-named methods. Callers cannot bypass rules with `.push()`. The encapsulation is intentionally strict: A5 finding (`MessageHistory` encapsulation) is `active` (T6 owner); the `pub` surface is too wide, but the data structure itself is correctly owned.

Public surface (all methods):

| Method | Purpose | Cap enforced? |
|--------|---------|---------------|
| `new()` | Empty history | n/a |
| `from_messages(messages)` | Bulk construct | NO (N15: bypasses MAX_MESSAGES) |
| `append(message)` | Add to end, evict head if over cap | YES |
| `edit(id, new_text)` | Update active swipe text by id | n/a |
| `delete_last()` | Pop last | n/a |
| `get(id)` | Find by id | n/a |
| `last()` / `last_mut()` | Most recent | n/a |
| `is_last(id)` | Boolean check | n/a |
| `is_empty()` / `len()` | Length | n/a |
| `iter()` / `iter_mut()` | Iteration | n/a |
| `retain(f)` | Filter in place | n/a |
| `clear()` | Empty | n/a |
| `as_slice()` | `&[Message]` | n/a |
| `replace(messages)` | Wholesale replace | NO (N15: bypasses MAX_MESSAGES) |
| `last_ai_response_index()` | `rposition` for Narration/Dialogue | n/a |
| `last_input_index()` | `rposition` for Input | n/a |
| `last_input_text()` | `(sender, text)` of last Input | n/a |
| `is_last_ai_response_event_continuation()` | Last AI response has `event_header` | n/a |
| `to_message_entries()` | Convert to view DTOs | n/a |

`iter_mut()` is the one method that allows callers to mutate `Message` directly. This is needed for swipe navigation (`set_active_swipe`) and inline edits (`update_active_swipe_text`). A future tightening could replace `iter_mut` with explicit methods per use case.

## Retry and Swipe Behaviour

Retry replaces the last AI message's active swipe with a new swipe containing the new generation. The old swipe is preserved (non-destructive). Swipe navigation (`set_active_swipe`) on the last message lets the user compare generations.

Why only the last message? Per ADR-017: swiping a non-last message would require deleting all messages after it (they depend on the state that the swipe changes). This is equivalent to retry, which already exists. Limiting swipes to the last message avoids history-truncation complexity while preserving the core value: A/B comparison of the most recent generation.

Each `Swipe` carries its own `snapshot_id`. Switching swipes restores the exact `GameStateSnapshot` that produced that swipe's text. This is the state-consistent swipe property: no "ghost state" between swipe and world.

Event independence: narration swipes and event swipes are completely separate. Retriggering an event does not affect the narration. The retry flow distinguishes by `is_event = last().event_header().is_some()`.

## Persistence Mapping

| `Message` field | `messages` table column | `message_swipes` table column |
|-----------------|-------------------------|-------------------------------|
| `id` | `id` (PK) | `message_id` (FK) |
| `sender` | `sender` | -- |
| `message_type` | `message_type` | -- |
| `timestamp` | `timestamp` | -- |
| `active_swipe_index` | `active_swipe_index` | -- |
| `is_deleted` | `is_deleted` | -- |
| `swipes[*].text` | -- | `text` |
| `swipes[*].snapshot_id` | -- | `snapshot_id` |
| `swipes[*].location_header` | -- | `location_header` |
| `swipes[*].event_header` | -- | `event_header` |

`Swipe::snapshot_id` is `Option<u64>` because the initial message (before any snapshot was saved) has no snapshot. Nullable at the schema level.

Soft deletes (`is_deleted = true`) preserve the row + swipes for retry restoration. Hard deletes happen via `Storage::purge_soft_deleted` after a successful retry.

## MAX_MESSAGES Cap

`const MAX_MESSAGES: usize = 1000;`. `append()` enforces: if `len >= MAX_MESSAGES`, evict head (`remove(0)`) before push. This is the FIFO eviction policy.

**N15 finding**: `from_messages` and `replace` bypass the cap. Storage loaders use `from_messages` to hydrate from DB; if the DB ever contains > 1000 messages (e.g. from a downgrade + upgrade cycle), `from_messages` would accept all of them. Owner T6.

## Cross-references

- [ADR-017: Message Swipes](../adr/adr-017-message-swipes.md) -- swipe rationale, snapshot-per-swipe, event independence
- T6: MessageHistory Encapsulation (N15 cap bypass + `from_messages_trusted` decision) -- see super-plan Finding State table
- [architecture/system.md §1](../architecture/system.md) -- `state` tier definition
- [system/character_state.md](./character_state.md) -- how `MessageHistory` lives on `NarrativeState`
- [system/game_flow.md](./game_flow.md) -- `add_message` + `state.narrative.history.append(message)` call sites
- [diagnostics/error_catalog.md](../diagnostics/error_catalog.md) -- `Message entry not found` + `History is empty` error variants

## Open Findings

Items from the abstraction-fixes super-plan Finding State table that affect this code:

- **A5 `MessageHistory` encapsulation** -- `active`, owner T6. `replace`, `retain`, `iter_mut`, `as_slice`, `clear`, `from_messages` are all `pub`; cap bypass (N15) is the load-bearing issue. T6 will narrow the surface.
- **A11 `MessageEntry` DTO mirroring** -- `active`, owner T10. `MessageEntry` mirrors `Message` fields via `From<&Message>`; T10 will collapse via additional `From` impls or struct-of-arrays view model.
- **N15 `from_messages` bypasses MAX_MESSAGES cap** -- `active`, owner T6. Only `append` enforces the cap; `from_messages` and `replace` do not. T6 will add `from_messages_trusted` for storage loaders.
- **A4 `Message` mirrors `Swipe`** -- `closed`. Accessor pattern landed; this finding is resolved. Any future re-flag of A4 is itself a stale finding (per the super-plan Re-flag Rule).