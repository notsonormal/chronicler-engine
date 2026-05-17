# Plan: GameState Extensibility Fix

## Problem

Adding any new systemic concern to `GameState` (e.g., combat state, quests, dynamic player attributes) requires manual updates across:
- The `GameState` struct definition (`src/model/state.rs`)
- `GameState::new` constructor
- `GameState::from_snapshot` constructor
- `GameStateSnapshot` and `NarrativeSnapshot` in `src/model/state_snapshot.rs`
- At least 15+ manual struct literal constructions across unit tests, integration tests, and fixtures
- Five separate mock constructors in `src/test_support/fixtures.rs`

## Root Cause

`GameState` is constructed via raw struct literals in ~15+ locations across the codebase. Rust requires struct literals to list every field. When a field is added, every literal breaks. There is no enforced indirection.

## Current State (Verified)

The codebase compiles and tests compile successfully. The field names are consistent:
- `GameState.npc_encounter_log: NpcEncounterLog`
- `NarrativeState.input_buffer: InputBuffer`
- `NarrativeSnapshot.input_buffer: InputBuffer` (with serde rename from `generation`)
- `GameStateSnapshot.npc_encounter_log: NpcEncounterLog` (with serde rename from `character_state`)

## Solution

### 1. Introduce `GameStateBuilder`

Create a `GameStateBuilder` in `src/model/state.rs` (or `src/model/state_builder.rs`) following the manual builder pattern already established by `PromptBuilder` in the codebase.

```rust
pub struct GameStateBuilder {
    world: Arc<WorldCard>,
    map: Arc<MapDef>,
    player: Arc<PlayerCard>,
    starting_room: String,
    npcs: Vec<NpcCard>,
    narrative: Option<NarrativeState>,
    scene: Option<SceneState>,
    npc_encounter_log: Option<NpcEncounterLog>,
}

impl GameStateBuilder {
    pub fn new(world: Arc<WorldCard>, map: Arc<MapDef>, player: Arc<PlayerCard>, starting_room: impl Into<String>) -> Self;
    pub fn with_npcs(mut self, npcs: Vec<NpcCard>) -> Self;
    pub fn with_narrative(mut self, narrative: NarrativeState) -> Self;
    pub fn with_scene(mut self, scene: SceneState) -> Self;
    pub fn with_npc_encounter_log(mut self, log: NpcEncounterLog) -> Self;
    pub fn build(self) -> GameState;
}
```

New fields added to `GameState` in the future are added to the builder with `Default::default()` fallback, so existing call sites do not break.

### 2. Mark `GameState` with `#[non_exhaustive]`

This prevents integration tests (which are external crates) from constructing `GameState` with struct literals. Unit tests inside `src/` can still use literals, but the builder will be the encouraged and documented path.

### 3. Refactor `GameState::new` to use the builder

```rust
pub fn new(world, map, player, npcs, starting_room) -> Self {
    GameStateBuilder::new(world, map, player, starting_room)
        .with_npcs(npcs)
        .build()
}
```

### 4. Refactor All Manual `GameState { ... }` Literals

Replace every manual struct literal with either:
- `GameState::new(...)` for simple cases
- `GameStateBuilder` for cases needing custom field values
- `TestGameState::*` helpers for tests

Files with manual literals to refactor:
- `src/test_support/fixtures.rs` (2 raw constructors: `with_npc_raw`, `with_npc_in_named_room_raw`)
- `src/test_support/context_tests.rs` (1 `minimal_state`)
- `src/engine/trigger_eval_tests.rs` (1 `make_state`)
- `src/engine/logic_tests.rs` (1 `setup_test_state` - check if literal or `GameState::new`)
- `src/application/game_service/helpers_tests.rs` (1 `minimal_state`)
- `src/model/state_snapshot_tests.rs` (2 test states)
- `tests/components.rs` (1 `create_test_state` - check if literal or `GameState::new`)

### 5. Update `TestGameState` Fixtures to Use Builder Internally

All `TestGameState` helpers (`in_room`, `with_npc`, `with_npcs`, `with_npc_raw`, `with_npc_in_named_room_raw`) should delegate to `GameStateBuilder`. This makes them immune to future field additions.

### 6. Update State Snapshot Types

`GameStateSnapshot` and `NarrativeSnapshot` must continue to use direct field access (these are internal persistence types and are acceptable). `apply_to` and `from_game_state` must be updated when fields change, but this is expected for persistence contracts.

## Verification

Success criteria:
1. `cargo check` passes with zero errors
2. `cargo test --no-run` passes (all tests compile)
3. `cargo test` passes
4. Adding a new field to `GameState` (e.g., `pub combat: Option<CombatState>`) requires changes in only:
   - `GameState` struct definition
   - `GameStateBuilder` (one line: `combat: Option<CombatState>` with default)
   - `GameStateSnapshot` (if the field needs persistence)
   - No changes to any test file or fixture

## Files to Modify

| File | Change |
|------|--------|
| `src/model/state.rs` | Add `GameStateBuilder`, add `#[non_exhaustive]` to `GameState`, refactor `GameState::new` |
| `src/test_support/fixtures.rs` | Use builder internally in all `TestGameState` helpers |
| `src/test_support/context_tests.rs` | Use `TestGameState::in_room` or builder instead of manual literal |
| `src/engine/trigger_eval_tests.rs` | Use builder in `make_state` |
| `src/engine/logic_tests.rs` | Use builder or `GameState::new` in `setup_test_state` |
| `src/application/game_service/helpers_tests.rs` | Use builder or `TestGameState::in_room` instead of manual literal |
| `src/model/state_snapshot_tests.rs` | Use builder or `TestGameState::in_room` instead of manual literals |
| `tests/components.rs` | Use builder or `GameState::new` instead of manual literal (if applicable) |

## Trade-offs

- **Builder adds ~60 lines of boilerplate** in `state.rs`, but eliminates ~150+ lines of brittle struct literals across tests.
- **`#[non_exhaustive]` prevents integration tests from using struct literals**, which is the desired enforcement. Unit tests inside `src/` can still use literals, but convention + code review should steer them toward the builder.
- **Snapshot types still need manual updates** when fields change, but this is acceptable because persistence is an explicit contract.
