# Review Fixes — Pipeline Decomposition Quality

**Status:** Implemented 2026-06-16
**Archived:** 2026-06-17

## Context

Thermo-nuclear review of the pipeline/run decomposition identified 4 actionable findings. The decomposition itself is sound (pipeline.rs 720→253, run.rs 495→261+282); the fixes target specific quality problems introduced or preserved by the extraction. None are design overhauls.

## Approach

### Step 1 — Remove unnecessary `next_state.clone()` in `run_from_input`

**File:** `chronicler_engine/src/application/action_pipeline/pipeline.rs:150`

**Current code:**
```rust
match self.phase_trigger_continuation(next_state.clone(), &request) {
```

**Change:** Replace `next_state.clone()` with `next_state` (move by value). The `Err` branch returns immediately (`Err(e) => return Err(e)`), and the `Ok` branch overwrites `next_state` — so the original is never used after the call. The clone deep-copies `HashMap<String, NpcCard>`, narrative history, and Arc internals for no reason.

**Depends on:** nothing. Independent.

### Step 2 — Remove `state.clone()` in `phase_post_trigger_reconcile` wrapper

**File:** `chronicler_engine/src/application/action_pipeline/pipeline.rs:199-201`

**Current code:**
```rust
match phases::reconcile_post_trigger_npcs(self.service, state.clone(), input, continuation_text) {
```

The `state.clone()` exists to preserve `state` for the `Err` branch (line 208: `let mut state = state;`). But `reconcile_post_trigger_npcs` returns `Result<GameState, EngineError>` — on `Err`, the `GameState` is lost (the function consumed it by taking `mut state: GameState`).

**Better approach:** Move error mapping into `phases::reconcile_post_trigger_npcs` itself, returning `GameState` directly (never `Err`). This matches the pattern already used by `phase_trigger_continuation`, which signals errors through `state.status` and returns `PipelineResult<(GameState, String)>`. The function already does this internally for the normal path — the `Err` from `apply_npc_events` can just set `GenerationStatus::Error` on state and return the state.

**Concrete changes:**

2a. Change `phases::reconcile_post_trigger_npcs` return type from `Result<GameState, EngineError>` to `GameState`. On `apply_npc_events` error, set `GenerationStatus::Error` on the state and log, then return the state.

2b. Remove the `phase_post_trigger_reconcile` method from `ActionPipeline` entirely — it was a thin wrapper that only added a tracing line and error mapping. Instead, call `phases::reconcile_post_trigger_npcs` directly from `run_from_input` and `retry_event_continuation`. Add the `tracing::info!` line inside the phases function itself (matching `phase_trigger_continuation`'s pattern).

**Callsites to update:**
- `pipeline.rs:155` — `self.phase_post_trigger_reconcile(next_state, &input, &continuation_text)` → `phases::reconcile_post_trigger_npcs(self.service, next_state, &input, &continuation_text)`
- `retry.rs:126` — `pipeline.phase_post_trigger_reconcile(s, &input_text, &continuation_text)` → `phases::reconcile_post_trigger_npcs(backend, s, &input_text, &continuation_text)`

2c. Add `use super::phases;` to `retry.rs` (it doesn't currently import `phases`).

**Depends on:** nothing. Independent from Step 1.

### Step 3 — Fix `ArrivalTaskContext::run` using `AppSettings::default()` for backend selection

**File:** `chronicler_engine/src/bootstrap/init_game.rs:166-167`

**Current code:**
```rust
let backend = crate::narrative::llm::get_llm_backend_for(
    &AppSettings::default().narration_connection(),
    Some(Arc::clone(&self.storage)),
);
```

This uses default connection settings for LLM backend selection, but the function reads the real settings for token limits. If the user configured e.g. Ollama instead of OpenRouter, arrival narration hits the wrong backend.

**Fix:** Store the `Connection` in `ArrivalTaskContext` (it's already resolved in `spawn_arrival_task_if_needed` at line 257 where `guard.narration_connection()` is called). Pass it through and use it in `run()`.

**Concrete changes:**

3a. Add `connection: crate::model::settings::Connection` field to `ArrivalTaskContext`.

3b. In `spawn_arrival_task_if_needed`, extract `conn` from `with_settings` alongside the existing values, and store it in `ArrivalTaskContext`.

3c. In `ArrivalTaskContext::run`, replace `AppSettings::default().narration_connection()` with `&self.connection`.

**Depends on:** nothing. Independent.

### Step 4 — Restore `Copy` on `NpcContext`

**File:** `chronicler_engine/src/narrative/prompt/types.rs:10`

**Current code:**
```rust
#[derive(Debug, Clone)]
pub struct NpcContext<'a> {
```

`NpcContext` is two `&[T]` fat pointers. `Copy` is the correct trait for this — it copies no data, just two (pointer, len) pairs. Removing `Copy` forces explicit `.clone()` at every callsite (assembler.rs:67) with no safety benefit. The changelog rationale ("shouldn't be implicitly copied") misunderstands what `Copy` on references does — it never clones the underlying data.

**Change:** Add `Copy` to the derive. Remove `.clone()` at the sole callsite in `assembler.rs:67` (change `npcs: context.npcs.clone()` to `npcs: context.npcs`).

**Callsites to update:** Only `chronicler_engine/src/narrative/prompt/assembler.rs:67`.

**Depends on:** nothing. Independent.

## Critical files & anchors

- `chronicler_engine/src/application/action_pipeline/pipeline.rs:150` — the unnecessary clone (Step 1)
- `chronicler_engine/src/application/action_pipeline/pipeline.rs:192-215` — thin wrapper to remove (Step 2b)
- `chronicler_engine/src/application/action_pipeline/phases.rs:268-304` — `reconcile_post_trigger_npcs` return type change (Step 2a)
- `chronicler_engine/src/bootstrap/init_game.rs:111-169` — `ArrivalTaskContext` struct + `run()` (Step 3)
- `chronicler_engine/src/narrative/prompt/types.rs:10` — `NpcContext` derives (Step 4)

## Verification

Run from `chronicler_engine/`:

```
python build.py
```

This runs `cargo fmt`, `cargo clippy`, and `cargo test` with coverage. All existing tests pass, plus these specific checks:

- **Step 1:** The pipeline still completes trigger continuation flows. Existing test `test_pipeline_trigger_happy_path` exercises `run_from_input` with a trigger match. If the clone was needed, this test fails.
- **Step 2:** `test_phase_trigger_continuation_cancels_at_start` and `test_trigger_continuation_save_post_trigger_error` still pass. `reconcile_post_trigger_npcs` now returns `GameState` directly, so no callers match on `Err` — search for `reconcile_post_trigger_npcs` to confirm zero `Err` matches remain.
- **Step 3:** The `ArrivalTaskContext` struct has a `connection` field. The `get_llm_backend_for` call in `run()` uses `&self.connection` instead of `AppSettings::default()`. Existing `test_trigger_continuation_save_post_trigger_error` and integration tests cover the arrival path indirectly — no new test needed since this is a bugfix in an untested code path (arrival narration runs at startup, no unit test covers it).
- **Step 4:** `NpcContext` is `Copy`. `assembler.rs:67` uses `context.npcs` without `.clone()`. `cargo clippy` would flag if `Copy` + `Clone` diverge.

## Assumptions & contingencies

- **`reconcile_post_trigger_npcs` error mapping:** On `apply_npc_events` error, set `GenerationStatus::Error` on state, log, and return the state — matching how `phase_trigger_continuation` handles `commit_trigger_narration` errors. If tests assert on `Err` return, adjust them to check `state.narrative.input_buffer.status` instead. Current tests (`pipeline_tests.rs`, `retry_tests.rs`) do not directly test `reconcile_post_trigger_npcs` in isolation, so no test adjustments are expected.
- **`ArrivalTaskContext` `Connection` field:** `Connection` must be `Clone` (it already derives `Clone` — confirmed in settings.rs). The `with_settings` closure already resolves `conn` at line 257; we just add `conn` to the tuple and store it.
- **`Copy` on `NpcContext`:** If `PromptContext` or `LayerRenderer` holds a `&NpcContext` (not owned), `Copy` on `NpcContext` is irrelevant to the `context.npcs.clone()` callsite — but in this codebase `PromptContext.npcs` is `NpcContext<'a>` (owned), so `Copy` lets us assign without `.clone()`.
