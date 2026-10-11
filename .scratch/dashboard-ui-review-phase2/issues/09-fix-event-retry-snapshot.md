# Make event Retry restore the trigger context and never roll the game back

Type: task (AFK)
Status: open
Blocked by: —

## Question

Event Retry always fails with "Retry failed: missing trigger context", and the failure saves an older snapshot as the current game state. Which snapshot must an event Retry restore, and how does the failure path stop rolling the game back?

## Context

- Finding **S1 (P1)** of the [2026-10-11 re-review](../re-review-2026-10-11.md), shot `tmp/p2-07/20-event-retry-failed.png`.
- `find_retry_anchor_msg` (`src/application/message_service.rs`) anchors an event Retry on the last non-event Message, the main narration. Its snapshot was written when the narration was first saved, before movement and before `last_trigger` was set. `save_message_and_snapshot` links a snapshot only to a Message saved for the first time, so the "pre-event snapshot" from `phase_trigger_continuation_llm_call` (`src/application/pipeline/pipeline_run.rs`) links to nothing.
- `retry_event_continuation` (`src/application/pipeline/action_pipeline/core.rs`) then finalizes the error on the restored state and saves it. Observed: player back in `courtyard` while the story shows the Entrance Hall, `trigger_fired` cleared, and the non-repeating trigger fired again on the next turn.
- Tier-1 scenario 10.1 (`tests/http/swipe_new.rs`) passes because `seed_event_flow` (`src/test_support/fixtures.rs:752`) links a snapshot that holds `last_trigger` to the main narration. The real pipeline never does that, so the seed must follow the real save order.
- Decide with care: the narration's swipe snapshot is also what a swipe switch restores ([ticket 43 of phase 1](../../dashboard-ui-review/issues/43-reset-status-and-dock-on-swipe-switch.md)), and main Retry anchors on the Input. A change to which snapshot a narration swipe points at changes those paths too.

## Done when

- Event Retry after a normal trigger turn and after a Retrigger adds a new swipe to the event Message and ends Ready.
- A failed event Retry (and any failed retry) leaves the current game state as it was before the click.
- A test reproduces the real save order (no hand-linked `last_trigger` snapshot) and fails without the fix. Name the tier.
- `python build.py` is green. Commit after user approval.
