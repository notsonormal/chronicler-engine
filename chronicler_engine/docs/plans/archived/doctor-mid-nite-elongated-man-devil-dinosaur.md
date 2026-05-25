# Plan: Reduce GameState Blast Radius

## Goal
Contain `GameState` to the domain and application layers. The HTTP server and integration tests should not depend on `GameState` structure.

## Architecture Decisions
- **Server layer reads data via ApplicationService methods only.** No direct `GameState` field access in `src/server/`.
- **Domain layer (engine, pipeline) keeps GameState.** Refactoring phases to avoid `GameState` would add parameter bloat without reducing real blast radius.
- **Guardrails use existing `tests/guardrails.rs` infrastructure.** `Violation`/`assert_violations` pattern with `walkdir` file discovery.
- **Tests construct state through helpers, not `GameState` directly.** `create_app_for_testing` builds its own `GameState` internally.

## Investigation Results (Read-Only)

### Load State Callers
**Server layer leaks (5 calls):**
- `src/server/fragments/renderers.rs:39/89/97/114` — 4 calls in renderers
- `src/server/debug.rs:34` — 1 call in debug handler

**Application layer tests (38 calls)** — expected domain-layer consumers, no action needed.

### GameState Construction in Server
**Only in `src/server/mod.rs`** — `create_app_for_testing`, `create_app_for_testing_with_settings`, `create_app_with_storage` all take `GameState` as parameter.

**Callers in tests:** ~80 calls across 10 test files in `tests/components/`.

### Existing Guardrail Infrastructure
- `tests/guardrails.rs` — `Violation`/`assert_violations`/`walkdir` pattern
- `tests/guardrails/structure.rs` and `tests/guardrails/style.rs` — existing modules
- `arch-lint.toml` — module-level scope bans only, no per-type restrictions
- `build.py` — runs `cargo nextest run --test guardrails` automatically

---

## Task List

### Phase 1: Server Read Decoupling

#### Task 1: Story Log Read Decoupling
**Description:** Add `get_story_log_entries` to `ApplicationService` and migrate `render_story_log` to use it instead of `load_state().narrative.history()`.

**Acceptance criteria:**
- [ ] `ApplicationService` trait has `get_story_log_entries` method
- [ ] `render_story_log` calls `get_story_log_entries` and does not reference `GameState`
- [ ] `StoryLogTemplate` still renders correctly with the returned data

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo nextest run` passes
- [ ] `grep "load_state" src/server/fragments/renderers.rs` returns 3 results (was 4)

**Dependencies:** None

**Files likely touched:**
- `src/application/application_service.rs`
- `src/server/fragments/renderers.rs`

**Estimated scope:** Small (2 files)

#### Task 2: Action Area Read Decoupling
**Description:** Add `get_input_status` to `ApplicationService` and migrate `render_action_area` to use it instead of `load_state().narrative.input_buffer`.

**Acceptance criteria:**
- [ ] `ApplicationService` trait has `get_input_status` method
- [ ] `render_action_area` calls `get_input_status` and does not reference `GameState`
- [ ] `ActionAreaTemplate` receives `(GenerationStatus, GenerationPhase)` directly

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo nextest run` passes
- [ ] `grep "load_state" src/server/fragments/renderers.rs` returns 2 results

**Dependencies:** None

**Files likely touched:**
- `src/application/application_service.rs`
- `src/server/fragments/renderers.rs`

**Estimated scope:** Small (2 files)

#### Task 3: Visual Sidebar Read Decoupling
**Description:** Add `get_current_room_view` and `get_npc_headshots` to `ApplicationService` and migrate `render_visual_sidebar` to use them. Eliminate `render_visual_sidebar_unlocked(&GameState)`.

**Acceptance criteria:**
- [ ] `ApplicationService` trait has `get_current_room_view` and `get_npc_headshots` methods
- [ ] `render_visual_sidebar` calls both new methods and does not reference `GameState`
- [ ] `render_visual_sidebar_unlocked` is removed or made private
- [ ] `VisualSidebarTemplate` renders correctly with view data

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo nextest run` passes
- [ ] `grep "load_state" src/server/fragments/renderers.rs` returns 0 results
- [ ] `grep "GameState" src/server/fragments/renderers.rs` returns 0 results

**Dependencies:** None

**Files likely touched:**
- `src/application/application_service.rs`
- `src/server/fragments/renderers.rs`

**Estimated scope:** Medium (2 files, but involves new view types)

#### Task 4: Debug + Character Headshots Read Decoupling
**Description:** Add `get_debug_state_view` (or narrow equivalent) and migrate `debug.rs` and `render_character_headshots` to use ApplicationService methods. Remove `load_state` from public API.

**Acceptance criteria:**
- [ ] `debug.rs` no longer calls `load_state()` directly
- [ ] `render_character_headshots` uses `get_npc_headshots` from Task 3
- [ ] `ApplicationService::load_state()` is removed from trait or made `pub(crate)`
- [ ] `GameState` import is removed from `src/server/fragments/renderers.rs`

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo nextest run` passes
- [ ] `grep "GameState" src/server/` returns only `mod.rs` and `debug.rs`
- [ ] `grep "load_state" src/server/` returns 0 results (or only in `mod.rs` if test helpers need it)

**Dependencies:** Task 3 (needs `get_npc_headshots`)

**Files likely touched:**
- `src/application/application_service.rs`
- `src/server/fragments/renderers.rs`
- `src/server/debug.rs`

**Estimated scope:** Medium (3 files)

### Checkpoint: After Tasks 1-4
- [ ] `cargo check` clean
- [ ] `cargo nextest run` passes
- [ ] `grep -r "GameState" src/server/fragments/` returns 0 results
- [ ] `grep -r "load_state" src/server/fragments/` returns 0 results

---

### Phase 2: Server Test Helper Decoupling

#### Task 5: Redesign Server Test Helpers
**Description:** Remove `GameState` from `create_app_for_testing*` signatures. The helpers should construct `GameState` internally from world/player parameters, or accept an `AppState` builder pattern.

**Acceptance criteria:**
- [ ] `create_app_for_testing` no longer takes `GameState`
- [ ] `create_app_for_testing_with_settings` no longer takes `GameState`
- [ ] `create_app_with_storage` no longer takes `GameState`
- [ ] New signatures accept `WorldCard`, `PlayerCard`, and optional starting room/NPCs
- [ ] `GameState` import removed from `src/server/mod.rs` public API

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo nextest run` passes
- [ ] Compilation errors in tests are expected (fixed in Task 6)

**Dependencies:** Checkpoint after Tasks 1-4

**Files likely touched:**
- `src/server/mod.rs`
- `src/lib.rs` (re-exports)

**Estimated scope:** Small (2 files)

#### Task 6: Update Integration Test Callers
**Description:** Migrate all `tests/components/` files that call `create_app_for_testing` to use the new helper signatures. Break into sub-sessions if needed.

**Acceptance criteria:**
- [ ] All `tests/components/*.rs` compile without `GameState` construction
- [ ] No `GameState` imports remain in `tests/components/`
- [ ] Test behavior is unchanged

**Verification:**
- [ ] `cargo nextest run --test components` passes
- [ ] `grep -r "GameState" tests/components/` returns 0 results

**Dependencies:** Task 5

**Files likely touched:**
- `tests/components/actions.rs`
- `tests/components/css.rs`
- `tests/components/connections.rs`
- `tests/components/debug.rs`
- `tests/components/fragment.rs`
- `tests/components/misc.rs`
- `tests/components/prompt_presets.rs`
- `tests/components/settings.rs`
- `tests/components/text_check.rs`
- `tests/components/world.rs`

**Estimated scope:** Large (10 files — break into multiple sub-sessions during implementation)

### Checkpoint: After Tasks 5-6
- [ ] `cargo nextest run --test components` passes
- [ ] `grep -r "GameState" tests/` returns 0 results
- [ ] `python build.py` passes

---

### Phase 3: Architecture Guardrails

#### Task 7: Add Layer Boundary Guardrails
**Description:** Add `tests/guardrails/layers.rs` with checks that ban `GameState` references in `src/server/` (except `mod.rs` and `debug.rs`) and ban `GameState` construction in `tests/`.

**Acceptance criteria:**
- [ ] `check_server_layer_boundaries` detects `GameState` imports/references in `src/server/`
- [ ] `check_test_layer_boundaries` detects `GameState` construction in `tests/`
- [ ] `check_load_state_calls` detects `.load_state()` calls in `src/server/`
- [ ] Each check supports file-level exceptions (`mod.rs`, `debug.rs`)
- [ ] Three new `#[test]` functions registered in `tests/guardrails.rs`

**Verification:**
- [ ] `cargo nextest run --test guardrails` passes
- [ ] Temporarily re-introducing a `GameState` reference in `src/server/` causes the guardrail test to fail

**Dependencies:** Checkpoint after Tasks 5-6

**Files likely touched:**
- `tests/guardrails.rs`
- `tests/guardrails/layers.rs` (new)

**Estimated scope:** Medium (2 files)

#### Task 8: Update Agent Rules and Architecture Docs
**Description:** Document the layer boundary in `.agents/rules/chronicler_engine.md` and `docs/architecture/system.md`.

**Acceptance criteria:**
- [ ] `.agents/rules/chronicler_engine.md` has a "Layer Boundary" section
- [ ] `docs/architecture/system.md` documents the server/domain separation
- [ ] Both documents reference `ApplicationService` as the required boundary

**Verification:**
- [ ] `grep -i "layer boundary" .agents/rules/chronicler_engine.md` finds the new section
- [ ] `grep -i "ApplicationService" docs/architecture/system.md` finds the boundary description

**Dependencies:** Task 7

**Files likely touched:**
- `.agents/rules/chronicler_engine.md`
- `docs/architecture/system.md`

**Estimated scope:** Small (2 files)

### Checkpoint: After Tasks 7-8
- [ ] `cargo nextest run --test guardrails` passes
- [ ] `cargo nextest run --test architecture` passes
- [ ] `python build.py` passes end-to-end

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `get_current_room_view` needs many fields (room, world default image, NPCs) | Medium | Return a composed `RoomView` struct instead of individual fields |
| Integration tests need complex state setup (NPCs in specific rooms) | Medium | Keep `TestGameState` helpers in `test_support/` but hide them behind `create_app_for_testing` variants |
| Guardrails are too strict (false positives) | Low | Explicit exceptions list for `mod.rs` and `debug.rs` |
| Task 6 is large and error-prone | High | Break into multiple sub-sessions; run tests after every 2-3 files |

## Excluded
- Engine/pipeline `GameState` passing — domain-layer coupling, not a leak
- `ApplicationService` internal field mutations — acceptable within application layer
- `state.rs` file splitting — cosmetic, does not reduce blast radius
- `GameServiceContext` visibility — boilerplate, not semantic blast radius
