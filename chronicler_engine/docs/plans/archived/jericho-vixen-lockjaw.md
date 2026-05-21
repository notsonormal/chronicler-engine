# Plan: Fix Game UI Delay for LLM Responses

## Problem
The game UI (`/fragment/story-log`) takes ~5 seconds to show a narration after the LLM generates it, even though the LLM Messages tab shows the raw response immediately. The story-log polls every 2s but the narration only appears after the full pipeline completes.

## Root Cause
The action pipeline only saves to snapshot storage at the start (`phase_pre_main_snapshot`) and end (`phase_finalize`). The narration is added to in-memory history during `phase_engine_commit`, but the snapshot is not updated until after the quantifier LLM call and any trigger continuation finish.

**Pipeline timeline:**
1. `phase_pre_main_snapshot` — saves input-only state
2. `phase_narrate` — narration LLM (~2-3s) ← LLM Messages shows this now
3. `phase_post_generation` — quantifier LLM (~2-3s)
4. `phase_engine_commit` — narration added to history (not saved)
5. `phase_finalize` — snapshot saved with narration ← Game UI shows this now

The snapshot is stale between steps 1 and 5, so the story-log cannot display the narration until the very end.

## Fix: Intermediate Snapshot Save After Narration

Add the narration to state history and save an intermediate snapshot immediately after `phase_narrate`, before the quantifier runs. Then guard `execute_freeaction_impl` against re-adding the same narration.

### Changes

**1. `src/application/action_pipeline/pipeline.rs`**
After `phase_narrate` returns the narration text:
- Add `state.add_log(narration_text.clone(), None, LogType::Narration)`
- Save intermediate snapshot via `save_state(self.ctx, &mut state)`
- Log errors but do not fail the pipeline if the intermediate save fails

**2. `src/engine/action_processing.rs`**
In `execute_freeaction_impl`, before calling `next_state.add_log()`:
- Check if the last history entry is already the same narration text with `LogType::Narration`
- Skip adding if it's a duplicate

### Why This Is Safe
- The quantifier bases its decision on `<LatestNarration>` (the narration text passed directly to the prompt), not on `<RecentHistory>`. Including the narration in recent history is harmless.
- `persist_new_messages` in subsequent saves skips entries whose `id` is already assigned (not `UNPERSISTED_ID`).
- The story-log will show the narration within ~2s of the LLM completing, while the quantifier and triggers continue processing.
- The status display still shows "Generating..." until `phase_finalize` sets it to Idle.

### Alternative Considered
**Reduce poll interval from 2s to 1s.** Rejected because it only shaves ~1s off the perceived delay and does not address the root cause (the snapshot remains stale for the entire quantifier + trigger duration).

## Files to Modify
- `src/application/action_pipeline/pipeline.rs`
- `src/engine/action_processing.rs`

## Validation
- Run `cd chronicler_engine && cargo test` to verify pipeline and action processing tests pass
- Manual verification: submit an action and confirm the narration appears in the game UI shortly after the LLM Messages tab shows it
