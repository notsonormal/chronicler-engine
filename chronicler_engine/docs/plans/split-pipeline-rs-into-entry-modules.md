# Split `pipeline.rs` into entry modules

> **Status:** Chartered. Run after the current test refactor is settled and there is budget for structural cleanup.
> **Scope:** 8 SP code + test reorganization. No behavior change.

## Summary

`src/application/pipeline/pipeline.rs` is 810 lines and mixes four distinct responsibilities:

- the action entry path (`process_action`, `continue_narration`, `execute_action`)
- the retry entry path (`retry`, `retry_last_response`, `check_retry_anchor`, `retry_main_narration`, `retry_event_continuation`, `handle_retry_outcome`)
- the retrigger entry path (`retrigger`, `retrigger_event`)
- shared orchestration (`ActionPipeline` struct, constructors, `run_from_input`, `finalize_phase_error`, `persist_generation_error`, `load_world_bundle`, `phase_trigger_continuation`, `run_post_generation_agents`)

Its sibling test file `pipeline_tests.rs` has grown to 1499 lines. Splitting the source into focused entry modules will naturally split the tests into smaller sibling `_tests.rs` files.

## Key Changes

- New modules under `src/application/pipeline/`:
  - `action.rs` — action entry path
  - `retry.rs` — retry entry path
  - `retrigger.rs` — retrigger entry path
  - `core.rs` — shared `ActionPipeline` state, constructors, and helpers
- Move the duplicated gate-claim / spawn ceremony into a single `claim_and_spawn` helper in `spawn.rs` or `core.rs` so `process_action`, `retry`, and `retrigger` delegate to it.
- `pipeline/mod.rs` re-exports `ActionPipeline` from `core.rs` and declares the new modules.
- Delete or shrink `src/application/pipeline/pipeline.rs` once its contents are moved.
- Sibling test files:
  - `core_tests.rs` for shared / orchestration tests
  - `action_tests.rs` for action entry tests
  - `retrigger_tests.rs` for retrigger tests
  - keep `retry_tests.rs` and move its share of tests there
- Move tests from `pipeline_tests.rs` into the appropriate new sibling files.

## Implementation

### Phase 1: Extract shared core and spawn helper

- [ ] #### Task 1: Create `core.rs` and `claim_and_spawn` helper (3 SP)
  - Move `ActionPipeline` struct, constructors, `rebind_for_test`, `is_shutting_down`, `reset_persisted_status`, `load_world_bundle`, `finalize_phase_error`, `persist_generation_error`, `phase_trigger_continuation`, `run_post_generation_agents`, and `run_from_input` into `core.rs`.
  - Extract the common `heal_stale` → `try_claim` → `spawn_pipeline_task` sequence into a `claim_and_spawn` helper used by `action`, `retry`, and `retrigger`.
  - Update `mod.rs` to declare `core` and re-export `ActionPipeline`.

### Phase 2: Create entry modules

- [ ] #### Task 2: Move retry and retrigger paths to their own modules (3 SP)
  - Move retry-related items into `retry.rs` and retrigger-related items into `retrigger.rs`.
  - Move action-related items into `action.rs`.

### Phase 3: Split tests

- [ ] #### Task 3: Reorganize `pipeline_tests.rs` into sibling files (2 SP)
  - Create `core_tests.rs`, `action_tests.rs`, and `retrigger_tests.rs`.
  - Move tests from `pipeline_tests.rs` to the matching module test file.
  - Keep `retry_tests.rs`; move its share of retry tests there.

## Test Plan

- `cargo check -p chronicler_engine --tests`
- `cargo nextest run -p chronicler_engine`
- `python chronicler_engine/scripts/validate_feature_spec.py`
- No gaps, no orphans, no behavior change.

## Assumptions

- This is a file-level refactor with no behavior change.
- `PipelineRun` and phase logic stay in `phases.rs`.
- `phase_error.rs` and `spawn.rs` stay unchanged except for the new `claim_and_spawn` helper.
- New test files follow the sibling `_tests.rs` convention.

## Story Points

8 SP total.

## Relationships

- **Motivated by:** pre-merge review findings on `testing-refactor` (duplicate gate/claim ceremony, `pipeline_tests.rs` past 1k lines).
- **Precedes:** any further feature work in the action pipeline.
