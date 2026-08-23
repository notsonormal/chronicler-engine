# 02 — Deepen a ReplaySteering module for the replay blob (architecture candidate 1)

Type: grilling
Status: open
Blocked by: (none)

## Question

Do we commit to a deep **ReplaySteering** module that owns the replay-blob
lifecycle — stage → attach-to-swipe → resolve-for-turn → resolve-for-retry —
so the invariant "steering is transient, rides on the swipe, re-applied on
retry" lives in one place, and if so, what is the shape of the deepened module?

## Background

**Friction (from `architecture-review.html`, candidate 1, Strong):**
The replay-blob invariant is enforced by no single module. Six files jointly
maintain it:

- `src/domain/model/message.rs` — blob definition (`GenerationReplay`).
- `src/domain/model/state/narrative_state.rs` — `pending_replay` transient
  field.
- `src/domain/model/state/game_state.rs` — `push_message` copies
  `pending_replay` onto the active swipe when a message is created.
- `src/application/pipeline/action_pipeline/action.rs` — builds the blob and
  stages it as `pending_replay`.
- `src/application/pipeline/pipeline_run.rs` — `resolve_guide` /
  `resolve_impersonate` read the blob back from both fresh `PipelineInputs`
  *and* `retry_target.replay()`.
- `src/application/pipeline/action_pipeline/retry.rs` — reconstructs the retry
  mode from `old_target.replay()`.

Understanding the lifecycle requires bouncing between all six. Lost locality.

**Relationship to ticket 01 (NarrationTurn).** The report notes NarrationTurn
gives ReplaySteering "a single consumer to call against." This ticket can be
grilled independently — the blob lifecycle is its own concern — but the
grilling should account for where the blob is consumed (NarrationTurn, if 01 is
accepted; the current scattered pipeline reads, if not).

## What to decide

- Commit or reject the ReplaySteering deepening.
- If committed: the module's **interface** (the report sketches
  `stage() · attach_to_swipe() · resolve_for_turn() · resolve_for_retry()`),
  its **seam** (does it sit on the domain `GameState`, the application
  pipeline, or between?), what stays as the value type in `message.rs` vs what
  lifecycle moves behind the interface, and what **tests** cover the invariant
  at the new seam.
- Whether `GameState::push_message`'s two implicit steering invariants
  (swipe-retry materialisation; `pending_replay` attachment) move behind this
  module or stay on `GameState`.
- Whether the mutual-exclusion rule (guide vs impersonate) lives here or in
  ticket 04's SteeringPromptPolicy.
