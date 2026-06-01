# Plan: Fix Generation Phase Stuck on "Generating narration..." During Post-Generation

**Date:** 2026-06-01
**Status:** ✅ Completed
**Type:** Bug Fix

## Problem

When the pipeline runs `phase_post_generation()` (quantification), the `GenerationPhase` remains `Narrating`, so the UI displays "Generating narration..." even though narration is complete and quantification is running.

The `GenerationPhase::Quantifying` variant and its display text `"Quantifying scene..."` already exist — they're just never set in the main pipeline path.

## Root Cause Analysis

In `pipeline.rs`:
- `phase_pre_main_snapshot` (line 130) sets `phase = Narrating` ✅
- `phase_narrate` completes narration — no phase change
- `phase_post_generation` (line 208) runs quantification — **no phase change** ❌
- `phase_finalize` (line 371) resets to `Idle`/default

Only the trigger continuation path (`reconcile_post_trigger_npcs`, line 492) correctly sets `GenerationPhase::Quantifying`.

## Solution

Add phase transition and snapshot save at the start of `phase_post_generation()` in `pipeline.rs`, mirroring the pattern in `reconcile_post_trigger_npcs()`.

### Files Changed

| File | Lines Changed | Description |
|------|---------------|-------------|
| `src/application/action_pipeline/pipeline.rs` | +4 | Add phase transition at start of `phase_post_generation()` |

### Code Change

```rust
// In phase_post_generation(), before quantification work:
state.narrative.input_buffer.phase = GenerationPhase::Quantifying;
if let Err(e) = save_message_and_snapshot(self.ctx, state) {
    tracing::warn!("Failed to save pre-quantifier phase update: {e}");
}
```

## Documentation Updated

| Document | Change |
|----------|--------|
| `docs/system/game_flow.md` | Updated Phase 4.5 flowchart to show phase transition step |
| `docs/CHANGELOG.md` | Added entry under "Fixed" section for 2026-06-01 |

## Verification

1. ✅ `cd chronicler_engine && python build.py` — all 878 tests pass (+2 new tests)
2. ✅ Coverage maintained at 81.9% (above 80% threshold)
3. ✅ Clippy clean with `-D warnings`
4. ✅ Architecture and guardrail tests pass
5. ✅ Template tests already assert `GenerationPhase::Quantifying` display text ("Quantifying scene...") — no change needed

## New Tests Added

| Test | Location | Purpose |
|------|----------|----------|
| `test_phase_transitions_to_quantifying_during_post_generation` | `src/application/action_pipeline/actions_tests.rs` | Verifies phase transitions to Quantifying mid-flight during post-generation |
| `test_narration_saved_before_quantifying_phase` | `src/application/action_pipeline/actions_tests.rs` | Verifies narration is saved and phase is Quantifying before quantifier completes |

Both tests use a slow quantifier backend (200ms delay) to inspect the state mid-flight and confirm:
- Phase is `GenerationPhase::Quantifying` during post-generation (not stuck on Narrating)
- Narration is persisted to storage before quantifier completes
- Phase resets to default after completion

## Impact

**Before:** UI stuck on "Generating narration..." for ~29s during quantifier analysis
**After:** UI correctly shows "Quantifying scene..." during post-generation

This provides accurate feedback to users about what the system is doing and maintains consistency between the main pipeline and trigger continuation paths.
