# Replay blob on Swipe for guide and impersonate retry

Type: task
Status: pending

## Question

Add a per-swipe replay blob so guided and impersonated turns are retryable without losing the steering inputs.

Per the design synthesis (`../research/04-design-synthesis.md`, Q4 + Q6):

1. Add a replay field to `Swipe` in `src/domain/model/message.rs`. Today `Swipe` is `{ text, snapshot_id, location_header, event_header }` — no metadata field. The blob mirrors Marinara's `GenerationReplay` (`generation-replay.ts`): `guide: Option<String>`, `impersonate: bool`, `impersonate_direction: Option<String>`, `impersonate_preset_id: Option<String>`.
2. Storage migration: the `message_swipes` table (`src/adapters/driven/storage/swipes.rs`) gains a column for the blob (JSON-serialized). Update `insert_swipe`, the load path, and in-memory backend.
3. Retry replay: on retry (`src/application/pipeline/action_pipeline/retry.rs`), read the blob off the target swipe and re-apply the steering inputs to the regenerated turn. Verified Marinara path: regenerate reads `extra.generationReplay`, re-sets impersonate, excludes the old text from context, re-runs, saves new result as a swipe.
4. The blob lives on `Swipe`, NOT `GameStateSnapshot`. Verified: `Message.set_snapshot_id` delegates to `active_swipe_mut().snapshot_id` (`message.rs:108`) — the snapshot is associated with the swipe. `GameStateSnapshot` is a pure world-state freeze (`from_game_state`); steering is generation metadata, not world state.

This is the shared mechanism — guide retry (ticket 07) and impersonate retry (ticket 09) both depend on it.