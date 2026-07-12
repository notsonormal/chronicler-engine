# T13: T9-01 Tier-1 Cleanup + Dynamic-Rooms Bug Fix

**Status:** Planning, deferred from T9-01 after-plan-workflow
**Date:** 2026-07-11
**Depends on:** T9-01 (chunks 1-4 complete)
**Blocks:** none
**Priority:** P1 (one latent correctness bug + 8 mechanical cleanups)

## Summary
9 items from T9-01 review triage: 1 latent correctness bug + 8 mechanical type/hygiene/dead-code cleanups. Bundle into 3 implementation phases: bug fix first (small, isolated), then type alignment (touches pipeline+retry error flow), then mechanical cleanup (no behavior change). All items verified against HEAD at planning time.

## Key Changes
- **Bug fix**: `phases.rs:101` `phase_narrate` adds `.or_else(dynamic_rooms)` fallback so dynamic rooms (created by `create_dynamic_room`) resolve correctly. New regression test locks the fix.
- **Type alignment**: `fetch_world_bundle` (pipeline.rs) and `fetch_world_bundle_for_retry` (retry.rs) return `Result<_, EngineError>` instead of `String` / `Option`. WorldBundle alias dropped. Regression tests assert on **property** (`GenerationStatus::Error(_)` was set), not on message wording.
- **Mechanical**: 5 dead helpers deleted, 2 misleading helper names merged, `unwrap_or(0)` → `expect`, 2 dead one-line `find_room_*` wrappers inlined, 3 unused args dropped from `action_processing.rs`.

## Implementation

### Phase 1: Latent bug fix

- [ ] #### Task 1.1: Fix `phase_narrate` missing dynamic-room fallback (1 SP)
  - File: `src/application/action_pipeline/phases.rs:99-105`
  - Change `let Some(room) = inputs.map.get_room_by_id(&state.movement.current_room_id)` to use the same shape as 4 other inline sites in the codebase:
    ```rust
    let Some(room) = inputs
        .map
        .get_room_by_id(&state.movement.current_room_id)
        .or_else(|| state.movement.dynamic_rooms.get(&state.movement.current_room_id))
    else {
        return self.error_return(state, "Room not found".to_string());
    };
    ```
  - **Required**: add regression test in `pipeline_tests.rs` (alongside the existing `orchestrator_records_error_when_world_missing` shape) that:
    1. Builds `GameState::new("room1")`.
    2. Inserts a synthetic room into `state.movement.dynamic_rooms` keyed by `"dynamic_room"`.
    3. Sets `state.movement.current_room_id = "dynamic_room".to_string()`.
    4. Invokes the pipeline path that exercises `phase_narrate` (e.g. `pipeline.run_from_input(...)` with a mock backend).
    5. Asserts no `GenerationStatus::Error` is set and the outcome is `ActionOutcome::Completed`.
  - Verify: `cargo build` (0 errors); `cargo test --quiet` (1242 passed).

### Phase 2: Type alignment

- [ ] #### Task 2.1: Drop `WorldBundle` alias + change `fetch_world_bundle` return type (3 SP)
  - File: `src/application/action_pipeline/pipeline.rs`
  - Delete `type WorldBundle = ...` alias (line 37).
  - Change `fetch_world_bundle` signature to `fn fetch_world_bundle(app: &DefaultApplicationService, game_id: u64) -> Result<(Arc<WorldCard>, Arc<MapDef>, Arc<PersonaCard>, HashMap<String, NpcCard>), EngineError>`.
  - Add `#[allow(clippy::type_complexity)]` on the function.
  - Replace all `format!("...: {e}")` / `format!("...'...' not found")` with `EngineError::Storage(...)` propagation. Preserve the context key/world_id in the error message (for ops/debug visibility) but do not couple test assertions to it.
  - Update caller at `pipeline.rs:108` to consume `EngineError` (`state.narrative.input_buffer.status = GenerationStatus::Error(e.to_string())`).
  - Update regression tests `orchestrator_records_error_when_world_missing` and `orchestrator_records_error_when_persona_missing` at `pipeline_tests.rs`: change string-match (`msg.contains("world")`, `msg.contains("persona")`) to **property-match** — assert `matches!(state.narrative.input_buffer.status, GenerationStatus::Error(_))`. The fail-loud contract is the invariant; message wording is implementation detail.
  - Verify: `cargo build` (0 errors); `cargo clippy --all-targets -- -D warnings` (0); `cargo test --quiet` (1242 passed).

- [ ] #### Task 2.2: Align `fetch_world_bundle_for_retry` to `Result<_, EngineError>` (5 SP)
  - Files: `src/application/action_pipeline/retry.rs`, `src/application/action_pipeline/retry_tests.rs`
  - Change signature from `pub(crate) fn fetch_world_bundle_for_retry(app) -> Option<(Arc<MapDef>, HashMap<String, NpcCard>)>` to `pub(crate) fn fetch_world_bundle_for_retry(app) -> Result<(Arc<MapDef>, HashMap<String, NpcCard>), EngineError>`. Pure fetch — no side-effects.
  - Move the `save_retry_error` side-effect out of the helper into the caller (`retry_event_continuation` at `retry.rs:166`).
  - Update `retry_event_continuation` pattern:
    ```rust
    let (map, npcs_map) = match fetch_world_bundle_for_retry(app) {
        Ok(b) => b,
        Err(e) => {
            save_retry_error(app, e.to_string());
            return ActionOutcome::Completed;
        }
    };
    ```
  - Update test `test_fetch_world_bundle_for_retry_returns_none_on_world_fetch_error` at `retry_tests.rs:770`: assert `Err(_)` instead of `None`. Rename to `..._returns_err_on_world_fetch_error`. Use property-match for the integration assertion (any `GenerationStatus::Error(_)` is sufficient).
  - Verify: `cargo build` (0 errors); `cargo clippy --all-targets -- -D warnings` (0); `cargo test --quiet` (1242 passed).

### Phase 3: Mechanical cleanup

- [ ] #### Task 3.1: Delete 5 dead helpers in `tests/helpers/fixtures.rs` (1 SP)
  - File: `tests/helpers/fixtures.rs`
  - Delete: `create_test_game_state` (line 203), `create_navigation_test_map` (line 207), `create_simple_test_map` (line 274), `create_basic_test_state` (line 317), `create_basic_test_state_no_scenario` (line 321).
  - Verify: `cargo build` (0 errors).

- [ ] #### Task 3.2: Merge 2 misleading helper names in `pipeline_helpers.rs` (2 SP)
  - File: `tests/helpers/pipeline_helpers.rs`
  - Both `create_test_state_with_map` (line 8) and `create_test_state_with_trigger_npc` (line 12) have identical body `GameState::new("room1".to_string())` and misleading names. Merge into a single helper `create_minimal_test_state()` with the same body.
  - Update 25 callers: invariant_contract.rs:3 (118,179,421), retry.rs:294, arrival_persistence.rs:2 (135,215), retry_main.rs:10, sequence.rs:10.
  - Verify: `cargo build` (0 errors); `cargo test --quiet` (1242 passed).

- [ ] #### Task 3.3: Replace `unwrap_or(0)` with `expect` (1 SP)
  - File: `tests/helpers/sqlite_test_app_builder.rs:223`
  - Change `storage.save_snapshot(&snapshot).unwrap_or(0)` to `storage.save_snapshot(&snapshot).expect("snapshot save must succeed in test builder")`.
  - Verify: `cargo build` (0 errors).

- [ ] #### Task 3.4: Inline `find_room_in_map` / `find_room_in_world_map` + delete trivial tests (3 SP)
  - Files: `src/domain/engine/logic.rs` (definitions), `src/application/scenario.rs:24` (1 caller), `src/domain/engine/logic_tests.rs` (callers + tests).
  - Delete `find_room_in_map` (logic.rs:10, zero callers) and `find_room_in_world_map` (logic.rs:14, 5 callers).
  - Inline `map.get_room_by_id(...)` at 4 production callers (scenario.rs:24, logic.rs:19 inside `attempt_semantic_walk`, logic_tests.rs:70, 77, 134).
  - Verification step: read `test_attempt_walk_dangling_exit` (logic_tests.rs:76) and any other tests at the listed lines before deciding count. Decision tree:
    - Body uses `find_room_in_world_map` AND nothing else → delete.
    - Body uses `attempt_semantic_walk` with a real navigation scenario → preserve, inline the call.
  - Delete 2 or 3 trivial tests as determined above. Final count: **1239 or 1240 passed** after Phase 3.
  - Verify: `cargo build` (0 errors); `cargo test --quiet` (1239 or 1240 passed).

- [ ] #### Task 3.5: Drop unused args in `action_processing.rs` (3 SP)
  - Files: `src/domain/engine/action_processing.rs` (3 fn defs), 9 caller sites.
  - Remove `_map: &MapDef` from `update_npc_encounters_on_room_change` (line 65). Update 1 caller at `action_processing.rs:114`.
  - Remove `_npcs: &HashMap<String, NpcCard>` from `log_movement_completion` (line 79). Update 1 caller at `action_processing.rs:115`.
  - Remove `_world: &Arc<WorldCard>` from `execute_freeaction_impl` (line 172). Update 7 callers: `phases.rs:433`, `action_processing_tests.rs:5 sites (104,136,171,195,371,608)`, `invariant_contract.rs:3 sites (146,350,438)`.
  - Verify: `cargo build` (0 errors); `cargo test --quiet` (1239 or 1240 passed); `cargo clippy --all-targets -- -D warnings` (0).

## Test Plan

Per-phase verification (full `python build.py` at end):
- After Phase 1: `cargo build` + `cargo test --quiet` → **1242 passed** (added 1 dynamic-room regression test in Task 1.1).
- After Phase 2: `cargo build` + `cargo clippy --all-targets -- -D warnings` + `cargo test --quiet` → **1242 passed** (test renames + property-shape rewrites; net count unchanged).
- After Phase 3: `cargo build` + `cargo clippy --all-targets -- -D warnings` + `cargo test --quiet` → **1239 or 1240 passed** (delete 2–3 trivial tests per Task 3.4 verification step).
- Final: `python build.py` from workspace root.

## Per Task/Sub Task Validation Steps

Each subagent must end their chunk by pasting the relevant `cargo` output (build errors count, clippy warnings count, test pass count). Primary verifies before proceeding to next chunk.

## Assumptions

- Tier-1 #1 (WorldBundle alias removed) — explicit user direction during T9-01 review triage.
- Tier-1 #2 / #3 (EngineError everywhere) — explicitly confirmed via plan-mode question.
- Logic_tests.rs trivial tests (`test_get_room_by_id_missing`, `test_get_room_by_id_existing`) are deletable — they wrap a function being removed. Final count for `test_attempt_walk_dangling_exit` is decided at implementation time per Task 3.4 verification step.
- EngineError variants (`Storage`, `Internal`) provide sufficient context for ops/debug; tests do not couple to message wording.
- Tier-2 #9 (retry double-fetch unification) stays out of scope per user.
- Tier-2 #11 (QuantifierCtx to drop allow attribute) stays out of scope per user.
- Tier-2 #12 (seed_test_world_into_storage fidelity) stays out of scope per user.
- Single-commit policy continues from T9-01 — no intermediate commits between phases.

## Out of Scope

- Tier-2 #9 (retry double-fetch), #11 (QuantifierCtx), #12 (test fidelity smell).
- T9-01 architectural documentation updates.
- T12 browser test flakiness plan (separate plan, separate ticket).
- Plan-mode-related session state (`save_retry_error` helpers, `TestOutcome` variants).

## Reference

- T9-01 base: `.scratch/t9-world-snapshot-removal/issues/01-world-snapshot-dies-engine-unbundles.md`
- T9-01 after-plan-workflow report and tier prioritization: see session recall entries.
- Prior T9-01 commits and code changes uncommitted in working tree.

## Review Decisions Captured

1. **Issue 1 (Decision A)**: Dynamic-room regression test in Task 1.1 promoted to **Required**.
2. **Issue 2 (Decision: Move as planned)**: `save_retry_error` side-effect moves to caller in Task 2.2; no docstring added — any caller is expected to handle `Result<_, EngineError>` properly.
3. **Issue 3 (Decision A)**: Regression-test assertions in Task 2.1 and 2.2 changed from string-match (`msg.contains("world")`) to **property-match** (`GenerationStatus::Error(_)` set). Fail-loud contract is the invariant; message wording is implementation detail.
4. **Issue 4 (Decision A)**: Task 3.4 final test count deferred to implementation. Verification step added — read `test_attempt_walk_dangling_exit` before deciding whether to delete.
5. **Issue 5 (Decision A)**: Test Plan section pass counts updated to match Issue 1's promoted regression test.
