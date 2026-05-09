# Plan: Defensive Architecture & Invariant Enforcement

**Date:** 2026-05-09
**Status:** Planned
**Goal:** Prevent entire classes of bugs by making invariants unrepresentable or immediately detectable.

---

## Overview

Currently, the engine relies on **documented conventions** for correctness:
- State mutation order in `action_processing.rs` is "load-bearing" (documented in `DEBUGGING.md` and `docs/architecture/invariants.md`)
- `GameState` is a 323-line central struct accessed through `Arc<Mutex<GameState>>`
- Trigger timing rules (evaluate BEFORE incrementing `times_met`) are documented but enforced only by programmer discipline

This plan moves from "document and hope" to **compile-time and runtime enforcement**. We use the type system, session types, and property-based testing to make violations impossible or to catch them at the exact moment they occur.

---

## Background

**Current architectural risks:**

1. **`GameState` is a god object.**
   - 323 lines in `src/model/state.rs`
   - Contains: player state, NPC states, log history, generation state, dynamic rooms, trigger flags
   - Any bug can mutate any part of state from anywhere that holds the `MutexGuard`

2. **State mutation order is conventional.**
   - `action_processing.rs` documents the exact order:
     1. `handle_movement()`
     2. Resolve NPCs from quantifier
     3. `state.add_log(narration)`
     4. `evaluate_and_narrate_triggers()`
     5. `compute_npc_events()` + `apply_npc_events()`
   - Swapping steps 3 and 4 means triggers miss current narration context. Nothing prevents this.

3. **`Arc<Mutex<GameState>>` allows any code to lock and mutate.**
   - `game_service.rs` locks the mutex
   - `server/debug.rs` locks the mutex
   - Tests lock the mutex directly
   - There is no audit trail of who mutated what

4. **No property-based or generative testing.**
   - All tests are example-based (specific inputs, specific outputs)
   - State space is large enough that edge cases are likely missed

---

## Architecture Decisions

1. **Type-system enforcement over runtime checks where possible.** If Rust can enforce an invariant at compile time, we use that. Runtime checks are secondary.
2. **Split `GameState` into smaller, single-responsibility structs.** Each sub-state has its own invariant rules and access patterns.
3. **Session types for ordered operations.** Use Rust's type system (builder pattern with phantom types) to encode valid state transition sequences.
4. **`diagnostics` compile feature for expensive assertions.** Debug and test builds run deep consistency checks; release builds skip them.
5. **Property-based testing for state transitions.** Use `proptest` to generate random sequences of actions and verify invariants hold.

---

## Phase 1: Investigation — Map Invariants and Mutation Surface

### Task 1.1: Catalog All State Mutations
- Audit every function that acquires a `MutexGuard<GameState>` or mutates `GameState`.
- Classify by: what field is mutated, which subsystem owns the logic, whether the mutation is order-dependent.
- **Deliverable:** Table of all mutation sites with `file.rs:line` and `field_mutated`.

### Task 1.2: Identify Enforceable vs. Document-Only Invariants
- Review `docs/architecture/invariants.md` and `DEBUGGING.md`.
- Classify each invariant:
  - **Type-enforceable:** Can be encoded in structs/traits (e.g., "AI response must follow player input")
  - **Session-enforceable:** Can be encoded in builder pattern (e.g., state mutation order)
  - **Runtime-checkable:** Needs assertion after mutation (e.g., `current_room_id` exists in map)
  - **Document-only:** Too contextual to enforce mechanically (e.g., "narration should be atmospheric")
- **Deliverable:** Categorized invariant list with proposed enforcement mechanism.

### Task 1.3: Measure GameState Cohesion
- Count how many distinct subsystems read/write each field in `GameState`.
- Identify fields with high cross-subsystem access (candidates for extraction).
- **Deliverable:** Heat map of `GameState` field access by subsystem.

---

## Phase 2: Implementation — State Decomposition

### Task 2.1: Extract Sub-State Objects
Split `GameState` into focused structs:

```rust
pub struct GameState {
    pub movement: MovementState,       // current_room_id, dynamic_rooms
    pub narrative: NarrativeState,     // log_history, generation_state
    pub characters: CharacterState,    // npcs, player, times_met, triggers_fired
    pub world: Arc<WorldCard>,         // immutable reference data
    pub settings: GameSettings,        // runtime settings
}

pub struct MovementState {
    pub current_room_id: String,
    pub dynamic_rooms: HashMap<String, Room>,
}

pub struct NarrativeState {
    pub history: Vec<LogEntry>,
    pub generation: GenerationState,
}

pub struct CharacterState {
    pub player: Arc<PlayerCard>,
    pub npcs: HashMap<String, NpcCard>,
    pub met_counters: HashMap<String, u32>,
    pub fired_triggers: HashMap<String, Vec<bool>>,
}
```

- **File:** `src/model/state.rs` (refactor)
- **Acceptance criteria:**
  - [ ] `GameState` fields are grouped into sub-structs
  - [ ] All existing tests compile and pass
  - [ ] No change to public API behavior (purely internal reorganization)

### Task 2.2: Add Accessor Traits
Define traits that constrain how subsystems access state:

```rust
pub trait MovementAccess {
    fn current_room(&self) -> Result<&Room, EngineError>;
    fn move_to(&mut self, room_id: String) -> Result<(), EngineError>;
}

pub trait NarrativeAccess {
    fn append_log(&mut self, entry: LogEntry) -> Result<(), EngineError>;
    fn last_player_input(&self) -> Option<&LogEntry>;
}
```

- **File:** `src/model/state_access.rs` (new)
- **Acceptance criteria:**
  - [ ] Each subsystem uses only the trait it needs
  - [ ] `engine/` uses `MovementAccess + NarrativeAccess + CharacterAccess`
  - [ ] `server/` uses read-only traits where possible

---

## Phase 3: Implementation — Session Types for Mutation Order

### Task 3.1: Design ActionProcessor Session Type
Use phantom types to enforce the state mutation order:

```rust
struct ActionProcessor<Stage> {
    state: Arc<Mutex<GameState>>,
    _stage: std::marker::PhantomData<Stage>,
}

struct PreMovement;
struct PostMovement;
struct PostNarration;
struct PostTriggers;

impl ActionProcessor<PreMovement> {
    fn handle_movement(self, input: &str) -> Result<ActionProcessor<PostMovement>, EngineError> {
        // ... mutate movement state ...
        Ok(ActionProcessor { state: self.state, _stage: PhantomData })
    }
}

impl ActionProcessor<PostMovement> {
    fn resolve_npcs(self) -> Result<ActionProcessor<PostMovement>, EngineError> {
        // ... quantifier call ...
        Ok(self)
    }

    fn add_narration(self, text: String) -> Result<ActionProcessor<PostNarration>, EngineError> {
        // ... append to log ...
        Ok(ActionProcessor { state: self.state, _stage: PhantomData })
    }
}

impl ActionProcessor<PostNarration> {
    fn evaluate_triggers(self) -> Result<ActionProcessor<PostTriggers>, EngineError> {
        // ... trigger eval ...
        Ok(ActionProcessor { state: self.state, _stage: PhantomData })
    }
}
```

- **File:** `src/engine/action_processor.rs` (new) or refactor `action_processing.rs`
- **Acceptance criteria:**
  - [ ] Swapping mutation steps is a compile-time error
  - [ ] All existing behavior preserved
  - [ ] Tests for `action_processing.rs` still pass

### Task 3.2: Add Compile-Time Invariant Checks
Where session types are too heavy, use Rust's type system directly:

```rust
// Ensures add_log can only be called with a LogEntry that has a valid predecessor
pub struct ValidatedLogEntry(LogEntry);

impl ValidatedLogEntry {
    pub fn new_ai_response(
        previous: &LogEntry,
        text: String,
    ) -> Result<Self, EngineError> {
        if previous.log_type != LogType::PlayerInput {
            return Err(EngineError::Internal(internal_error(
                "AI response must follow player input"
            )));
        }
        Ok(ValidatedLogEntry(LogEntry { log_type: LogType::AiNarration, text }))
    }
}
```

- **File:** `src/model/state.rs` or `src/model/log.rs`
- **Acceptance criteria:**
  - [ ] Invalid log sequences are rejected at the call site
  - [ ] Existing valid log sequences compile unchanged

---

## Phase 4: Implementation — Runtime Diagnostic Assertions

### Task 4.1: Add `diagnostics` Feature Flag
Add to `Cargo.toml`:
```toml
[features]
default = []
diagnostics = []
```

- **File:** `Cargo.toml`

### Task 4.2: Implement Consistency Check Functions
Add functions that verify state consistency after mutations:

```rust
#[cfg(feature = "diagnostics")]
fn assert_state_consistency(state: &GameState) -> Result<(), EngineError> {
    // current_room_id must exist in map or dynamic_rooms
    if state.movement.current_room().is_err() {
        return Err(EngineError::Internal(internal_error(
            "current_room_id not found in map or dynamic_rooms"
        )));
    }

    // history must alternate PlayerInput / AiNarration / System
    // (validate ordering invariant)

    // times_met must not decrease
    // triggers_fired length must match npc.triggers length

    Ok(())
}
```

- **File:** `src/model/state_diagnostics.rs` (new)
- **Acceptance criteria:**
  - [ ] Every public mutation function calls `assert_state_consistency` on `#[cfg(feature = "diagnostics")]`
  - [ ] `cargo test` runs with `--features diagnostics` and passes
  - [ ] `cargo build --release` does not include diagnostics (verify via `cargo expand` or inspection)

---

## Phase 5: Implementation — Property-Based Testing

### Task 5.1: Add `proptest` Dependency
```toml
[dev-dependencies]
proptest = "1.0"
```

- **File:** `Cargo.toml`

### Task 5.2: Write State Transition Properties
Define properties that must hold for any sequence of actions:

1. **Room existence:** After any action, `current_room_id` is either in the map or `dynamic_rooms`.
2. **Log alternation:** History never has two consecutive `PlayerInput` or two consecutive `AiNarration` entries.
3. **Monotonic times_met:** `times_met` for any NPC never decreases.
4. **Trigger idempotency:** A non-repeatable trigger, once fired, never fires again for the same NPC.

```rust
#[cfg(test)]
mod prop_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn room_always_exists_after_action(actions in prop::collection::vec(any_action(), 1..10)) {
            let mut state = make_test_state();
            for action in actions {
                execute_action(&mut state, action);
                prop_assert!(state.movement.current_room().is_ok());
            }
        }
    }
}
```

- **File:** `tests/prop_state_tests.rs` (new)
- **Acceptance criteria:**
  - [ ] Properties run for 10,000+ random action sequences without failure
  - [ ] Each property failure produces a minimal shrinking example
  - [ ] Properties run in CI (or locally with `cargo test --features proptest`)

---

## Phase 6: Verification

### Task 6.1: Introduce Deliberate Invariant Violations
- Create a branch that swaps mutation order 3 and 4 in `action_processing.rs`.
- Verify:
  - Session type approach: Compile-time error (Approach 4 success)
  - Runtime diagnostics: `assert_state_consistency` catches it with clear message
  - Property tests: Proptest finds a counterexample
- **Acceptance criteria:**
  - [ ] At least one enforcement layer catches every deliberate violation

### Task 6.2: Measure Mutation Surface Reduction
- Count lines of code that directly mutate `GameState` before and after refactoring.
- **Acceptance criteria:**
  - [ ] Direct `GameState` field mutations reduced by ≥50%
  - [ ] All mutations now go through typed accessors or session-type stages

### Task 6.3: Performance Regression Check
- Run `cargo build --release` before and after.
- Run the full test suite.
- **Acceptance criteria:**
  - [ ] Release binary size change ≤5%
  - [ ] Test suite runtime change ≤10%
  - [ ] No new clippy warnings or architecture violations

---

## Dependencies

| Task | Depends on | Blocks |
|------|-----------|--------|
| 1.1 Catalog mutations | None | 1.2, 1.3, 2.1 |
| 1.2 Classify invariants | None | 2.2, 3.1, 3.2, 4.2, 5.2 |
| 1.3 Measure cohesion | 1.1 | 2.1 |
| 2.1 Extract sub-states | 1.1, 1.3 | 2.2, 3.1, 4.2, 6.2 |
| 2.2 Add traits | 2.1 | 3.1, 6.2 |
| 3.1 Session types | 1.2, 2.1, 2.2 | 6.1 |
| 3.2 Compile-time checks | 1.2 | 6.1 |
| 4.1 Feature flag | None | 4.2 |
| 4.2 Runtime assertions | 1.2, 2.1, 4.1 | 6.1 |
| 5.1 Add proptest | None | 5.2 |
| 5.2 Write properties | 1.2, 5.1 | 6.1 |
| 6.1 Deliberate violations | 3.1, 3.2, 4.2, 5.2 | — |
| 6.2 Mutation surface | 2.1, 2.2 | — |
| 6.3 Performance | 2.1, 3.1, 5.2 | — |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Refactoring breaks existing tests | Make changes incrementally; run full test suite after each sub-state extraction |
| Session types are too complex for future maintainers | Document with examples; keep the builder simple (no generic traits beyond stage phantom) |
| Runtime diagnostics slow tests too much | Make diagnostics a feature flag; only enable in test builds |
| Proptest finds pre-existing bugs | Those are real bugs — fix them. Budget time for unexpected findings. |
| `Arc<Mutex<>>` makes session types awkward | Hold the lock for the entire action processing pipeline; sub-states are fields inside the locked `GameState` |

---

## Success Criteria

1. Swapping state mutation order is a compile-time error (session types) or caught immediately by runtime diagnostics.
2. `GameState` direct field mutations are reduced by ≥50%; most access goes through typed accessors.
3. Property-based tests find ≥1 previously uncaught edge case or confirm the state space is safe.
4. No performance regression in release builds.
5. All existing tests pass without modification to test logic (only import updates).
