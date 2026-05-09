# Chronicler Engine: Defensive Architecture Overview

**Purpose:** Frame of reference for future changes. Describes the engine's layered architecture, state decomposition, and invariant enforcement after the defensive-architecture refactor.

---

## 1. Layer Cake (Inner → Outer)

```
┌─────────────────────────────────────────┐
│  server/        HTTP + WebSocket layer  │  Axum, HTMX, HTML fragments
│  ─────────────────────────────────────  │  Holds Arc<Mutex<GameState>>
├─────────────────────────────────────────┤
│  narrative/     LLM integration         │  Prompt builders, quantifier,
│  ─────────────────────────────────────  │  LLM backends (Ollama, OpenRouter)
├─────────────────────────────────────────┤
│  engine/        Game rules + logic      │  Action parsing, state mutations,
│  ─────────────────────────────────────  │  trigger evaluation, diagnostics
├─────────────────────────────────────────┤
│  model/         Pure data structures    │  GameState, WorldCard, NpcCard,
│  ─────────────────────────────────────  │  serde, NO business logic
├─────────────────────────────────────────┤
│  test_support/  Test fixtures           │  TestGameState, TestNpc, TestMap
└─────────────────────────────────────────┘
```

**Rule:** Inner layers never import outer layers. `model/` cannot see `engine/`. Enforced by `arch-lint`.

---

## 2. GameState Decomposition

Before: one flat 323-line struct with 15+ fields.

After: grouped sub-structs with clear ownership.

```rust
pub struct GameState {
    // Immutable reference data (cheap to clone)
    pub world:  Arc<WorldCard>,
    pub map:    Arc<MapDef>,
    pub player: Arc<PlayerCard>,

    // Mutable sub-states
    pub movement:        MovementState,   // rooms, navigation
    pub narrative:       NarrativeState,  // history, generation UI state
    pub scene:           SceneState,      // who's currently visible
    pub character_state: CharacterState,  // trigger tracking, times_met

    // NPC database
    pub npcs: HashMap<String, NpcCard>,
}
```

### What Lives Where

| Sub-state | Owns | Example Access |
|-----------|------|----------------|
| `movement` | `current_room_id`, `dynamic_rooms` | `state.movement.current_room_id` |
| `narrative` | `history`, `next_log_id`, `generation` | `state.narrative.history` |
| `scene` | `npcs_in_area` | `state.scene.npcs_in_area` |
| `character_state` | `npcs` map of `NpcEncounterState` | `state.character_state.npcs["carla"].times_met` |

**Important:** Fields are still `pub`. Any code with `&mut GameState` can mutate anything. The grouping helps *humans* understand ownership; Rust does not enforce it yet.

---

## 3. The Action Pipeline (One FreeAction)

```
Player types "look around"
        │
        ▼
┌───────────────┐
│  fragments.rs │  HTTP handler receives POST
│  (server)     │  Parses command, adds Input log
└───────┬───────┘
        │  spawns tokio::task::spawn_blocking
        ▼
┌───────────────┐
│ game_service  │  Locks GameState, clones needed data
│ (engine)      │  Drops lock before LLM call
└───────┬───────┘
        │  LLM call (narrate_action)
        ▼
┌───────────────┐
│  narrative/   │  Generates narration text
│  llm backend  │  e.g. "You see Carla by the fire."
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  quantifier   │  Analyzes text: which NPCs? Movement?
│  (narrative)  │  Returns QuantifierResult
└───────┬───────┘
        │
        ▼
┌───────────────────────────┐
│ execute_freeaction_impl   │  THE LOAD-BEARING MUTATION SEQUENCE
│ (engine/action_processing)│  Order matters. Swapping steps = bugs.
│                           │
│  1. handle_movement()     │  ← updates current_room_id, dynamic_rooms
│  2. add_log(narration)    │  ← appends to narrative.history
│  3. scene.npcs_in_area =  │  ← updates who is visible
│  4. evaluate_triggers()   │  ← reads mutated state, may fire
│  5. apply_npc_events()    │  ← increments times_met, sets currently_meeting
│                           │
│  assert_state_consistency │  ← runtime invariant check (diagnostics feature)
└───────────────────────────┘
        │
        ▼
┌───────────────┐
│  If trigger   │  Second LLM call for trigger narration
│  fired:       │  commit_trigger_narration()
└───────────────┘
```

**Why the order is load-bearing:**
- Step 2 (log narration) must happen before Step 4 (evaluate triggers) because triggers read `history` for context.
- Step 5 (NPC events) must happen after Step 4 because trigger evaluation depends on pre-event state.
- There is **no compile-time enforcement** of this order. It is documented in `docs/architecture/invariants.md` and checked by runtime diagnostics.

---

## 4. Invariant Enforcement

### The Four Runtime Invariants

Checked by `engine/state_diagnostics.rs` after state mutations.

| ID | Invariant | When It Fires | Checked After |
|----|-----------|---------------|---------------|
| **INV-ROOM** | `current_room_id` exists in map or `dynamic_rooms` | Player teleports to non-existent room | `handle_movement` |
| **INV-NPC** | Every NPC in `scene.npcs_in_area` exists in `state.npcs` | Quantifier injects phantom NPC | `apply_npc_events`, `commit_trigger_narration`, `execute_freeaction_impl` |
| **INV-CHAR** | `character_state` only references loaded NPCs | NPC unloaded but trigger state remains | `apply_npc_events`, `execute_freeaction_impl` |
| **INV-LOG** | Last AI response index > last player input index | `replace_last_ai_response` precondition violated | `commit_trigger_narration`, `evaluate_and_narrate_triggers` |

### How Diagnostics Work

```rust
// In engine/state_diagnostics.rs

#[cfg(feature = "diagnostics")]
pub fn assert_state_consistency(state: &GameState) -> Result<(), EngineError> {
    assert_room_exists(state)?;
    assert_npc_consistency(state)?;
    assert_character_state_consistency(state)?;
    assert_log_invariants(state)?;
    Ok(())
}

#[cfg(not(feature = "diagnostics"))]
pub fn assert_state_consistency(_state: &GameState) -> Result<(), EngineError> {
    Ok(())  // zero cost
}
```

**Feature flag:**
- `cargo test --features diagnostics` — checks run, tests may fail if invariant violated
- `cargo build --release` — checks compile away to nothing

**Current swallow pattern:**
```rust
assert_state_consistency(state).ok();  // logs error, does not fail test
```
*Known issue:* This hides failures in tests. See review doc for fix.

---

## 5. Testing Strategy

```
                    ┌─────────────────┐
                    │   E2E Tests     │  Browser automation (Playwright)
                    │   24 tests      │  Full stack: HTTP → server → engine
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Integration    │  Game service tests, trigger tests
                    │  ~100 tests     │  Mock backends, real state mutations
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Property Tests │  Proptest: random sequences
                    │  7 properties   │  verify invariants hold across
                    │  100+ cases each│  random action sequences
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   Unit Tests    │  Individual functions, edge cases
                    │  474 tests      │  state logic, parsing, prompt building
                    └─────────────────┘
```

### Property Tests in Detail

| Property | What It Checks | File |
|----------|----------------|------|
| `prop_log_ids_are_strictly_increasing` | `add_log` increments `next_log_id` | `model/state_tests.rs` |
| `prop_log_history_never_exceeds_max_capacity` | History capped at 1000 entries | `model/state_tests.rs` |
| `prop_npcs_in_area_are_always_known` | `npcs_in_area` subset of `npcs` | `model/state_tests.rs` |
| `prop_character_state_references_valid_npcs` | `character_state` subset of `npcs` | `model/state_tests.rs` |
| `prop_handle_movement_preserves_state_consistency` | Movement doesn't break invariants | `engine/action_processing_tests.rs` |
| `prop_apply_npc_events_preserves_state_consistency` | NPC events don't break invariants | `engine/action_processing_tests.rs` |
| `prop_execute_freeaction_impl_preserves_state_consistency` | Full pipeline doesn't break invariants | `engine/action_processing_tests.rs` |

**Limitation:** Property tests call `assert_state_consistency`. They do not independently verify numeric invariants (e.g., "`times_met` increased by exactly 1").

---

## 6. Central State Pattern

```
┌─────────────────────────────────────┐
│  AppState (server/mod.rs)           │
│  ────────────────────────────────   │
│  state: Arc<Mutex<GameState>>       │
│  game_service: Arc<dyn GameService> │
│  cancel_token: CancellationToken    │
└─────────────────────────────────────┘
```

**Rules:**
1. Lock the mutex → do work → drop lock immediately.
2. Long-running LLM calls happen **outside** the lock. The frontend polls `generating_status_handler` to see progress.
3. `GeneratingGuard` (RAII) sets `status = Generating` on construct and resets to `Idle` on drop, even if the mutex is poisoned.

---

## 7. Key Files for Each Concern

| Concern | Primary File | Tests |
|---------|--------------|-------|
| State definition | `model/state.rs` | `model/state_tests.rs` |
| State decomposition | `model/state.rs` (MovementState, NarrativeState, SceneState) | `model/state_tests.rs` |
| Invariant checks | `engine/state_diagnostics.rs` | implicit via all tests with `--features diagnostics` |
| Action pipeline | `engine/action_processing.rs` | `engine/action_processing_tests.rs` |
| Trigger logic | `engine/trigger_eval.rs` | `engine/trigger_eval_tests.rs` |
| Game service orchestration | `engine/game_service.rs` | `tests/game_service_tests.rs` |
| LLM backends | `narrative/llm/` | `narrative/llm/*_tests.rs` |
| Quantifier | `narrative/quantifier/` | `narrative/quantifier/*_tests.rs` |
| HTTP handlers | `server/fragments.rs` | `tests/component_tests.rs`, `tests/e2e_tests.rs` |
| Architecture guardrails | `arch-lint.toml` | `tests/architecture.rs` |

---

## 8. What's Guarded vs. What's Conventional

| Protection | Mechanism | Status |
|------------|-----------|--------|
| No `.unwrap()` in production | Clippy deny + arch-lint | Enforced |
| `model/` pure (no engine imports) | arch-lint scope deps | Enforced |
| `GameState` decomposition | Type system (sub-structs) | Present, not enforced |
| Mutation order in action pipeline | Documentation + runtime diagnostics | Conventional |
| Accessor traits (`MovementAccess`) | Not implemented | Skipped |
| Session types for mutation order | Not implemented | Rejected as too complex |
| Property-based testing | Proptest | Active |

---

## 9. Adding a New Feature: Checklist

1. **Spec first** — update `docs/system/` or `docs/architecture/` before touching code.
2. **Model second** — if new data: add to `model/`, update `GameState` or sub-struct.
3. **Engine third** — add logic to `engine/`, call `assert_state_consistency` at end of mutation.
4. **Tests fourth** — unit tests first, then property test if state transition involved.
5. **Guardrails last** — run `python build.py`.

---

## 10. Glossary

| Term | Meaning |
|------|---------|
| **Quantifier** | LLM-powered module that reads narration text and decides which NPCs are present and whether the player moved. |
| **Trigger** | Condition + action attached to an NPC. E.g., "if times_met == 0, narrate an introduction." |
| **Dynamic room** | Room created at runtime when the player moves to a non-existent location. |
| **GeneratingGuard** | RAII wrapper that sets `generation.status = Generating` and resets to `Idle` on drop. |
| **FreeAction** | Any player input that is not a built-in command (Look, Talk, Inventory, Quit). Requires LLM narration. |
