# Fix orphan `orchestrator_tests.rs` and broken `check_test_file_pairing` guardrail

## Summary

`src/application/orchestrator_tests.rs` has no matching `orchestrator.rs` source file — it was renamed from `application_service_tests.rs` when the façade was deleted (ticket 16), but the new name doesn't match any module. The `check_test_file_pairing` guardrail in `tests/infrastructure/guardrails/location.rs` should catch this but doesn't: it silently skips orphan checks when the parent directory has a `mod.rs`. Additionally, `orchestrator_tests.rs` contains two test helpers (`make_test_service` / `make_test_service_with_agent`) that duplicate canonical helpers already in `test_support::context.rs`. Fix all three — split the test file to match real modules, consolidate the duplicated helpers onto the canonical ones, and fix the guardrail logic so it catches this class of orphan.

## Key Changes

- **Split `orchestrator_tests.rs`** into two files matching the modules their tests actually exercise:
  - `src/application/games/view_query_tests.rs` — the 9 `GameViewQuery` tests (lines 73–161: `test_get_generating_status_*`, `test_get_current_game_name_*`, `test_list_latest_llm_messages_*`, `test_get_story_log_entries_*`, `test_get_current_room_view_*`, `test_get_npc_headshots_*`, `test_get_debug_state_*`, `test_active_quantifier_prompt_*`).
  - `src/application/generation/gate_tests.rs` — the `GenerationGate` tests (lines 168–234: `test_reset_generating_status_sets_idle`, `test_boot_heal_resets_stale_generating_status`, plus the `SyncQuantifierAgent` helper).
  - The remaining 7 pipeline execution tests (lines 304–509: `test_execute_action_*`, `test_phase_transitions_*`, `test_narration_saved_*`) move to the existing `src/application/pipeline/pipeline_tests.rs`.
  - Delete `orchestrator_tests.rs` and remove its `mod orchestrator_tests;` from `src/application/mod.rs`.
  - Add `mod view_query_tests;` to `src/application/games/mod.rs` and `mod gate_tests;` to `src/application/generation/mod.rs` (both `#[cfg(test)]`).
- **Consolidate duplicated test helpers** — delete `make_test_service` and `make_test_service_with_agent` from `orchestrator_tests.rs`; replace their call sites with the canonical helpers from `test_support::context.rs`:
  - `make_test_service(recorder, quantifier_provider)` → `make_test_pipeline_with_mock_quantifier(Arc::new(Storage::new_in_memory()), recorder, quantifier_provider)`
  - `make_test_service_with_agent(recorder, agent)` → `make_test_pipeline_with_backends(Arc::new(Storage::new_in_memory()), recorder, AgentRegistry::with_agent(agent))`
  - The inline pipeline construction in `test_boot_heal_resets_stale_generating_status` (lines 253–268) also duplicates the canonical helper — replace it with `make_test_pipeline_with_backends(...)`.
- **Fix `check_test_file_pairing`** in `tests/infrastructure/guardrails/location.rs`:
  - The bug: the `else if !parent_has_mod_rs` branch means that when a directory has a `mod.rs`, orphan `_tests.rs` files are silently allowed. The `parent_has_mod_rs` check should not be an escape hatch — a `_tests.rs` file next to a `mod.rs` still needs a matching `<name>.rs` sibling or a `<name>/mod.rs` subdirectory.
  - The fix: remove the `parent_has_mod_rs` guard from the orphan-detection logic. A test file `foo_tests.rs` is valid only if `foo.rs` exists in the same directory or `foo/mod.rs` exists as a subdirectory. If neither exists, it's an orphan — regardless of whether the parent directory has its own `mod.rs`.
  - Remove the two pre-existing TODO comments at `location.rs:4–6` and `location.rs:45–47` that describe this bug.
- **Add unit tests** for `check_test_file_pairing` in `tests/infrastructure/guardrails/structure_tests.rs` (or a new `location_tests.rs` if that better matches the file-pairing convention): one test asserting an orphan `_tests.rs` next to a `mod.rs` is flagged, one asserting a valid pairing passes.

## Implementation

### Phase 1: Fix the guardrail

- [ ] #### Task 1.1: Fix `check_test_file_pairing` logic (1 SP)
  - In `tests/infrastructure/guardrails/location.rs`, rewrite the orphan-detection branch in `check_test_file_pairing`. Current logic:
    ```
    if !has_source_file {
        if has_module_dir && !parent_has_mod_rs { /* orphan outside module dir */ }
        else if !parent_has_mod_rs { /* orphan */ }
    }
    ```
    New logic:
    ```
    if !has_source_file && !has_module_dir {
        // orphan — no matching <name>.rs or <name>/mod.rs
    }
    ```
  - Delete the two TODO comments (lines 4–6 and 45–47).

- [ ] #### Task 1.2: Add guardrail unit tests (1 SP)
  - Add tests that call `check_test_file_pairing` with synthetic paths (the function takes `&str` paths and uses `Path` methods, so tests can pass path strings that don't exist on disk — the function checks `expected_source.exists()` and `module_mod_rs.exists()` against the real filesystem, so tests need to use paths that exist or mock accordingly).
  - Test cases: (a) orphan `_tests.rs` in a directory with `mod.rs` → violation, (b) valid `_tests.rs` with matching `.rs` → no violation, (c) valid `_tests.rs` with matching `/mod.rs` → no violation.

### Phase 2: Consolidate duplicated test helpers

- [ ] #### Task 2.1: Replace `make_test_service` / `make_test_service_with_agent` with canonical helpers (1 SP)
  - Delete `make_test_service` from `orchestrator_tests.rs` (lines 210–232). Replace its 4 call sites (lines 304, 333, 355, 396) with `make_test_pipeline_with_mock_quantifier(Arc::new(Storage::new_in_memory()), recorder, quantifier_provider)` from `test_support::context`.
  - Delete `make_test_service_with_agent` from `orchestrator_tests.rs` (lines 275–301). Replace its 2 call sites (lines 425, 469) with `make_test_pipeline_with_backends(Arc::new(Storage::new_in_memory()), recorder, AgentRegistry::with_agent(agent))`.
  - Replace the inline pipeline construction in `test_boot_heal_resets_stale_generating_status` (lines 253–268) with `make_test_pipeline_with_backends(...)`.

### Phase 3: Split the orphan test file

- [ ] #### Task 3.1: Create `view_query_tests.rs` (1 SP)
  - Create `src/application/games/view_query_tests.rs`.
  - Move the 9 `GameViewQuery` tests and the `minimal_app`/`minimal_app_no_game`/`minimal_state` helpers they use from `orchestrator_tests.rs`.
  - Add `#[cfg(test)] mod view_query_tests;` to `src/application/games/mod.rs`.

- [ ] #### Task 3.2: Create `gate_tests.rs` (1 SP)
  - Create `src/application/generation/gate_tests.rs`.
  - Move `test_reset_generating_status_sets_idle`, `test_boot_heal_resets_stale_generating_status`, and the `SyncQuantifierAgent` struct + impl.
  - Add `#[cfg(test)] mod gate_tests;` to `src/application/generation/mod.rs`.

- [ ] #### Task 3.3: Move pipeline execution tests to `pipeline_tests.rs` (1 SP)
  - Move the 7 remaining tests (`test_execute_action_*`, `test_phase_transitions_*`, `test_narration_saved_*`) into the existing `src/application/pipeline/pipeline_tests.rs`.
  - Move any helpers they need that aren't already in `pipeline_tests.rs`.

- [ ] #### Task 3.4: Delete `orchestrator_tests.rs` and unregister (1 SP)
  - Delete `src/application/orchestrator_tests.rs`.
  - Remove `mod orchestrator_tests;` from `src/application/mod.rs`.

### Phase 4: Verify

- [ ] #### Task 4.1: Build and guardrails (1 SP)
  - `cargo check --all-targets --all-features` green.
  - `cargo nextest run --test guardrails` green (the fixed `check_test_file_pairing` must pass — no orphans remain).
  - `python chronicler_engine/build.py` green.

## Test Plan

- Guardrail unit tests for the fixed `check_test_file_pairing`.
- Full build confirms the guardrail walker passes against the real codebase (no orphan `_tests.rs` files remain).
- All existing tests pass from their new locations.

## Assumptions

- The `GameViewQuery` tests and `GenerationGate` tests are self-contained enough to move with only import adjustments.
- The `SyncQuantifierAgent` helper is only used by the two tests moving to `gate_tests.rs`; if the pipeline execution tests also use it, it stays with them or is duplicated.
- The `make_test_service`/`make_test_service_with_agent` helpers are deleted entirely (Phase 2); their call sites use the canonical `test_support::context` helpers instead, so there is nothing to move in Phase 3.
