# Put the room name on the Narration that arrives in the room

Type: task (AFK)
Status: resolved
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

## Answer

**The engine commit writes the room name back onto the Narration that moved the player.** The Quantifier reports a movement only after `narration_generation` saved the Narration, so the header cannot be set at push time.

- `GameState::log_movement_completion` (`src/domain/model/state/game_state.rs`) finds the arriving Narration (the redo's Swipe while a Retry runs, else the newest Input or Narration), sets its header, and clears `pending_location`, so the next Message gets none.
- The Swipe row already exists, so the domain queues an `ArrivingLocationWrite` in `NarrativeState::arriving_location_write` (`#[serde(skip)]`). `MessageService::save_message_and_snapshot` applies it through the new `Storage::update_swipe_location_header` and clears it only after a successful write.
- The F09-1 seed in `reconstruct_retry_state` is removed: a redo Swipe now names only the room its own generation arrives in.
- An event continuation no longer inherits the main Narration's room. It is the "next Message" of the moving generation. Scenario 10.8 now pins that every event Swipe has no header, so a swipe switch still neither adds nor removes one.
- **`pending_location` still exists.** Only the scenario seeds write it now. `docs/plans/subplan-c-mapless-enablement.md` can still delete it.

**Tests.** Tier 1: scenario 1.14 `test_moving_narration_reports_the_room_it_arrives_in_http` (`tests/http/actions.rs`) fails without the fix (`left: None, right: Some("Room room2")`); scenarios 9.2, 9.8, 10.8 and 22.5 in `tests/http/swipe_new.rs` cover main, guided and event Retry and the swipe switch. Unit tests in `game_state_tests.rs`, `retry_tests.rs`, `message_service_tests.rs` and the storage tests.

**Deferred by the user (2026-10-11, option B).** The Context asks that the header agree with the Snapshot that swipe switch restores, and this is not met: after a swipe switch to a moving Narration's Swipe, the restored state is the room the generation started from, while the header names the arrival room: the Swipe links the pre-quantifier Snapshot. The Snapshot pairing is older than this ticket, but this ticket makes it visible, because that Swipe had no header before; and changing it would move the guided-Retry anchor and ticket 09's failed-redo semantics. Owner: [Decide where the room a generation arrives in lives](../../turn-state-model/issues/03-decide-where-the-room-lives.md); the map lists it under Out of scope.

**Gate.** Branch gate `phase3-15-room-name-header/logs/build_20261011_223642.log`; combined with ticket 16, `phase3-integration-15-16/logs/build_20261011_224039.log` (both worktrees under `/home/node/.herdr/worktrees/chronicler-engine/`). After the review fixes, on the uncommitted tree in the main checkout: `logs/build_20261011_231121.log`, all steps OK, guardrails 163, integration 1634 passed / 2 skipped, browser 29: guardrails 163, integration 1633 passed / 2 skipped, browser 29.
