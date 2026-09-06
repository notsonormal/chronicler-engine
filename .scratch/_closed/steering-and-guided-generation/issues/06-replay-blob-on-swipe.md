# Replay blob on Swipe for guide and impersonate retry

Type: task
Status: closed

## Question

Add a per-swipe replay blob so guided and impersonated turns are retryable without losing the steering inputs.

Per the design synthesis (`../research/04-design-synthesis.md`, Q4 + Q6):

1. Add a replay field to `Swipe` in `src/domain/model/message.rs`. Today `Swipe` is `{ text, snapshot_id, location_header, event_header }` — no metadata field. The blob mirrors Marinara's `GenerationReplay` (`generation-replay.ts`): `guide: Option<String>`, `impersonate: bool`, `impersonate_direction: Option<String>`, `impersonate_preset_id: Option<String>`.
2. Storage migration: the `message_swipes` table (`src/adapters/driven/storage/swipes.rs`) gains a column for the blob (JSON-serialized). Update `insert_swipe`, the load path, and in-memory backend.
3. Retry replay: on retry (`src/application/pipeline/action_pipeline/retry.rs`), read the blob off the target swipe and re-apply the steering inputs to the regenerated turn. Verified Marinara path: regenerate reads `extra.generationReplay`, re-sets impersonate, excludes the old text from context, re-runs, saves new result as a swipe.
4. The blob lives on `Swipe`, NOT `GameStateSnapshot`. Verified: `Message.set_snapshot_id` delegates to `active_swipe_mut().snapshot_id` (`message.rs:108`) — the snapshot is associated with the swipe. `GameStateSnapshot` is a pure world-state freeze (`from_game_state`); steering is generation metadata, not world state.

This is the shared mechanism — guide retry (ticket 07) and impersonate retry (ticket 09) both depend on it.

## Resolution

Implemented the per-swipe `GenerationReplay` blob and its full persistence round-trip; `python build.py` green (168s, all tests pass).

**Domain** (`src/domain/model/message.rs`): new `GenerationReplay` struct `{ guide: Option<String>, impersonate: bool, impersonate_direction: Option<String>, impersonate_preset_id: Option<String> }` (derives `Default`); `Swipe` gains `replay: Option<GenerationReplay>` (`#[serde(default)]` for backward-compatible deserialization); `Message::replay()` accessor returns the active swipe's blob.

**Storage migration v15** (`src/adapters/driven/storage/utils/plumbing.rs`): `ALTER TABLE message_swipes ADD COLUMN replay TEXT` (guarded by `column_exists`). New DBs get it via the v15 ALTER, matching the codebase convention for column additions (persona columns, etc.).

**Storage plumbing** (`swipes.rs`, `models/swipe.rs`, `mappers/message.rs`): `insert_swipe` serializes the blob to JSON; `load_swipes_for_messages` parses it back; `DbSwipe` struct + `from_row` gain the `replay` column (index 7); the message mapper round-trips it both directions. The in-memory backend stores `Swipe` objects directly, so it needs no change — the blob rides along by clone.

**Retry inheritance** (`src/domain/model/state/game_state.rs`, `push_message`): when a retry target appends a new swipe, the new swipe inherits `target.replay().cloned()` from the previously-active swipe. This makes re-retry preserve the steering inputs.

**Integration point for 08/09 (not wired here):** the retry path already loads `old_target` (with its swipes, hence the blob) into `state.narrative.retry_target` (`retry.rs`, `reconstruct_retry_state`). During prompt assembly — before `push_message` advances the active index — tickets 08 (guide layer) and 09 (impersonate preset) read `state.narrative.retry_target.replay()` to re-apply the steering to the regenerated turn. The blob's prompt-layer *consumption* is deliberately deferred; this ticket delivers only the blob, its round-trip, and the retry inheritance.

**Tests added:** storage replay round-trip (sqlite + in-memory, `swipes_tests.rs`); `message_swipes.replay` column-exists (`db_tests.rs`); retry-inheritance (`game_state_tests.rs` — a replay-bearing target produces a new swipe that inherits the blob). All existing `Swipe { ... }` and `DbSwipe { ... }` literals updated for the new field.

**Out of scope here (correctly):** writing the blob on guide/impersonate swipe creation (tickets 07/08/09 own entry + the guide layer / impersonate preset that *populate* it), and reading it to steer the prompt (08/09 own the layer/preset consumption).