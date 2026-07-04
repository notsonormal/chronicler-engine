# Pipeline Decomposition Review Fixes (Round 3)

## Context

Three rounds of refactoring have been applied to `action_pipeline` and `bootstrap`:
1. Methods → free functions in `phases.rs`; `run.rs` → `init_game.rs`
2. Removed thin wrappers, fixed `ArrivalTaskContext` fake `GameServiceContext`
3. Restored `Copy` on `NpcContext`, removed `next_state.clone()`

Net result: the `(service: &B, ctx: &GameServiceContext)` pair now appears 5 times as call-site arguments and 7 times in function signatures, where previously `self` carried them in one place. Plus two bugs: `error_return` clones the entire `GameState` unnecessarily, and `phase_pre_main_snapshot` discards `persist_snapshot_failed`'s return value.

Fix: **re-attach the phase functions to `ActionPipeline` as `impl` methods** while keeping `phases.rs` as the file they live in. Split `impl` blocks across files are standard Rust. This eliminates parameter threading without adding another wrapper struct.

## Approach

### Step 1: Convert `phases.rs` free functions → `impl ActionPipeline` methods

**File**: `chronicler_engine/src/application/action_pipeline/phases.rs`

**Imports**: Add `ActionPipeline` to the `use super::pipeline` line:
```rust
use super::pipeline::{ActionOutcome, ActionPipeline, ActionPipelineBackend, PipelineResult};
```

Wrap all functions in:
```rust
impl<'a, B: ActionPipelineBackend> ActionPipeline<'a, B> {
    // all functions here
}
```

**Signature changes** — remove `ctx: &GameServiceContext` and `service: &B` params, add `&self`; remove `<B: ActionPipelineBackend>` generics (comes from impl header):

| Function | Old params | New params |
|---|---|---|
| `persist` | `(ctx: &GameServiceContext, state: &GameState)` | `(&self, state: &GameState)` |
| `persist_snapshot_failed` | `(ctx: &GameServiceContext, state: &mut GameState, label: &str) -> bool` | `(&self, state: &mut GameState, label: &str) -> bool` |
| `error_return` | `(ctx: &GameServiceContext, state: &mut GameState, msg: String) -> PipelineResult<…>` | `(&self, mut state: GameState, msg: String) -> PipelineResult<…>` — **also take state by value, remove `.clone()`** |
| `phase_narrate` | `<B>(service: &B, ctx: &GameServiceContext, mut state: GameState, input: &str, world: &WorldCard, map: &MapDef, player: &PlayerCard, all_npcs: &[NpcCard])` | `(&self, mut state: GameState, input: &str, world: &WorldCard, map: &MapDef, player: &PlayerCard, all_npcs: &[NpcCard])` |
| `phase_post_generation` | `<B>(service: &B, ctx: &GameServiceContext, state: &mut GameState, input: &str, narration_text: &str)` | `(&self, state: &mut GameState, input: &str, narration_text: &str)` |
| `phase_engine_commit` | `(state: &GameState, narration_text: &str, quantifier_result: &QuantifierResult)` | **No change** — doesn't use service/ctx |
| `phase_trigger_continuation` | `<B>(service: &B, ctx: &GameServiceContext, mut state: GameState, trigger: &StoredTriggerContext)` | Rename to `phase_trigger_continuation_raw` + `(&self, mut state: GameState, trigger: &StoredTriggerContext)` — **renamed** because `ActionPipeline` already has a public `phase_trigger_continuation` wrapper |
| `reconcile_post_trigger_npcs` | `<B>(service: &B, mut state: GameState, player_input: &str, continuation_text: &str) -> GameState` | `(&self, mut state: GameState, player_input: &str, continuation_text: &str) -> GameState` |
| `build_trigger_request` | `<B>(service: &B, ctx: &GameServiceContext, state: &GameState, narration_text: &str, world: &WorldCard, player: &PlayerCard, all_npcs: &[NpcCard], trigger_match: &TriggerMatch)` | `(&self, state: &GameState, narration_text: &str, world: &WorldCard, player: &PlayerCard, all_npcs: &[NpcCard], trigger_match: &TriggerMatch)` |
| `load_preset_and_response_length` | `(ctx: &GameServiceContext) -> Result<(PromptPreset, String), String>` | `(&self) -> Result<(PromptPreset, String), String>` |

**Body replacements** in each function:

| Was | Now |
|---|---|
| `save_state(ctx,` | `save_state(self.ctx,` |
| `save_message_and_snapshot(ctx,` | `save_message_and_snapshot(self.ctx,` |
| `service.assembler()` | `self.service.assembler()` |
| `service.complete(` | `self.service.complete(` |
| `service.run_post_generation_agents(` | `self.service.run_post_generation_agents(` |
| `ctx.cancel_token` | `self.ctx.cancel_token` |
| `ctx.world.global_rules` | `self.ctx.world.global_rules` |
| `ctx.settings` | `self.ctx.settings` |
| `ctx.preset_storage` | `self.ctx.preset_storage` |
| `persist_snapshot_failed(ctx,` | `self.persist_snapshot_failed(` |
| `persist(ctx,` | `self.persist(` |
| `error_return(ctx, &mut state, msg)` | `self.error_return(state, msg)` — ownership pass |
| `load_preset_and_response_length(ctx)` | `self.load_preset_and_response_length()` |
| `map_llm_error(` | stays unchanged |

**`error_return` clone fix**: Change `Ok((state.clone(), String::new(), String::new(), String::new()))` → `Ok((state, String::new(), String::new(), String::new()))`. Since `state` is now taken by value, no clone needed.

**`#[allow(clippy::too_many_arguments)]`**: Keep on `phase_narrate` (now 6 params, down from 8) and `build_trigger_request` (now 7 params, down from 9). Can remove from `phase_trigger_continuation_raw` (now 3 params).

### Step 2: Update `pipeline.rs` call sites

**File**: `chronicler_engine/src/application/action_pipeline/pipeline.rs`

In `run_from_input` (lines 59-171), replace every `phases::fn_name(self.service, self.ctx, ...)` with `self.fn_name(...)`:

| Line | Old | New |
|---|---|---|
| 68-76 | `phases::phase_narrate(self.service, self.ctx, state, &input, &world, &map, &player, &all_npcs)` | `self.phase_narrate(state, &input, &world, &map, &player, &all_npcs)` |
| 96-102 | `phases::phase_post_generation(self.service, self.ctx, &mut state, &input, &narration_text)` | `self.phase_post_generation(&mut state, &input, &narration_text)` |
| 108 | `phases::phase_engine_commit(&state, &narration_text, &quantifier_result)` | `self.phase_engine_commit(&state, &narration_text, &quantifier_result)` |
| 123-133 | `phases::build_trigger_request(self.service, self.ctx, &next_state, &narration_text, &world, &player, &all_npcs, trigger_match)` | `self.build_trigger_request(&next_state, &narration_text, &world, &player, &all_npcs, trigger_match)` |
| 139 | `phases::persist_snapshot_failed(self.ctx, &mut next_state, "post-engine snapshot")` | `self.persist_snapshot_failed(&mut next_state, "post-engine snapshot")` |
| 154-159 | `phases::reconcile_post_trigger_npcs(self.service, next_state, &input, &continuation_text)` | `self.reconcile_post_trigger_npcs(next_state, &input, &continuation_text)` |

In `phase_pre_main_snapshot` (line 173-178):
- Replace `phases::persist_snapshot_failed(self.ctx, &mut state, "pre-main snapshot")` → `self.persist_snapshot_failed(&mut state, "pre-main snapshot")`
- **Add bug fix** — check the return value (see Step 3 below)

In `phase_trigger_continuation` (line 186):
- Replace `phases::phase_trigger_continuation(self.service, self.ctx, state, trigger)` → `self.phase_trigger_continuation_raw(state, trigger)`

In `phase_finalize` (line 208):
- Replace `phases::persist(self.ctx, state)` → `self.persist(state)`

In `handle_cancellation` (line 216):
- Replace `phases::persist(self.ctx, &state)` → `self.persist(&state)`

### Step 3: Fix `phase_pre_main_snapshot` bug

**File**: `pipeline.rs`, `phase_pre_main_snapshot` method

Current code (line 173-179):
```rust
fn phase_pre_main_snapshot(&self, mut state: GameState) -> PipelineResult<GameState> {
    tracing::info!("Pipeline ▶ Narrating");
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    phases::persist_snapshot_failed(self.ctx, &mut state, "pre-main snapshot");
    Ok(state)
}
```

Replace with:
```rust
fn phase_pre_main_snapshot(&self, mut state: GameState) -> PipelineResult<GameState> {
    tracing::info!("Pipeline ▶ Narrating");
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    if self.persist_snapshot_failed(&mut state, "pre-main snapshot") {
        self.phase_finalize(&mut state);
        return Ok(state);
    }
    Ok(state)
}
```

Rationale: every other `persist_snapshot_failed` call site uses it as a guard. The original code (pre-decomposition) set `GenerationStatus::Error` and `save_state`, then continued — but `run_from_input` line 83-92 checks `error_message().is_some()` and early-returns. The guard makes the early-return explicit rather than relying on the downstream check. Consistent with every other persist-failure site in the pipeline.

### Step 4: Update `retry.rs`

**File**: `chronicler_engine/src/application/action_pipeline/retry.rs`

Line 127: Replace `phases::reconcile_post_trigger_npcs(backend, s, &input_text, &continuation_text)` → `pipeline.reconcile_post_trigger_npcs(s, &input_text, &continuation_text)` (uses the existing `pipeline` from line 123).

Line 8: Remove `use super::phases;` — no longer needed.

### Step 5: Verification

1. `cd chronicler_engine && python build.py` — fmt + clippy + all tests pass
2. Search `phases::` in `pipeline.rs` and `retry.rs` — zero hits
3. Search `state.clone()` in `phases.rs` — only `commit_trigger_narration(state.clone(), ...)` (line ~244, pre-existing) and `apply_npc_events(state.clone(), ...)` (line ~298, pre-existing). The `error_return` clone is gone.

## Critical files & anchors

1. **`phases.rs`** — all 10 functions become `impl` methods. `error_return` (line 48): take `state` by value, remove clone.
2. **`pipeline.rs:59-219`** — call sites simplified. `phase_pre_main_snapshot` (line 173): add `persist_snapshot_failed` guard.
3. **`retry.rs:109-139`** — `reconcile_post_trigger_npcs` call changes from `phases::fn(backend, ...)` to `pipeline.method(...)`. Remove `use super::phases;`.

## Assumptions & contingencies

- **Split `impl` blocks**: Standard Rust. If compilation rejects it, fall back to free functions passing `(&B, &GameServiceContext)` as a `PhaseDeps(&'a B, &'a GameServiceContext)` tuple — one struct, not another wrapper layer.
- **Pre-main snapshot guard is a behavioral change**: Previously a pre-main persist failure was silently swallowed. After the fix, the pipeline aborts with error status. Since `persist_snapshot_failed` sets `GenerationStatus::Error`, the downstream code at line 83-92 would have caught it anyway — the fix just makes it explicit. If this proves wrong in practice, remove the guard and add a `// Best-effort: do not guard` comment.
- **Not fixing `commit_trigger_narration(state.clone(), ...)` and `apply_npc_events(state.clone(), ...)`**: Pre-existing. Fixing requires engine API changes (accept `&GameState` + return new state). Separate refactor.

## Status: COMPLETED (2026-06-16)
