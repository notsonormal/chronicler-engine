# Port lifecycle + arrival unit tests down from the component tier (ticket 12)

## Summary

Dissolve the last two component-tier files: port 5 net-new branch-coverage
tests down to unit tier (2 into existing `catalogue_tests.rs`, 3 into new
`arrival_service_tests.rs`), delete 13 component-tier tests across 2 files,
unwire them from `tests/integration/mod.rs`, remove the now-empty
directories. Pure mechanical work; all dispositions settled in ticket 06's
asset. No spec changes, no SCENARIO tags, no new production code.

**Plan-review fixes applied:**
- **Finding 1 (state coverage):** test #1 uses
  `TestDataBuilder::default_test().build().seed_into(&storage)` (world
  *with* scenario) instead of `seed_test_world_into_storage` (world
  *without* scenario). The original `test_arrival_narration_survives_reload`
  used a scenario world; using a no-scenario world makes
  `state.inject_scenario_logs` a no-op, silently dropping that branch.
  Test #1 now uses `room_id = "room_1"` (the scenario's starting room).
- **Finding 2 (dead weight):** all 3 arrival tests construct `MessageService`
  directly instead of building a full `WiredApp` + `AppState` + a pipeline
  that `ArrivalTaskContext::run` never touches. Drops 5 imports
  (`AppState`, `build_test_wired_app`, `make_test_pipeline_with_mock_quantifier`,
  `PresetType`, `PromptPreset`) and ~25 lines of dead setup. Matches
  `src/application/message_service_tests.rs` (the closest structural
  neighbor), not `orchestrator_tests.rs`/`pipeline_tests.rs` (which justify
  the heavy harness because they exercise the pipeline).

## Key Changes

- **Add** `create_game_persists_scenario_message_and_swipe` and
  `delete_game_succeeds_silently_for_nonexistent_game` to
  `chronicler_engine/src/application/games/catalogue_tests.rs` (uses
  existing `seeded_catalogue()` helper).
- **Create** `chronicler_engine/src/application/arrival_service_tests.rs`
  with 3 tests, each constructing `MessageService::new(Arc::clone(&storage))`
  directly: `run_produces_and_persists_narration`,
  `run_falls_back_to_fresh_state_on_load_failure`,
  `run_returns_early_without_narration_on_world_fetch_failure`.
- **Wire** `#[cfg(test)] mod arrival_service_tests;` into
  `chronicler_engine/src/application/mod.rs`.
- **Delete** `chronicler_engine/tests/integration/application/lifecycle.rs`
  (10 tests) and `chronicler_engine/tests/integration/flow/arrival_persistence.rs`
  (3 tests).
- **Unwire** `mod lifecycle;` and `mod flow_arrival_persistence;` (plus
  their `#[path = ...]` lines) from `chronicler_engine/tests/integration/mod.rs`.
- **Remove** the empty directories `tests/integration/application/` and
  `tests/integration/flow/`.

## Implementation

### Phase 1: Add the 5 unit-tier tests

- [ ] #### Task 1.1: Add 2 tests to `catalogue_tests.rs` (3 SP)
  - [ ] ##### SubTask 1.1.1: Add `MessageType` import (1 SP)
    - Add `use crate::domain::model::state::message_types::MessageType;` to
      `chronicler_engine/src/application/games/catalogue_tests.rs` imports
      (group 3 `crate::`, after existing `use crate::test_support::TestDataBuilder;`).
  - [ ] ##### SubTask 1.1.2: Add `create_game_persists_scenario_message_and_swipe` (1 SP)
    - Use `seeded_catalogue()` (its `TestDataBuilder::default_test()` world has
      a scenario → `build_fresh_initial_state` emits 1 Narration).
    - Call `catalogue.create_game(&world_key, &persona_key).expect("ok")`.
    - `let messages = storage.load_message_rows().unwrap();`
    - `let narrations: Vec<_> = messages.into_iter().filter(|m| m.message_type == MessageType::Narration).collect();`
    - `assert_eq!(narrations.len(), 1, "exactly one scenario Narration should be persisted");`
    - `let swipe_count = storage.count_swipes_for_message(narrations[0].id).unwrap();`
    - `assert!(swipe_count > 0, "scenario message should have at least one swipe");`
  - [ ] ##### SubTask 1.1.3: Add `delete_game_succeeds_silently_for_nonexistent_game` (1 SP)
    - `let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();`
    - `catalogue.delete_game(99999).expect("delete_game should succeed silently for nonexistent game");`
      (covers idempotent `DELETE FROM games WHERE id=?` no-row branch).

- [ ] #### Task 1.2: Create `arrival_service_tests.rs` with 3 tests (5 SP)
  - [ ] ##### SubTask 1.2.1: Wire `mod arrival_service_tests;` (1 SP)
    - Add `#[cfg(test)] mod arrival_service_tests;` to
      `chronicler_engine/src/application/mod.rs` (alphabetically after
      `mod arrival_service;`, before `mod games;`).
  - [ ] ##### SubTask 1.2.2: Create file header + imports (1 SP)
    - `chronicler_engine/src/application/arrival_service_tests.rs` first line:
      `//! Unit tests for \`ArrivalTaskContext\`.` (plain summary, no DOC
      anchor — guardrail ADR-028 for `_tests.rs`).
    - Imports (group order std → crate; no external, no `AppState`, no
      `WiredApp`, no pipeline builder, no `PresetType`/`PromptPreset`):
      ```
      use std::sync::Arc;

      use crate::adapters::driven::llm::providers::MockBackend;
      use crate::adapters::driven::storage::{Storage, TestOverride};
      use crate::application::arrival_service::ArrivalTaskContext;
      use crate::application::message_service::MessageService;
      use crate::application::ports::llm_provider::LlmProvider;
      use crate::domain::model::character::NpcCard;
      use crate::domain::model::state::game_state::GameState;
      use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
      use crate::domain::model::state::generation_status::GenerationStatus;
      use crate::domain::model::state::message_types::MessageType;
      use crate::test_support::{
          default_test_preset_storage, make_test_recorder_with_storage, seed_test_world_into_storage,
          TestDataBuilder,
      };
      ```
  - [ ] ##### SubTask 1.2.3: Add `run_produces_and_persists_narration` (2 SP)
    - Rewrite of `test_arrival_narration_survives_reload` on in-memory storage
      (not `SqliteTestAppBuilder`). Drop the reload round-trip (already
      covered by `message_service_tests.rs` at unit + `message_storage.rs`
      at driven-adapter).
    - **Setup (Finding 1):** use `TestDataBuilder::default_test()` (world
      *with* scenario `test_intro`, `starting_room_id: "room_1"`) so
      `inject_scenario_logs` actually injects. Do NOT use
      `seed_test_world_into_storage` (its `TestWorld::minimal()` has
      `scenarios: vec![]` → inject is a no-op).
    - Body:
      ```
      let data = TestDataBuilder::default_test().build();
      let storage = Arc::new(Storage::new_in_memory());
      data.seed_into(&storage);
      storage
          .save_snapshot(&GameStateSnapshot::from_game_state(&GameState::new("room_1")))
          .expect("test setup: save initial snapshot");

      let arrival_preset = default_test_preset_storage()
          .get_preset("system_default")
          .ok()
          .flatten()
          .expect("system_default preset should exist");

      let llm: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
      let recorder = make_test_recorder_with_storage(Arc::clone(&llm), Arc::clone(&storage));
      let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));

      let task_ctx = ArrivalTaskContext::new_for_test(
          Arc::clone(&message_service),
          Arc::clone(&storage),
          "room_1".to_string(),
          Vec::<NpcCard>::new(),
          Vec::<NpcCard>::new(),
          Some(arrival_preset),
          "short".to_string(),
          1024,
          None,
          recorder,
      );
      task_ctx.run_sync();
      ```
    - Assertions:
      ```
      let state = message_service.load_or_fresh();
      let narrations: Vec<_> = state
          .narrative
          .history()
          .into_iter()
          .filter(|e| e.message_type == MessageType::Narration)
          .collect();
      assert!(!narrations.is_empty(), "arrival run should persist at least one Narration");
      assert_eq!(
          state.narrative.input_buffer.status,
          GenerationStatus::Idle,
          "status should return to Idle after successful arrival narration"
      );

      let messages = storage.load_messages_with_swipes().unwrap();
      assert!(
          messages.iter().any(|m| m.message_type == MessageType::Narration),
          "narration should be persisted to messages table"
      );

      let llm_messages = storage.list_latest_llm_messages(50).unwrap();
      assert!(
          llm_messages.iter().any(|m| m.agent_name == "narrator"),
          "narrator row should be persisted to llm_messages table"
      );
      ```
    - Note: `narrations.len()` is 2 (1 scenario inject + 1 arrival). Asserting
      `!is_empty()` matches the ticket's "at least 1 Narration" literally;
      the scenario-inject branch is *executed* (coverage-tool level, matching
      the original), not separately asserted. Strengthening to `>= 2` is
      available if the user wants assertion-level coverage of inject.
  - [ ] ##### SubTask 1.2.4: Add `run_falls_back_to_fresh_state_on_load_failure` (1 SP)
    - Port from `arrival_service_tests_falls_back_to_fresh_state_on_load_error`.
      Already in-memory + failure-injected. Apply Finding 2: drop
      `make_test_pipeline_with_mock_quantifier` + `build_test_wired_app` +
      `AppState::from_wired` + the manual `preset_storage.save_preset(...)`
      block. Use `MessageService::new(Arc::clone(&failing_storage))` directly
      and `default_test_preset_storage()` for the preset.
    - Replace `let state = create_minimal_test_state();` with
      `let state = GameState::new("room1");` (same value; helper lives in
      `tests/helpers/fixtures.rs`, unreachable from `src/`). Replace all
      `chronicler_engine::*` paths with `crate::*`.
    - Keep `seed_test_world_into_storage(&failing_storage, &state)` (no-scenario
      world is fine here — the failure path is the point, not scenario inject).
    - Keep `handle.set("load_latest_snapshot", TestOverride::internal("simulated load_latest_snapshot failure"))`
      and `handle.clear("load_latest_snapshot")` after `run_sync`.
    - Rename: `arrival_service_tests_falls_back_to_fresh_state_on_load_error`
      → `run_falls_back_to_fresh_state_on_load_failure`.
    - Assertion unchanged: after `run_sync`, `message_service.load_or_fresh().narrative.history()`
      has ≥1 Narration (the arrival narration; `build_fresh_initial_state`
      produces 0 since the no-scenario world has no scenario text, then
      arrival adds 1).
  - [ ] ##### SubTask 1.2.5: Add `run_returns_early_without_narration_on_world_fetch_failure` (1 SP)
    - Port from `arrival_service_returns_early_without_narration_on_world_fetch_failure`.
      Same substitutions as SubTask 1.2.4 (`chronicler_engine::` → `crate::`,
      `create_minimal_test_state()` → `GameState::new("room1")`, Finding 2
      drops pipeline/wired-app/manual-preset).
    - Keep `seed_test_world_into_storage(&failing_storage, &state)` + the
      `failing_storage.save_snapshot(&GameStateSnapshot::from_game_state(&state))`
      before the `handle.set("get_world", ...)` (so `load_expecting_valid_state`
      succeeds, then `require_world` fails → `run` returns Err early).
    - Keep `handle.set("get_world", TestOverride::internal("simulated get_world failure"))`
      and `handle.clear("get_world")` after `run_sync`.
    - Rename: `arrival_service_returns_early_without_narration_on_world_fetch_failure`
      → `run_returns_early_without_narration_on_world_fetch_failure`.
    - Assertion unchanged: after `run_sync`, `message_service.load_or_fresh().narrative.history()`
      has 0 Narrations (snapshot was saved with empty history; `run` returned
      Err before adding any message).

### Phase 2: Delete component-tier files + unwire + verify

- [ ] #### Task 2.1: Delete files + unwire + rmdir (3 SP)
  - [ ] ##### SubTask 2.1.1: Unwire from `tests/integration/mod.rs` (1 SP)
    - Remove 4 lines:
      `#[path = "application/lifecycle.rs"]` + `mod lifecycle;`
      `#[path = "flow/arrival_persistence.rs"]` + `mod flow_arrival_persistence;`
    - Leave `mod llm_client;`, `mod bootstrap;`, `mod storage;`, `mod model;`
      and all `use`/`pub`/helper-fn lines untouched.
  - [ ] ##### SubTask 2.1.2: Delete the 2 component-tier files + empty dirs (2 SP)
    - `rm chronicler_engine/tests/integration/application/lifecycle.rs`
    - `rm chronicler_engine/tests/integration/flow/arrival_persistence.rs`
    - `rmdir chronicler_engine/tests/integration/application`
    - `rmdir chronicler_engine/tests/integration/flow`
      (both dirs empty after deletion; matches ticket 02/04's convention of
      removing the emptied `action_pipeline/` dir.)

- [ ] #### Task 2.2: Verify build, tests, guardrails, spec validator (3 SP)
  - [ ] ##### SubTask 2.2.1: Build + full nextest (2 SP)
    - `cargo nextest run -p chronicler_engine` — expect **1348 pass, 2 skipped**
      (baseline 1356/2 from ticket 08, minus 13 deleted, plus 5 added = net -8).
    - `cargo build -p chronicler_engine` — 0 warnings.
    - `cargo clippy --tests -p chronicler_engine` — 0 errors.
  - [ ] ##### SubTask 2.2.2: Guardrails + spec validator (1 SP)
    - `cargo nextest run -p chronicler_engine guardrails` — expect 101/101 pass
      (no new violations: `arrival_service_tests.rs` pairs with
      `arrival_service.rs`; `_tests.rs` exempt from WiredApp-scope + doc-anchor
      rules; `catalogue_tests.rs` already wired).
    - `python scripts/validate_feature_spec.py` — expect unchanged spec
      coverage (no new SCENARIO tags in `src/`; no spec changes in this
      ticket).

## Test Plan

- 5 new unit tests pass:
  - `catalogue_tests::create_game_persists_scenario_message_and_swipe`
  - `catalogue_tests::delete_game_succeeds_silently_for_nonexistent_game`
  - `arrival_service_tests::run_produces_and_persists_narration`
  - `arrival_service_tests::run_falls_back_to_fresh_state_on_load_failure`
  - `arrival_service_tests::run_returns_early_without_narration_on_world_fetch_failure`
- 13 deleted tests gone (10 from `lifecycle.rs`, 3 from `arrival_persistence.rs`).
- `tests/integration/mod.rs` no longer references `lifecycle` or `flow_arrival_persistence`.
- `tests/integration/application/` and `tests/integration/flow/` directories removed.
- No new `SCENARIO:` tags in `src/` (per STRATEGY.md — tags live in `tests/http/` only).
- Arrival tests use `MessageService::new` directly (no `AppState`/`WiredApp`/pipeline);
  test #1 uses `TestDataBuilder::default_test()` (scenario world) so
  `inject_scenario_logs` is exercised, not a no-op.

## Per Task/Sub Task Validation Steps

- **1.1.1**: `cargo check -p chronicler_engine --tests` compiles; import-ordering guardrail green.
- **1.1.2**: `cargo nextest run -p chronicler_engine create_game_persists_scenario_message_and_swipe` passes.
- **1.1.3**: `cargo nextest run -p chronicler_engine delete_game_succeeds_silently_for_nonexistent_game` passes.
- **1.2.1**: `cargo check -p chronicler_engine --tests` compiles with new `mod arrival_service_tests;`.
- **1.2.2**: `cargo check -p chronicler_engine --tests` compiles the new file's imports.
- **1.2.3**: `cargo nextest run -p chronicler_engine run_produces_and_persists_narration` passes.
- **1.2.4**: `cargo nextest run -p chronicler_engine run_falls_back_to_fresh_state_on_load_failure` passes.
- **1.2.5**: `cargo nextest run -p chronicler_engine run_returns_early_without_narration_on_world_fetch_failure` passes.
- **2.1.1**: `cargo check -p chronicler_engine --tests` compiles after unwiring (no lingering refs).
- **2.1.2**: `ls chronicler_engine/tests/integration/application chronicler_engine/tests/integration/flow` → "No such file or directory".
- **2.2.1**: `cargo nextest run -p chronicler_engine` → 1348 pass, 2 skipped; 0 build warnings; clippy 0 errors.
- **2.2.2**: `cargo nextest run -p chronicler_engine guardrails` → 101/101; `validate_feature_spec.py` → 52/52 (unchanged from ticket 08).

## Assumptions

- **Baseline test count: 1356 pass, 2 skipped** (from ticket 08's resolution;
  no later resolved ticket in this map touches the count). Post-ticket target:
  1348 pass, 2 skipped (net -8 = 13 deleted − 5 added).
- **Preset construction:** all 3 tests use `default_test_preset_storage()`
  (matches original test #1's preset source). The manual
  `Storage::save_preset(...)` block in original tests #2/#3 is dropped (it
  built a preset identical in behavior — `MockBackend` output doesn't depend
  on `name`/`role`).
- **`create_minimal_test_state()` substitution:** helper lives in
  `tests/helpers/fixtures.rs` (integration-binary-only), unreachable from
  `src/`. Replace with `GameState::new("room1")` (the exact value the helper
  returns) — substance-neutral.
- **`storage.load_messages()` in ticket wording:** `Storage` has no
  `load_messages()` method; the valid seam is
  `storage.load_messages_with_swipes()` (what the original test #1 uses).
  (`MessageService::load_messages()` delegates to it; same data.)
- **Empty-directory removal:** ticket 02/04 removed the emptied
  `action_pipeline/` dir; following the same convention here.
- **Tests #2/#3 keep `seed_test_world_into_storage`** (no-scenario world):
  justified — the failure path is the point (`load_latest_snapshot` fails
  → fallback; `get_world` fails → early return), not scenario inject. The
  no-scenario world doesn't affect the assertion (arrival adds the only
  narration in #2; #3 asserts 0 narrations).
- **Scenario-inject branch in test #1 is *executed* (coverage-tool level),
  not separately asserted.** Matches the original test's `!is_empty()`
  assertion shape. Strengthening to `>= 2` is available if the user wants
  assertion-level coverage of `inject_scenario_logs`.
- **Out of scope (per ticket):** the 5 uncovered `ArrivalTaskContext::run`
  branches (both-fail, room-not-in-map, preset-None, recorder-Err,
  save-failure) — fog on the map's "Branch coverage pattern" entry, not
  this ticket. HTTP E2E for games CRUD is ticket 13.
  `games_fragment_handlers.rs` cleanup is ticket 07/08's job.
- **No new SCENARIO tags** in `src/` (STRATEGY.md: tags live in `tests/http/`
  only). No spec changes (no `docs/specs/games.md` here — that's ticket 13).
- **Guardrails:** `arrival_service_tests.rs` pairs with `arrival_service.rs`
  (location-pairing guardrail satisfied); `_tests.rs` suffix exempts it from
  WiredApp-scope and module-doc-anchor guardrails; first line must be plain
  `//! <summary>` (no `[DOC:` anchor) — matches existing `catalogue_tests.rs`
  / `message_service_tests.rs` convention.
- **Considered and skipped (Ponytail):** extracting a shared helper for
  tests #2/#3 (they share ~10 lines of failing-storage + seed + preset +
  recorder + message_service setup). The helper would need to parameterize
  the failure key + whether to save a snapshot, returning the handle too —
  adds indirection for ~10 lines saved. Inline is more readable (you see
  exactly what's faked per test). Left inline.

## NOT in scope

- The 5 uncovered `ArrivalTaskContext::run` branches (both-fail path,
  room_id-not-in-map early return, `arrival_preset`-None Config-error,
  recorder-Err status, `save_message_and_snapshot` failure path). Fog on
  the map's "Branch coverage pattern" entry; not this ticket.
- HTTP E2E for the games CRUD endpoints + new `docs/specs/games.md` —
  ticket 13.
- `tests/http/games_fragment_handlers.rs` cleanup — leave the
  `GET /fragment/games` tests in place; re-tiering those to browser is
  ticket 07/08's job.
- Any production code change. This ticket is test-only.
- Any spec change. `docs/specs/*.md` untouched.

## What already exists

- `seeded_catalogue()` helper in `catalogue_tests.rs` — reused for the 2
  new catalogue tests.
- `TestDataBuilder::default_test()` in `src/test_support/test_data_builder.rs`
  — world with scenario `test_intro` (`starting_room_id: "room_1"`) +
  `TestMap` room `room_1` + persona `test_player` + npc `npc_1`. Used for
  arrival test #1.
- `seed_test_world_into_storage()` in `src/test_support/context.rs` —
  no-scenario world (`TestWorld::minimal()`) + single-room map. Used for
  arrival tests #2/#3.
- `default_test_preset_storage()` in `src/test_support/context.rs` —
  `Arc<Storage>` with `system_default` preset saved. Used for all 3 arrival
  tests' `arrival_preset`.
- `make_test_recorder_with_storage()` in `src/test_support/fixtures.rs` —
  `LlmCallRecorder` wired to `storage.save_llm_message`. Used for all 3
  arrival tests.
- `ArrivalTaskContext::new_for_test()` — the test constructor; signature
  verified (message_service, storage, room_id, nearby_npcs, all_npcs,
  arrival_preset, response_length, max_context_tokens, max_tokens, recorder).
- `MessageService::new(Arc<Storage>)` — the direct constructor used by
  `src/application/message_service_tests.rs` (the structural neighbor);
  used here instead of the heavier `AppState::from_wired(WiredApp)` path.
- `GameState::new(room_id)`, `GameStateSnapshot::from_game_state(&state)`,
  `Storage::new_in_memory()`, `Storage::with_test_failures()` →
  `(Storage, handle)`, `TestOverride::internal(msg)` — all existing test
  seams.
- Predecessor pattern: tickets 02 (action pipeline unit) and 04 (retry
  unit) did the same dissolution work; conventions followed.

## Failure modes

- **`ArrivalTaskContext::run` returns `Err` internally** (e.g.,
  `require_world` fails in test #3): `run_sync` swallows the `Result`
  (`let _ = self.run()`). The test then runs its assertions, which fail
  loudly because the expected narrations are absent. Failure surfaces as
  an `assert!`/`assert_eq!` failure, not a silent pass. Confirmed: test #3
  asserts `narrations.is_empty()` — if `run` somehow added a narration
  before failing, this would fail and signal the bug.
- **`save_message_and_snapshot` fails inside `run`**: logged via
  `tracing::error!` and the `if let Err(e) = ...` block swallows it
  (returns `Ok(())` from `run`). Test #1's `load_messages_with_swipes()`
  assertion would then fail (no narration persisted), surfacing the bug.
  This is one of the 5 uncovered branches (out of scope to test here), but
  the test doesn't mask it — a regression in the save path would make test
  #1 fail.
- **`MockBackend::default()` returns empty text**: if `MockBackend`'s
  default narration were empty, `recorder.complete` would return
  `Err(EngineError)` (empty-narration branch), `run` would set
  `status = Error`, and test #1's `assert_eq!(status, Idle)` would fail.
  Current `MockBackend` returns non-empty text (verified by the existing
  pipeline/retry tests passing), so test #1 is green.
- **`load_or_fresh` falls back to fresh when snapshot missing**: in test
  #1, if `save_snapshot` silently failed, `load_expecting_valid_state`
  would error → `load_or_fresh` falls back to `build_fresh_initial_state`
  → still produces narrations (via the scenario world + arrival). The
  `status == Idle` assertion still holds. But the test would be exercising
  the fallback path, not the happy path — a coverage lie. Mitigation:
  `save_snapshot(...)` is `.expect("test setup: save initial snapshot")`,
  so a save failure panics in setup, not silently misleads.
- **Test #2's `handle.clear("load_latest_snapshot")` after `run_sync`:**
  if this were forgotten, the post-`run_sync` `load_or_fresh()` would
  still hit the injected failure → fall back to `build_fresh_initial_state`
  → 0 narrations (no-scenario world) → `!is_empty()` would fail. So
  forgetting the clear surfaces as a test failure, not a silent pass.
- **Deletion regression:** if a deleted test's coverage wasn't actually
  ported, the next `cargo nextest run` shows the missing-branch regression
  in a downstream change. The 5 ported tests are the net-new branch
  coverage; the 8 deleted tests are confirmed redundant in ticket 06's
  asset (each mapped to existing unit-tier coverage).

## Unresolved decisions

- **Assertion strength for test #1's narration count.** Plan uses
  `!is_empty()` (matches ticket's "at least 1 Narration" literally;
  scenario-inject branch is *executed* but not *asserted*). Strengthening
  to `>= 2` would catch `inject_scenario_logs` regressions at the assertion
  level. Low-stakes; can strengthen during implementation if desired.
- **Whether to also delete `tests/helpers/sqlite_test_app_builder.rs`'s
  `build_with_state_and_storage`/`PipelineFn` paths.** Out of scope —
  `SqliteTestAppBuilder` is still used by other integration tests (e.g.,
  `bootstrap/run_branches.rs`); only `arrival_persistence.rs` used it for
  arrival. Not touched by this ticket.
