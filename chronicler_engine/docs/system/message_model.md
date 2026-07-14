# Message Model

## Objective

The message model defines how conversation history is structured, accessed, and mutated. Two core types: `Message` (a single narrative unit with multiple swipes) and `MessageHistory` (the ordered collection). The single load-bearing rule: **content lives in `swipes[active_swipe_index]`; use accessor methods for reads; never mirror fields onto `Message` directly**.

## Components

The message model is split across three files:

- **`Message` + `Swipe`** (`src/domain/model/message.rs`) — one `Message` owns its `Vec<Swipe>` + `active_swipe_index`; one `Swipe` holds the actual content fields (`text`, `location_header`, `event_header`, `snapshot_id`). Content reads go through `Message::text()`, `Message::location_header()`, etc. See [Message Accessor Pattern](#message-accessor-pattern) for the read contract.
- **`MessageHistory`** (`src/domain/model/message_history.rs`) — owns `Vec<Message>`; exposes intent-named methods. The encapsulation prevents `.push()` bypass. See [MessageHistory Encapsulation](#messagehistory-encapsulation) for the public surface.
- **`MessageType` + `MessageEntry`** (`src/domain/model/state/message_types.rs`) — `MessageType` discriminates the four message kinds (`Narration`, `Dialogue`, `System`, `Input`); `MessageEntry` is the view-model DTO that templates consume, built via `From<&Message>`.

`MAX_MESSAGES = 1000` lives next to `MessageHistory`. It is the FIFO cap enforced by `append`. See [MAX_MESSAGES Cap](#max_messages-cap) for the bypass surface.

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

## Invariant: Swipe is Sole Holder of Content Fields

`Message` carries no `text`, `location_header`, `event_header`, or `snapshot_id` field. The accessor pattern routes all field reads and writes through swipes, eliminating two-source-of-truth coupling. State is `Swipe`.

## MessageHistory Encapsulation

`MessageHistory` owns `Vec<Message>` and exposes intent-named methods. Callers cannot bypass rules with `.push()`. The encapsulation is intentionally strict: the `pub` surface is wide enough that some methods bypass the per-method cap (see the `MAX_MESSAGES` section).

Public surface (all methods):

| Method | Purpose | Cap enforced? |
|--------|---------|---------------|
| `new()` | Empty history | n/a |
| `from_messages(messages)` | Bulk construct | NO |
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
| `replace(messages)` | Wholesale replace | NO |
| `last_ai_response_index()` | `rposition` for Narration/Dialogue | n/a |
| `last_input_index()` | `rposition` for Input | n/a |
| `last_input_text()` | `(sender, text)` of last Input | n/a |
| `is_last_ai_response_event_continuation()` | Last AI response has `event_header` | n/a |
| `to_message_entries()` | Convert to view DTOs | n/a |

`iter_mut()` is the one method that allows callers to mutate `Message` directly. This is needed for swipe navigation (`set_active_swipe`) and inline edits (`update_active_swipe_text`).

## Retry and Swipe Behaviour

Retry replaces the last AI message's active swipe with a new swipe containing the new generation. The old swipe is preserved (non-destructive). Swipe navigation (`set_active_swipe`) on the last message lets the user compare generations.

Why only the last message? Swiping a non-last message would require deleting all messages after it (they depend on the state that the swipe changes). This is equivalent to retry, which already exists. Limiting swipes to the last message avoids history-truncation complexity while preserving the core value: A/B comparison of the most recent generation.

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

**Cap bypass**: `from_messages` and `replace` do not enforce the cap. Storage loaders use `from_messages` to hydrate from DB; if the DB ever contains more than `MAX_MESSAGES` entries (for example after a downgrade + upgrade cycle), `from_messages` would accept them all.

## Document References

- [ADR-017: Message Swipes](../adr/adr-017-message-swipes.md) -- swipe rationale, snapshot-per-swipe, event independence
- [architecture/system.md §1](../architecture/system.md) -- `state` tier definition
- [system/character_state.md](./character_state.md) -- how `MessageHistory` lives on `NarrativeState`
- [system/game_flow.md](./game_flow.md) -- `add_message` + `state.narrative.history.append(message)` call sites
- [diagnostics/error_catalog.md](../diagnostics/error_catalog.md) -- `Message entry not found` + `History is empty` error variants

