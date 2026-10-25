# Make event Retry restore the trigger context and never roll the game back

Type: task (AFK)
Status: resolved
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

## Answer

**An event Retry regenerates the continuation on the Snapshot of the Swipe it redoes, and a failed redo is put back where the click found it.**

**Root cause.** A trigger turn saves the main Narration before the engine commit's movement and before the trigger exists, so `find_retry_anchor`'s anchor (the last non-event Message) pointed at a snapshot with the pre-turn room and `last_trigger: None`. The snapshot that does hold the trigger context is the one the trigger commit writes, and that is the snapshot the event Message's own Swipe links to. Rewinding to the Narration's snapshot replayed the whole turn; `retry_event_continuation` then failed with "Retry failed: missing trigger context" and saved the rewound state as the current one (courtyard, `trigger_fired` cleared, the non-repeating trigger free to fire again). The pre-event snapshot keeps its job: a recovery point for a trigger call that never lands (property I.9 of `docs/specs/retrigger.md`).

**Which snapshot an event Retry restores.** `ActionPipeline::retry_last_response` (`src/application/pipeline/action_pipeline/retry.rs`) reconstructs an event target from the Snapshot of the event Message's active Swipe — the post-trigger Snapshot — and keeps the anchor Narration's Snapshot for the non-event modes only. That Snapshot holds the trigger context, the post-movement room, the fired-trigger marks and the scene *before* the post-trigger Character reconciliation, so the redo replays the event turn from its own start: the reconciliation recomputes enter/leave events from the same baseline, and the outcome does not depend on any later write. The narration's own Swipe snapshot is untouched, so swipe switch and main Retry keep the anchors ticket 43 of phase 1 gave them.

**A first revision of this fix used the latest Snapshot, and the review of `b5ff953e` showed why that is wrong.** The latest Snapshot is taken after the original event's Character reconciliation, so the new Swipe inherited the previous Swipe's encounter (Character X stays met even though the new continuation never meets X) and the retry read whatever trigger context a later turn had left behind. It also made the result depend on whether the player had switched a Swipe. Scenarios 10.5–10.7 pin all three.

**How the failure path stops rolling the game back.** Every redo replays from a rewound anchor and the pipeline persists as it goes, so a failure used to leave that older world durable. `restore_click_time_state_after_failed_redo` now puts the click-time state back, failure on screen, when the persisted status is an `Error` and the redo never persisted a Swipe for the Message it retries. The gate keeps the attempt's world when a narration *is* on screen, because the engine commit or the trigger commit already advanced it — the shape a main redo's failed trigger continuation leaves behind. The undo writes a snapshot only; Messages the attempt wrote (the trigger-failure System entry) stay. It is skipped when the current game changed while the redo ran, and when the attempt's Snapshot or the Messages cannot be read: an unknown outcome is not a licence to overwrite a game.

**Tests, by tier.**

- **Tier 1** (`tests/http/swipe_new.rs`): scenarios 10.1, 10.2, 10.4, 10.5, 10.6, 10.7, 11.8, 11.9 and 11.10.
  - 10.5 and 10.7: an event whose first Swipe meets Character Gabriella while the retry's own continuation leaves her out. Without the post-trigger-Snapshot fix 10.5 fails `left: 1, right: 0` — the previous Swipe's encounter carries over. 10.7 repeats it after a prior swipe switch and passes either way: that switch re-saves the Swipe's Snapshot as the latest one, which is exactly why the retry must not read the latest one.
  - 10.6: a later turn fires another Trigger, then the whole later turn is deleted. Without the fix it fails `left: Some("Gabriella Encounter in room2"), right: Some("Test")` — the retry picks up the other Trigger's context.
  - **Fail without the original fix** (pre-fix tree, real-order seed): 10.1 `event retry appends exactly one swipe`; 10.2 `left: "room_1", right: "room2"` (the rollback); 11.8 `Retry failed: missing trigger context` — the review's S1 wording; 11.9 `left: "room_1", right: "room2"`. 10.4 was written after the fix and fails by the same mechanism as 10.1 (inferred, not re-run on the pre-fix tree).
  - 11.10 pins the other half of the gate: with the gate commented out it fails `left: "room_1", right: "room2"`.
  - `seed_event_flow` and `seed_main_turn_flow` (`src/test_support/fixtures.rs`) seed the pipeline's real order, and the seeded event Narration now takes its event header from the trigger context it stores.
- **Unit** (`src/application/pipeline/action_pipeline/retry_tests.rs`): `setup_event_flow` follows the real order too, and the three inline event seeds that pointed the event Message at Snapshot id 0 (a `db_id` that `from_game_state` never sets) now link the row `save_snapshot` returned.

**Doc.** `docs/diataxis/reference/game_flow.md` "Retry Flow" names the Snapshot each redo restores and what a failed redo does with the click-time state.

**Gate.** `python build.py` green (`logs/build_20261011_162000.log`): all steps OK; integration `1616 passed, 0 failed, 2 skipped`; browser `27 passed, 0 failed`; guardrails `162 passed`; architecture `1 passed`.

**Follow-ups for ticket 13 (P3).**

1. A retried event Swipe loses the location header: the post-trigger Snapshot is written after `commit_trigger_narration` consumed `pending_location`, so the regenerated Swipe has `location_header: None` while the Message's first Swipe names the room. Switching swipes adds and removes the header. Same family as S6.
2. `check_retry_anchor` validates the anchor Narration's Snapshot while an event redo reads the event Message's own Swipe Snapshot. Property I.7 makes validating the anchor deliberate — a Message with a dangling Snapshot is a data-integrity violation — and the event's own Snapshot is checked when the redo loads it, so this was left alone.
3. A failed redo that persisted a Swipe whose engine commit never landed (a post-engine persist failure) keeps the rewound world, because the gate reads "a narration is on screen". Rare, and no better world exists; the rationale is in the doc comment.
4. From the review of `b5ff953e`: with a System Message last (a failed trigger continuation), `find_retry_anchor_msg` decides "event" from the last Message while `resolve_retry_target` decides it from the last Narration or Input, so a second Retry anchors on the Input and cuts the main Narration out of the in-memory history the options are rewritten from. Ticket 10's area.

**Status vocabulary.** `Status: resolved` follows `docs/agents/issue-tracker.md` and every sibling ticket, so ticket 14's `Blocked by` check sees this ticket as done.
