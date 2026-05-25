# Plan: Complete GameState Decoupling from Server + Tests

## Goal
Remove `GameState` from all test helper signatures and from `tests/components/` entirely. The server layer (`src/server/`) is already clean. Finish the job by eliminating `GameState` coupling in integration test infrastructure.

## Current State
- ✅ `src/server/` has zero `GameState` references
- ✅ `src/server/fragments/renderers.rs` uses narrow `ApplicationService` methods
- ✅ `src/test_support/server_helpers.rs` holds the three test helpers
- ❌ `create_app_for_testing*` still accepts `GameState` (moved from `server/` but signatures unchanged)
- ❌ `tests/components/*.rs` has ~114 call sites passing `GameState` to helpers
- ❌ `tests/components.rs` defines `create_test_state() -> GameState` used by component tests

## Architecture Decision
**Use `TestAppBuilder` in `test_support/`** — a single builder that internally constructs `GameState` and `AppState`, then returns a `Router`. Tests never touch `GameState` directly.

Why builder over multiple helper variants:
- Covers all current patterns (defaults, settings, pre-seeded logs, custom storage)
- Extensible: new builder methods for future test needs without new helper functions
- One call site migration pattern to learn

---

## Phase 1: Build TestAppBuilder

### Task 1: Create `TestAppBuilder` in `test_support`

**Description:** Add `TestAppBuilder` to `src/test_support/` that builds `AppState` internally without exposing `GameState` in its public API.

**Builder API:**
```rust
pub struct TestAppBuilder { ... }

impl TestAppBuilder {
    pub fn default_test() -> Self;  // uses same defaults as tests/components.rs::create_test_state
    pub fn new(world: WorldCard, player: PlayerCard) -> Self;
    pub fn map(self, map: MapDef) -> Self;
    pub fn npc(self, npc: NpcCard) -> Self;
    pub fn room_npc(self, npc_id: &str) -> Self;
    pub fn log(self, text: &str, speaker: Option<&str>, log_type: LogType) -> Self;
    pub fn settings(self, settings: AppSettings) -> Self;
    pub fn snapshot_storage(self, storage: Arc<dyn SnapshotStorage>) -> Self;
    pub fn message_storage(self, storage: Arc<dyn MessageStorage>) -> Self;
    pub fn llm_storage(self, storage: Arc<dyn LlmMessageStorage>) -> Self;
    pub fn build(self) -> Router;
}
```

**Files touched:**
- `src/test_support/test_app_builder.rs` (new)
- `src/test_support/mod.rs` (export)
- `src/test_support/server_helpers.rs` (delete or deprecate old helpers)
- `src/lib.rs` (re-export `TestAppBuilder`)

**Acceptance criteria:**
- [ ] `TestAppBuilder::default_test().build()` compiles and returns `Router`
- [ ] `TestAppBuilder` internally calls `GameState::new` but `GameState` is not in public API
- [ ] `cargo check --tests` passes after old helpers are removed

**Verification:**
- `cargo check --tests` passes

---

## Phase 2: Migrate Component Tests

### Task 2: Migrate `tests/components/debug.rs`, `tests/components/css.rs`, `tests/components/connections.rs`
**Description:** These three files only use simple `create_app_for_testing(create_test_state())` — lowest complexity.

**Changes:**
- Replace `use chronicler_engine::create_app_for_testing;` with `use chronicler_engine::test_support::TestAppBuilder;`
- Replace `create_app_for_testing(create_test_state())` with `TestAppBuilder::default_test().build()`
- Remove `GameState` imports

**Verification:** `cargo nextest run --test components debug:: css:: connections::` passes

### Task 3: Migrate `tests/components/settings.rs`, `tests/components/prompt_presets.rs`
**Description:** These use `create_app_for_testing(create_test_state())` exclusively — no custom storage or log seeding.

**Changes:** Same as Task 2.

**Verification:** `cargo nextest run --test components settings:: prompt_presets::` passes

### Task 4: Migrate `tests/components/fragment.rs`
**Description:** Uses `create_app_for_testing` plus some `mut state` + `add_log` patterns.

**Changes:**
- Simple calls → `TestAppBuilder::default_test().build()`
- Pre-seeded logs → `.log("...", Some("..."), LogType::Input).build()`

**Verification:** `cargo nextest run --test components fragment::` passes

### Task 5: Migrate `tests/components/text_check.rs`
**Description:** Uses `create_app_for_testing_with_settings` and `create_app_with_storage`.

**Changes:**
- `create_app_for_testing_with_settings(state, settings)` → `TestAppBuilder::default_test().settings(settings).build()`
- `create_app_with_storage(state, ...)` → `TestAppBuilder::default_test().snapshot_storage(...).message_storage(...).llm_storage(...).build()`

**Verification:** `cargo nextest run --test components text_check::` passes

### Task 6: Migrate `tests/components/world.rs`
**Description:** One test builds `GameState::new()` from loaded world data; others use `create_test_state()`.

**Changes:**
- `create_test_state()` calls → `TestAppBuilder::default_test()`
- `GameState::new(world, map, player, npcs, ...)` → `TestAppBuilder::new(world, player).map(map).npcs(npcs).build()`

**Verification:** `cargo nextest run --test components world::` passes

### Task 7: Migrate `tests/components/actions.rs`
**Description:** Mix of `create_app_for_testing`, `create_app_for_testing_with_settings`, `create_app_with_storage`, and `mut state` + `add_log`.

**Changes:** Builder pattern for all variants.

**Verification:** `cargo nextest run --test components actions::` passes

### Task 8: Migrate `tests/components/misc.rs`
**Description:** Largest file (~1300 lines). Uses ALL patterns: defaults, settings, log seeding, custom storage, `GameState::new` from scratch.

**Changes:** Builder pattern for all variants. This is the highest-risk file.

**Verification:** `cargo nextest run --test components misc::` passes

### Task 9: Clean up `tests/components.rs`
**Description:** Remove or make private the `create_test_state()` function in `tests/components.rs` once all component tests no longer need it. Keep `LogType` import if still needed.

**Verification:** `cargo check --tests` passes; `grep "GameState" tests/components.rs` returns 0 results

---

## Phase 3: Architecture Guardrails + Docs

### Task 10: Add Layer Boundary Guardrails
**Description:** Add `tests/guardrails/layers.rs` with checks that:
1. `src/server/` (except `mod.rs`) contains no `GameState` imports/references
2. `tests/components/` contains no `GameState` imports/construction
3. `src/server/` contains no `.load_state()` calls

**Files touched:**
- `tests/guardrails/layers.rs` (new)
- `tests/guardrails.rs` (register new module/tests)

**Acceptance criteria:**
- [ ] `check_server_layer_boundaries` detects `GameState` in `src/server/`
- [ ] `check_test_layer_boundaries` detects `GameState` in `tests/components/`
- [ ] `check_load_state_calls` detects `.load_state()` in `src/server/`
- [ ] File-level exceptions for `src/server/mod.rs` and `src/server/debug.rs`

**Verification:**
- `cargo nextest run --test guardrails` passes
- Temporarily adding `GameState` to `src/server/` causes test failure

### Task 11: Update Agent Rules and Architecture Docs
**Description:** Document the layer boundary in `.agents/rules/chronicler_engine.md` and `chronicler_engine/docs/architecture/system.md`.

**Files touched:**
- `.agents/rules/chronicler_engine.md`
- `chronicler_engine/docs/architecture/system.md`

**Acceptance criteria:**
- [ ] `.agents/rules/chronicler_engine.md` has a "Layer Boundary" section
- [ ] `docs/architecture/system.md` documents server/domain separation
- [ ] Both reference `ApplicationService` as the required boundary

**Verification:**
- `grep -i "layer boundary" .agents/rules/chronicler_engine.md` finds section
- `grep -i "ApplicationService" chronicler_engine/docs/architecture/system.md` finds boundary description

---

## Final Checkpoint
- [ ] `cargo nextest run --test components` passes
- [ ] `cargo nextest run --test guardrails` passes
- [ ] `cargo nextest run --test architecture` passes
- [ ] `python build.py` passes end-to-end
- [ ] `grep -r "GameState" tests/components/` returns 0 results
- [ ] `grep -r "GameState" src/server/` returns 0 results

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `TestAppBuilder` doesn't cover all state mutation patterns | Medium | Add builder methods as needed during migration; no external API change |
| Task 8 (`misc.rs`) is large and error-prone | High | Run tests after every 5-10 call site changes |
| Tests rely on specific `create_test_state()` defaults | Low | `TestAppBuilder::default_test()` mirrors existing defaults exactly |
| `LogType` still imported from `state.rs` in tests | Low | Acceptable — `LogType` is a simple enum, not structural `GameState` coupling |

## Excluded
- Engine/pipeline `GameState` passing — domain-layer coupling, not a leak
- `tests/test_data.rs` and `tests/helpers/` — outside `tests/components/` scope
- `ApplicationService` internal field mutations — acceptable within application layer
