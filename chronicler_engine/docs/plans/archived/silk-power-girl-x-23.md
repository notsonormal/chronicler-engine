# Implementation Plan: Eliminate Test Duplication (Moderate Approach)

## Overview

Extract duplicated fixture data, mock implementations, and builder logic from the three worst unit-test files (`retry_tests.rs`, `bootstrap_tests.rs`, `handlers_tests.rs`) and route them through existing `src/test_support/` infrastructure. Extend `TestAppBuilder` and `fixtures.rs` where gaps exist. No behavioral changes — purely structural refactoring.

## Architecture Decisions

1. **Use existing `test_support/` patterns** — `TestPlayer`, `TestNpc`, `TestMap`, `TestGameState`, `TestAppBuilder`, `make_test_context` already exist. We extend them, not replace them.
2. **Fixture structs are stateless `impl` blocks** — follows existing convention in `fixtures.rs` (`TestWorld::minimal()`, `TestPlayer::standard()`).
3. **Failing storage mocks live together** — new `failing_storage.rs` module under `test_support/` for discoverability.
4. **Builder extension is additive only** — `TestAppBuilder::prompt_preset_storage()` is a new fluent method; all existing calls compile unchanged.

---

## Dependency Graph

```
Phase 1: Foundation (independent tasks)
├── Task 1: Create failing_storage.rs
├── Task 2: Add TestStoredTriggerContext + TestPromptPreset fixtures
├── Task 3: Add bootstrap fixtures (TestWorldManifest, TestCharacterSheet)
└── Task 4: Extend TestAppBuilder with prompt_preset_storage()

Phase 2: Adopt in Target Files
├── Task 5: retry_tests.rs — use failing mocks + TestStoredTriggerContext
├── Task 6: retry_tests.rs — replace make_empty_context (TBD after equivalence check)
├── Task 7: handlers_tests.rs — use TestPromptPreset + failing mock from test_support
├── Task 8: handlers_tests.rs — replace manual AppState with TestAppBuilder
├── Task 9: fragments_tests.rs — replace manual AppState with TestAppBuilder
└── Task 10: bootstrap_tests.rs — use bootstrap fixtures

Phase 3: Cross-File StoredTriggerContext Cleanup
├── Task 11: pipeline_tests.rs — use TestStoredTriggerContext
├── Task 12: actions_tests.rs — use TestStoredTriggerContext
└── Task 13: action_processing_tests.rs — use TestStoredTriggerContext

Phase 4: Verification
└── Task 14: Full build + test validation
```

---

## Task List

### Phase 1: Foundation

#### Task 1: Create `src/test_support/failing_storage.rs`

**Description:** Extract the three inline failing-storage mocks into a shared module.

**Acceptance criteria:**
- [ ] `FailingSnapshotStorage` moved from `retry_tests.rs` (lines 26–85)
- [ ] `FailingMessageStorage` moved from `retry_tests.rs` (lines 87–169)
- [ ] `FailingPromptPresetStorage` moved from `handlers_tests.rs` (lines 14–69)
- [ ] All three types are `pub` and re-exported from `test_support/mod.rs`
- [ ] Compilation passes after creating the file

**Verification:**
- [ ] `cargo check` succeeds in `chronicler_engine/`

**Dependencies:** None

**Files touched:**
- `src/test_support/failing_storage.rs` (new)
- `src/test_support/mod.rs`

**Estimated scope:** Small

---

#### Task 2: Extend `src/test_support/fixtures.rs` — Trigger + Preset Helpers

**Description:** Add fixture helpers for the two most-duplicated structs.

**Acceptance criteria:**
- [ ] `TestStoredTriggerContext` struct added with methods:
  - `standard()` → `{ npc_id: "npc1", trigger_idx: 0, trigger_name: "Test", trigger_repeat: false, trigger_narration_prompt: "Test prompt", system_prompt: "sys", user_prompt: "user", max_tokens: None }`
  - `for_npc(npc_id: &str, trigger_name: &str, narration_prompt: &str)` → parameterized variant for `action_processing_tests.rs` ("carla", "Carla Introduction", "Carla appears")
  - `with_max_tokens(npc_id: &str, trigger_name: &str, max_tokens: u32)` → for the one test that uses `max_tokens: Some(512)`
- [ ] `TestPromptPreset` struct added with methods:
  - `system(id: &str, name: &str)` → `{ id, name, preset_type: PresetType::System, instructions: Some(name ~ "."), ..Default::default() }`
  - `system_default(id: &str, name: &str)` → same + `is_default: true`
  - `custom(id: &str, name: &str)` → `{ preset_type: PresetType::Custom, ... }`
  - `with_instructions(id: &str, name: &str, instructions: &str, preset_type: PresetType)` → fully parameterized for edge cases

**Verification:**
- [ ] `cargo check` succeeds
- [ ] New fixtures are reachable from unit tests via `crate::test_support::*`

**Dependencies:** None

**Files touched:**
- `src/test_support/fixtures.rs`

**Estimated scope:** Small

---

#### Task 3: Extend `src/test_support/fixtures.rs` — Bootstrap Helpers

**Description:** Add fixtures for bootstrap-test structs that don't yet have helpers.

**Acceptance criteria:**
- [ ] `TestWorldManifest::minimal()` → `WorldManifest { id: "test", name: "Test", starting_room_id: "room_a", description: "A test world", global_rules: vec![], map_file: "map.json", player_file: "player.json", characters_dir: "", scenarios: vec![], default_scenario_id: None, default_room_image: None }`
- [ ] `TestCharacterSheet::hero()` → `CharacterSheet { name: "Hero", description: "A hero", personality: "Brave", scenario: "Default", example_dialogue: "", summary: None, profile_image: None, headshot_image: None }`
- [ ] `TestCharacterSheet::named(name: &str)` → parameterized variant

**Verification:**
- [ ] `cargo check` succeeds

**Dependencies:** None

**Files touched:**
- `src/test_support/fixtures.rs`

**Estimated scope:** Small

---

#### Task 4: Extend `TestAppBuilder` with `prompt_preset_storage()`

**Description:** Add a fluent builder method so unit tests can inject custom preset storage (including failing mocks) instead of building `AppState` manually.

**Acceptance criteria:**
- [ ] New field `prompt_preset_storage: Option<Arc<dyn PromptPresetStorage>>` added to `TestAppBuilder`
- [ ] New method `prompt_preset_storage(mut self, storage: Arc<dyn PromptPresetStorage>) -> Self` added
- [ ] `build()` uses the injected storage if present, else falls back to existing `InMemoryPromptPresetStorage::new()`
- [ ] All existing `TestAppBuilder` calls compile unchanged (additive change)

**Verification:**
- [ ] `cargo check` succeeds
- [ ] Existing integration tests in `tests/components/` still compile

**Dependencies:** None

**Files touched:**
- `src/test_support/test_app_builder.rs`

**Estimated scope:** Small

---

### Checkpoint: Foundation Complete
- [ ] All four foundation tasks compile
- [ ] No regressions in existing tests

---

### Phase 2: Adopt in Target Files

#### Task 5: `retry_tests.rs` — Adopt Failing Mocks + `TestStoredTriggerContext`

**Description:** Remove inline mock definitions and replace inline `StoredTriggerContext` structs with fixtures.

**Acceptance criteria:**
- [ ] `FailingSnapshotStorage` and `FailingMessageStorage` definitions deleted; import from `test_support`
- [ ] All 7 inline `StoredTriggerContext { ... }` blocks replaced with `TestStoredTriggerContext::standard()`
- [ ] `EmptyTriggerBackend` stays inline (only used once, not a shared mock)

**Verification:**
- [ ] `cargo test --lib retry` passes (or targeted test run)

**Dependencies:** Task 1, Task 2

**Files touched:**
- `src/application/action_pipeline/retry_tests.rs`

**Estimated scope:** Medium

---

#### Task 6: `retry_tests.rs` — Replace `make_empty_context()`

**Description:** Evaluate whether `make_empty_context()` can be replaced with `make_test_context_without_snapshot()`.

**Important note:** `make_empty_context()` uses a bare `InMemoryPromptPresetStorage` (no presets saved), while `make_test_context_without_snapshot()` calls `default_test_preset_storage()` which saves a "system_default" preset. This may or may not affect retry test behavior.

**Acceptance criteria:**
- [ ] Run retry tests with `make_test_context_without_snapshot()` as a drop-in replacement
- [ ] If tests pass → replace and delete `make_empty_context()`
- [ ] If tests fail → document the discrepancy, keep `make_empty_context()`, and add a comment explaining why it exists

**Verification:**
- [ ] `cargo test --lib retry` passes

**Dependencies:** Task 5

**Files touched:**
- `src/application/action_pipeline/retry_tests.rs`

**Estimated scope:** Small

---

#### Task 7: `handlers_tests.rs` — Adopt `TestPromptPreset` + Failing Mock

**Description:** Remove inline `FailingPromptPresetStorage` and replace inline `PromptPreset` structs with fixtures.

**Acceptance criteria:**
- [ ] `FailingPromptPresetStorage` definition deleted; import from `test_support`
- [ ] All ~15 inline `PromptPreset { ... }` blocks replaced with `TestPromptPreset::*` calls
- [ ] `PresetType::System` presets use `TestPromptPreset::system()` or `TestPromptPreset::system_default()`
- [ ] `PresetType::Custom` presets use `TestPromptPreset::custom()`

**Verification:**
- [ ] `cargo test --lib prompt_presets_fragment` passes

**Dependencies:** Task 1, Task 2

**Files touched:**
- `src/server/prompt_presets_fragment/handlers_tests.rs`

**Estimated scope:** Medium

---

#### Task 8: `handlers_tests.rs` — Replace Manual `AppState` with `TestAppBuilder`

**Description:** Replace the three `make_test_app_state_*` functions with `TestAppBuilder`.

**Acceptance criteria:**
- [ ] `make_test_app_state_with_preset(preset)` → `TestAppBuilder::default_test().prompt_preset_storage(...).build()` (note: `TestAppBuilder` builds a `Router`, not `AppState` — verify if handlers need `AppState` directly or can accept `Router` + extract state)
- [ ] `make_test_app_state_with_storage(storage, preset)` → builder with injected storage
- [ ] `make_test_app_state_with_failing_storage(...)` → builder with `FailingPromptPresetStorage` injected via `prompt_preset_storage()`
- [ ] If handlers require `AppState` directly (not `Router`), consider adding `TestAppBuilder::build_app_state()` or keep a thin helper that extracts `AppState` from the router

**Risk / Open Question:** `TestAppBuilder::build()` returns `Router`, but the handler tests call functions directly with `axum::extract::State(app_state)`. We need to verify whether we can extract `AppState` from the router or whether we need a `build_app_state()` method.

**Verification:**
- [ ] `cargo test --lib prompt_presets_fragment` passes

**Dependencies:** Task 4, Task 7

**Files touched:**
- `src/server/prompt_presets_fragment/handlers_tests.rs`
- `src/test_support/test_app_builder.rs` (if `build_app_state()` is needed)

**Estimated scope:** Medium

---

#### Task 9: `fragments_tests.rs` — Replace Manual `AppState` with `TestAppBuilder`

**Description:** Replace `make_test_app_state()` with `TestAppBuilder` (same approach as Task 8).

**Acceptance criteria:**
- [ ] `make_test_app_state()` deleted or reduced to a thin wrapper
- [ ] Uses `TestAppBuilder` with `llm_storage()` if needed (the current function takes an optional LLM storage param)

**Verification:**
- [ ] `cargo test --lib fragments_tests` passes

**Dependencies:** Task 4, Task 8 (so approach is consistent)

**Files touched:**
- `src/server/fragments_tests.rs`

**Estimated scope:** Small

---

#### Task 10: `bootstrap_tests.rs` — Use Bootstrap Fixtures

**Description:** Replace repeated `WorldManifest`, `CharacterSheet`, `Room`, `Region`, `MapDef`, `Overworld`, `PlayerCard`, `NpcCard` blocks with fixtures.

**Acceptance criteria:**
- [ ] `WorldManifest { ... }` blocks replaced with `TestWorldManifest::minimal()`
- [ ] `CharacterSheet { name: "Hero" ... }` blocks replaced with `TestCharacterSheet::hero()`
- [ ] `Room { id: "room_a" ... }` blocks evaluated — if `TestMap::room("room_a")` is equivalent, use it; if bootstrap needs different defaults, keep inline or add `TestRoom::for_bootstrap()`
- [ ] `PlayerCard { sheet: CharacterSheet { ... } }` replaced with `TestPlayer::standard()` if equivalent
- [ ] `NpcCard { ... }` replaced with `TestNpc::named()` if equivalent

**Verification:**
- [ ] `cargo test --lib bootstrap` passes

**Dependencies:** Task 3

**Files touched:**
- `src/bootstrap_tests.rs`

**Estimated scope:** Medium

---

### Checkpoint: Target Files Complete
- [ ] All three worst files compile and tests pass
- [ ] Line count reduced measurably in `retry_tests.rs`, `bootstrap_tests.rs`, `handlers_tests.rs`

---

### Phase 3: Cross-File `StoredTriggerContext` Cleanup

#### Task 11: `pipeline_tests.rs` — Use `TestStoredTriggerContext`

**Description:** Replace 2 inline `StoredTriggerContext` blocks with fixtures.

**Acceptance criteria:**
- [ ] Both `StoredTriggerContext { npc_id: "npc1", trigger_name: "Test", trigger_narration_prompt: "Hello" ... }` replaced with `TestStoredTriggerContext::standard()`

**Verification:**
- [ ] `cargo test --lib pipeline_tests` passes

**Dependencies:** Task 2

**Files touched:**
- `src/application/action_pipeline/pipeline_tests.rs`

**Estimated scope:** Small

---

#### Task 12: `actions_tests.rs` — Use `TestStoredTriggerContext`

**Description:** Replace 1 inline `StoredTriggerContext` block with fixture.

**Acceptance criteria:**
- [ ] `StoredTriggerContext { trigger_name: "Old Trigger", npc_id: "npc1", ... }` replaced with appropriate fixture method (may need `TestStoredTriggerContext::named("Old Trigger", "npc1")`)

**Verification:**
- [ ] `cargo test --lib actions_tests` passes

**Dependencies:** Task 2

**Files touched:**
- `src/application/action_pipeline/actions_tests.rs`

**Estimated scope:** Small

---

#### Task 13: `action_processing_tests.rs` — Use `TestStoredTriggerContext`

**Description:** Replace 6 inline `StoredTriggerContext` blocks with fixtures.

**Acceptance criteria:**
- [ ] 5 identical "carla" blocks replaced with `TestStoredTriggerContext::for_npc("carla", "Carla Introduction", "Carla appears")`
- [ ] 1 "repeat" variant replaced with `TestStoredTriggerContext::for_npc("carla", "Carla Greeting", "Carla greets").with_repeat(true)` or equivalent
- [ ] 1 block with `max_tokens: Some(512)` replaced with `TestStoredTriggerContext::with_max_tokens(...)`

**Verification:**
- [ ] `cargo test --lib action_processing` passes

**Dependencies:** Task 2

**Files touched:**
- `src/engine/action_processing_tests.rs`

**Estimated scope:** Small

---

### Checkpoint: Cross-File Cleanup Complete
- [ ] All 21 `StoredTriggerContext` inline structs replaced
- [ ] All related tests pass

---

### Phase 4: Final Validation

#### Task 14: Full Build + Test Validation

**Description:** Run the full validation suite to ensure no regressions.

**Acceptance criteria:**
- [ ] `cd chronicler_engine && python build.py` passes (fmt + clippy + tests + coverage)
- [ ] No new warnings introduced
- [ ] Test count unchanged (no tests accidentally deleted)

**Verification:**
- [ ] Build output shows all tests passing
- [ ] Clippy is clean

**Dependencies:** All prior tasks

**Files touched:** None (validation only)

**Estimated scope:** Small

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Fixture not perfectly equivalent to inline struct | High | Compare field-by-field before replacing; run tests after each file |
| `TestAppBuilder::build()` returns `Router` but tests need `AppState` | Medium | Add `build_app_state()` method if needed; keep thin wrapper helpers |
| `make_test_context_without_snapshot()` has different preset setup | Medium | Task 6 explicitly tests drop-in replacement; keep local helper if mismatch |
| Bootstrap tests use `WorldManifest` (not `WorldCard`) — different struct | Low | Add bootstrap-specific fixtures; don't try to reuse `TestWorld` |
| Large file diff makes review hard | Low | One task per file; checkpoint after each phase |

## Open Questions

1. **AppState extraction from Router:** The handler tests pass `AppState` directly to axum handlers. `TestAppBuilder::build()` returns `Router`. Do we need a `build_app_state()` method, or can tests extract state from the router? *(Needs investigation during Task 8)*
2. **Bootstrap `Room` equivalence:** `TestMap::room("room_a")` sets `name: "Room room_a"` but bootstrap uses `name: "Room A"`. We may need `TestRoom::named("room_a", "Room A")` or keep some inline rooms.
