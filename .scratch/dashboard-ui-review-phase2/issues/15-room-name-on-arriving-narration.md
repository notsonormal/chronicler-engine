# Put the room name on the Narration that arrives in the room

Type: task (AFK)
Status: open
Blocked by: —

## Question

The location header (the room name) lands one Message late. How does the Action Pipeline put it on the Narration that moved the player instead?

## Decision (user, 2026-10-11)

The room name goes on the Narration that arrives in the room. The user agreed with the cause: the Quantifier finds the movement only after the pipeline saved the Narration, so the room name has to be written back onto that Narration after the Quantifier runs.

## Context

- Finding **S6 (P3)** of the [2026-10-11 re-review](../re-review-2026-10-11.md). [Follow up on small issues found in the phase-2 re-review](13-follow-up-phase-2-re-review-p3s.md) ruled it out of that ticket as a pipeline rule, not dashboard code; its Answer (follow-up 1) lists the paths.
- Today: the pipeline saves the Narration; the Quantifier finds the movement; `attempt_movement` / `attempt_semantic_walk` (`src/domain/model/state/game_state.rs`) set `narrative.pending_location`; the next `push_message` takes it as its `location_header`. The starting scenario (`game_state.rs`, the `pending_location = Some(room_name)` before `add_message`) already puts the header on its own Narration; keep that.
- Retry: ticket 13 fixed F09-1 by seeding `pending_location` from a sibling Swipe in `reconstruct_retry_state` (`src/application/pipeline/action_pipeline/retry.rs`). Once the header lives on the arriving Narration, check whether that seed is still needed, and remove it if not. Ticket 13 notes that the non-event retry paths (main, guided) lose the header the same way; cover them.
- Snapshot and Swipe: the header must agree between the Message, each Swipe and the Snapshot that Retry and swipe switch restore. A retry Swipe whose redo does not move the player must not keep a stale header.
- `docs/plans/subplan-c-mapless-enablement.md` plans to delete `pending_location`. Do not block on that plan; if the fix removes `pending_location`, say so in the Answer.
- Related, not in scope: [Replace `System` Messages with an error channel outside the Message history](../../turn-state-model/issues/02-replace-system-messages.md). "Entered unknown location" is one of those notices.

## Done when

- After a generation that moves the player, the moving Narration carries the new room's header, and the next Message does not.
- Main Retry, guided Retry and event Retry keep the header correct on each Swipe; a swipe switch shows the header of the selected Swipe.
- A tier-1 test pins the moving Narration's header and fails without the fix. Name the tier.
- `python build.py` is green.
