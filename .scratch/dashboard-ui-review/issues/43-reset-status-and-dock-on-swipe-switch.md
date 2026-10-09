# Reset the generation status and the options dock when a swipe switch restores a snapshot

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Switching a swipe (`POST /message/:id/swipe/:index`) restores that swipe's snapshot, but the snapshot was written during narration generation and carries `input_buffer.status = Generating` / `phase = Narrating`. `/status/generating` then answers `narrating` forever: the dashboard flips to "Generating narration...", Send becomes a disabled "Stop", Enter cannot submit, and a reload does not clear it — the player is stuck until some other generation finalizes. The same snapshot predates options generation, so the options dock silently blanks on its next poll even though the narration just shown had options.

Decide what a restored swipe's snapshot should contain, then make both follow it: the game Idle on restore (persist the swipe's snapshot after the turn finalizes, or normalise the status), and the dock keeping, dropping or regenerating the options. Surface the restore itself — today nothing tells the player a swipe was restored except the counter, and the one state signal it does touch is the wrong one.

Also decide finding 6.6: while the next turn generates, the dock keeps the previous turn's options live and clicking one submits through `useOption`, so a stale option can be picked mid-generation. This may be intentional — decide it rather than assume a defect.

## Context

- Findings 6.1 (P1), 6.2 (P2), 6.5 and 6.6 (P3) of [Review swipes, the options dock and the Thinking states](06-review-swipes-options-thinking.md). They share one root cause: what the restored snapshot holds.
- Evidence (local only): `tmp/ui-review/87-stuck-after-swipe-switch.png` (2/2 + "Generating narration..." + dimmed Stop; `/status/generating` = `narrating` after the switch, `idle` after a turn; `#submit-btn.disabled = true`; a real key press produced no request; reload plus 6s still stuck), `86-swipe-switched-1of2.png` (dock `innerHTML` 1125 bytes → 0), `82-turn-thinking.png` (dock populated while the status reads "Quantifying scene...").
- Root cause recorded in the review: `MessageService::switch_swipe` re-saves the swipe's snapshot, which `save_message_and_snapshot` wrote while `input_buffer.status == Generating`.
- The generation status of a swipe-less turn, and the "Stop" label during it, are [Rework the action area so a text check cannot strand it](42-rework-the-action-area.md)'s; do not duplicate them here.

## Done when

- Switching between swipes leaves the dashboard Ready with an enabled Send, and the restore is visible to the player.
- The dock matches the narration on screen after a switch, and a turn in flight cannot be hijacked by the previous turn's options.
- A browser test covers switch-between-swipes (Ready, enabled Send, dock contents) and option-click-during-generation.
- Test placement follows `tests/STRATEGY.md`, and the answer names the tier. Until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names.
- `python build.py` is green, the user reviews the diff, and the work is committed through `/commit-and-push`.

## Answer

**A restored Swipe is a settled branch point.** `MessageService::switch_swipe` normalises the restored snapshot: the game is `Idle`, the phase is the default, and the offered options are cleared. The restore is surfaced in a transient `#restore-notice` line (`role="status"`, cleared after 4s).

**Why not persist the post-turn snapshot instead.** A Swipe's snapshot *is* its retry anchor: `find_retry_anchor` reads the message's active-swipe snapshot, and guided / impersonate / user-regeneration redos reconstruct from it (pinned by `retry_tests::test_retry_flow_guided_turn_retries_without_older_input_text`). Re-pointing that snapshot at the post-turn state would make those redos start *after* their own turn. Regenerating the options on switch was rejected: an LLM call plus a gate claim inside the switch handler.

**Finding 6.6 is a defect, not intended.** `useOption` checked only `input.disabled`, which is false while a generation runs (only the primary button is disabled), so a stale option could be picked mid-turn. The client now refuses while the live status is generating.

**Tiers.** Unit: `message_service_tests::test_switch_swipe_restores_a_settled_snapshot`. Tier 1: `tests/http/swipe_switch.rs`, scenarios 36.1/36.2 in the new `docs/specs/swipe_switch.md`. Tier 2 (stub browser): 37.1 in the new `docs/specs/browser_swipes.md` and 26.5 in `docs/specs/browser_options.md`. The browser half is tier 2 because the real LLM mock cannot produce two swipeable swipes — its unparseable response appends an uncertain-NPC `[System]` message, so no swipe controls render; the server outcome is tier 1.

**Fail-without-fix evidence.** Commenting out the three normalisation lines fails the unit test; reverting `resetStatusToReady()` fails 37.1 on the status; reverting `announceSwipeRestore()` fails the notice assertion; reverting the `statusIsGenerating()` guard fails 26.5.

**Merge note.** With [Render the story-log and LLM Messages stub fixtures from the real templates](50-render-stub-fixtures-from-templates.md), the two hand-written story-log fixtures this ticket added were dropped: the scripted two-Swipe and restored shapes now render through `NarrativeLogTemplate` from scripted entry data.

`python build.py` is green on the merged tree.
