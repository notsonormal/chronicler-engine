# A Swipe's Snapshot is saved before the quantifier runs

Status: needs-triage
Type: bug

## Problem (inferred from the code, not yet tested)

The design says each Swipe keeps the Snapshot of the state its own narration produced, including the quantifier's NPC and movement updates, so a swipe switch never shows a "ghost state":

- `docs/diataxis/explanation/storage_design.md` §Per-swipe state binding
- `docs/diataxis/reference/game_flow.md` §Retry Flow: "Each Message and Swipe points to the Snapshot representing its resulting state."

The code sets a retry Swipe's `snapshot_id` once, at the "pre-quantifier narration" persist (`NarrationGeneration::run` → `PipelineRun::persist_snapshot_or_err` → `MessageService::save_message_and_snapshot`). `save_message_and_snapshot` sets it only while it is `None`, so the later saves ("pre-quantifier phase update", "pre-event snapshot", "post-trigger snapshot") do not move it. The comment in `MessageService::switch_swipe` confirms: "A Swipe's snapshot is written while its narration still generates". A swipe switch therefore probably restores the state before that swipe's quantifier pass.

## Owner decision

Treat it as a code bug (STOP 2, S1). The docs keep the design. Until the fix lands, the `game_flow.md` sentence above is not true of the code.

## First step

Write a failing test: retry a narration whose quantifier moves the player or changes the NPC set, switch away and back, and assert the restored state carries that swipe's quantifier result.
