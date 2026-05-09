# Follow-Up Plan: Complete Defensive Architecture Implementation

**Date:** 2026-05-09
**Status:** Planned
**Based on:** Review of initial Approach 4 implementation
**Goal:** Address gaps in runtime diagnostics, access control, and property-based testing.

---

## Context: What Was Built vs. What Was Planned

The initial implementation delivered:
- ✅ `GameState` decomposition into `MovementState`, `NarrativeState`, `SceneState`
- ✅ `diagnostics` feature with `assert_state_consistency`
- ✅ Basic proptest properties for `add_log` and static state

It omitted or compromised on:
- ❌ Log invariant check (deleted after being too strict)
- ❌ Broad integration of diagnostics (only 2 call sites in `action_processing.rs`)
- ❌ Accessor traits to constrain subsystem access
- ❌ Property tests that exercise actual engine functions (`handle_movement`, `apply_npc_events`)
- ❌ `state_diagnostics.rs` is in `model/` but depends on engine semantics

---

## Difficulties from Initial Implementation (to Avoid This Time)

1. **The log invariant was guessed, not derived.** I assumed Narration must follow Input/Narration/Dialogue/Event. The engine actually produces `System → Narration` sequences legitimately (dynamic room creation). This plan derives the invariant from the actual code (`replace_last_ai_response`) rather than guessing.

2. **`GameState` decomposition required touching ~28 files.** The blast radius was large because every field access changed. This plan avoids further field renames; it works with the decomposed structure as-is.

3. **`#[cfg(test)]` gates `test_support` from integration tests.** Proptest strategies must live in unit tests (`src/*_tests.rs`) or construct state manually.

4. **Architecture linter (`arch-lint`) blocked `model/` from importing `engine/`.** `state_diagnostics.rs` duplicated `get_current_room` logic to avoid the lint. The fix is to move the module to `engine/`.

5. **Guardrails require doc anchors on complex public functions.** Any new public function with control flow needs `// [DOC: ...]` inside the body.

6. **`Clone` was added to `GameState` for proptest.** This is now a permanent trait bound. Future property tests should work with it rather than fight it.

---

## Architecture Decisions

1. **Skip session types with phantom types.** The runtime diagnostics catch mutation-order violations with perfect locality and zero API complexity. Phantom-type builders would add ~100 lines of generic machinery for marginal additional safety. Not worth the maintenance cost per project conventions ("keep it stupidly simple").

2. **Use accessor traits instead.** Traits provide compile-time access control without generics complexity. `engine/` functions will take `&mut impl MovementAccess + NarrativeAccess` instead of `&mut GameState`. This is the intended middle ground from the original plan.

3. **Move `state_diagnostics` to `engine/`.** It checks engine-level invariants (room existence after movement, NPC consistency after quantification). The `model/` layer should not know about these rules. This eliminates the `get_current_room` duplication.

4. **Derive the log invariant from `replace_last_ai_response`.** The actual load-bearing invariant is: "last AI response index > last input index." This is exactly what `replace_last_ai_response` checks. We will replicate that check in diagnostics, not invent a new one.

---

## Task List

### Phase 1: Fix Diagnostics Core

#### Task 1.1: Move `state_diagnostics` to `engine/` Layer

**Description:**
Relocate `src/model/state_diagnostics.rs` to `src/engine/state_diagnostics.rs`. Update imports across the codebase. Remove the duplicated `get_current_room` logic and call `crate::engine::logic::get_current_room` directly.

**Acceptance criteria:**
- [ ] File lives at `src/engine/state_diagnostics.rs`
- [ ] `src/model/mod.rs` no longer declares `pub mod state_diagnostics`
- [ ] `src/engine/mod.rs` declares `pub mod state_diagnostics`
- [ ] `assert_room_exists` uses `get_current_room(state)` directly (no duplication)
- [ ] Architecture test passes (`cargo test --test architecture`)
- [ ] `cargo test --features diagnostics` passes

**Verification:**
- [ ] `python build.py` passes

**Dependencies:** None
**Files touched:** `src/model/mod.rs`, `src/engine/mod.rs`, `src/engine/state_diagnostics.rs` (new), `src/engine/action_processing.rs` (import update)
**Estimated scope:** Small
**Difficulty learned:** Arch-lint ALD003 enforces layer boundaries strictly. Moving the module is cleaner than inlining logic.

---

#### Task 1.2: Restore Correct Log Invariant Check

**Description:**
Add `assert_log_invariants` back to `state_diagnostics.rs`, but this time implement it as an exact copy of the logic in `GameState::replace_last_ai_response`:

```rust
fn assert_log_invariants(state: &GameState) -> Result<(), EngineError> {
    let ai_idx = state.get_last_ai_response_index();
    let input_idx = state.get_last_input_index();
    if let (Some(ai), Some(input)) = (ai_idx, input_idx) {
        if ai <= input {
            return Err(EngineError::Internal(internal_error(
                "last AI response is not after last player input"
            )));
        }
    }
    Ok(())
}
```

**Acceptance criteria:**
- [ ] `assert_log_invariants` exists and is called from `assert_state_consistency`
- [ ] `cargo test --features diagnostics` passes (including trigger tests and e2e tests)
- [ ] Guardrails test passes (doc anchor added inside function)

**Verification:**
- [ ] Run `cargo test --features diagnostics --test trigger_tests` — all 7 tests pass
- [ ] Run `cargo test --features diagnostics --test game_service_tests` — all 26 tests pass

**Dependencies:** Task 1.1
**Files touched:** `src/engine/state_diagnostics.rs`
**Estimated scope:** XS
**Difficulty learned:** The previous attempt guessed the invariant. This task derives it from existing code.

---

#### Task 1.3: Broaden `assert_state_consistency` Integration

**Description:**
Add `assert_state_consistency(state).ok()` to every public mutation function that modifies `GameState`. The `.ok()` swallow is intentional for functions that don't currently return `Result` (to avoid API breakage). For functions that already return `Result`, propagate with `?`.

Target functions:
1. `engine/action_processing.rs`:
   - `handle_movement` (end of function)
   - `apply_npc_events` (end of function)
   - `commit_trigger_narration` (end of function)
   - `evaluate_and_narrate_triggers` (end of function)
2. `engine/game_service.rs`:
   - After `Action::Quit` handler
   - After `Action::Look` handler
   - After `Action::Talk` handler
   - After `Action::Inventory` handler
   - After `Action::FreeAction` completion (both success and error paths)
3. `server/fragments.rs`:
   - After `process_sync_action` in `action_command_handler`
   - After `reset_generating_handler`

**Acceptance criteria:**
- [ ] Every public `GameState` mutation site calls `assert_state_consistency`
- [ ] `cargo test --features diagnostics` passes
- [ ] `cargo test` (without diagnostics) passes
- [ ] No change to public function signatures except `execute_freeaction_impl` (already changed)

**Verification:**
- [ ] `Select-String -Path "src/*.rs" -Pattern "assert_state_consistency" | Measure-Object` returns ≥10 occurrences
- [ ] `python build.py` passes

**Dependencies:** Task 1.2
**Files touched:** `src/engine/action_processing.rs`, `src/engine/game_service.rs`, `src/server/fragments.rs`, `src/engine/state_diagnostics.rs` (import additions)
**Estimated scope:** Medium
**Difficulty learned:** Some mutation functions (`commit_trigger_narration`, `apply_npc_events`) don't return `Result`. Using `.ok()` avoids API breakage while still running checks under the diagnostics feature.

---

### Checkpoint 1: Diagnostics Foundation

- [ ] `cargo test --features diagnostics` passes all suites
- [ ] `python build.py` passes with no warnings
- [ ] Architecture guardrails pass
- [ ] Every public mutation function has a diagnostic assertion

---

### Phase 2: Access Control via Traits

#### Task 2.1: Define Accessor Traits in `model/`

**Description:**
Create `src/model/state_access.rs` with traits that constrain how subsystems interact with `GameState` sub-structs:

```rust
pub trait MovementAccess {
    fn current_room_id(&self) -> &str;
    fn current_room(&self) -> Result<&Room, EngineError>;
    fn set_current_room_id(&mut self, room_id: String);
    fn dynamic_rooms_mut(&mut self) -> &mut HashMap<String, Room>;
}

pub trait NarrativeAccess {
    fn history(&self) -> &[LogEntry];
    fn history_mut(&mut self) -> &mut Vec<LogEntry>;
    fn append_log(&mut self, text: String, sender: Option<String>, log_type: LogType);
    fn next_log_id(&self) -> u64;
    fn increment_log_id(&mut self);
    fn generation_state(&self) -> &GenerationState;
    fn generation_state_mut(&mut self) -> &mut GenerationState;
}

pub trait SceneAccess {
    fn npcs_in_area(&self) -> &[NpcCard];
    fn set_npcs_in_area(&mut self, npcs: Vec<NpcCard>);
}

pub trait CharacterAccess {
    fn npcs(&self) -> &HashMap<String, NpcCard>;
    fn npcs_mut(&mut self) -> &mut HashMap<String, NpcCard>;
    fn character_state(&self) -> &CharacterState;
    fn character_state_mut(&mut self) -> &mut CharacterState;
}
```

Implement these traits for `GameState`.

**Acceptance criteria:**
- [ ] Traits compile
- [ ] `GameState` implements all traits
- [ ] Existing tests pass without modification (backward compatibility)
- [ ] No dead code warnings

**Verification:**
- [ ] `cargo check --tests` passes

**Dependencies:** Checkpoint 1
**Files touched:** `src/model/state_access.rs` (new), `src/model/mod.rs`
**Estimated scope:** Small
**Difficulty learned:** Traits must be defined in `model/` (since `model/` can't import `engine/`). The implementations are straightforward delegation.

---

#### Task 2.2: Migrate `action_processing.rs` to Use Traits

**Description:**
Refactor `action_processing.rs` functions to accept trait bounds instead of `&mut GameState` where possible. Start with the purest cases:

- `get_static_npcs` → `&impl CharacterAccess`
- `handle_movement` → `&mut impl MovementAccess + CharacterAccess`
- `apply_npc_events` → `&mut impl CharacterAccess`

Keep `execute_freeaction_impl` on `&mut GameState` for now (it touches everything).

**Acceptance criteria:**
- [ ] `get_static_npcs` signature uses trait bound
- [ ] `handle_movement` signature uses trait bounds
- [ ] `apply_npc_events` signature uses trait bounds
- [ ] All tests pass
- [ ] Clippy passes

**Verification:**
- [ ] `python build.py` passes

**Dependencies:** Task 2.1
**Files touched:** `src/engine/action_processing.rs`
**Estimated scope:** Small
**Difficulty learned:** Generic trait bounds in function signatures are verbose but compile reliably. The risk is making the code harder to read; we will limit traits to the 3 functions above.

---

### Checkpoint 2: Access Control

- [ ] `python build.py` passes
- [ ] At least 3 engine functions use trait bounds instead of `&mut GameState`
- [ ] `GameState` still compiles and all tests pass

---

### Phase 3: Real Engine Property Tests

#### Task 3.1: Add Proptest for `handle_movement`

**Description:**
Add a property test to `src/engine/action_processing_tests.rs` that verifies room existence after `handle_movement`:

```rust
proptest! {
    #[test]
    fn prop_room_exists_after_handle_movement(
        destination in "[a-z0-9_]{1,20}",
        mut state in Just(TestGameState::in_room("room1"))
    ) {
        handle_movement(&mut state, Some(&destination), &[]);
        prop_assert!(
            state.movement.current_room_id.starts_with("dynamic_")
            || state.map.get_room_by_id(&state.movement.current_room_id).is_some(),
            "current_room_id '{}' must exist after movement",
            state.movement.current_room_id
        );
    }
}
```

**Acceptance criteria:**
- [ ] Property runs 100+ cases without failure
- [ ] Shrinking works on failure
- [ ] `cargo test --lib prop_room_exists_after_handle_movement` passes

**Verification:**
- [ ] `cargo test --lib prop_` shows the new property passing

**Dependencies:** Checkpoint 2
**Files touched:** `src/engine/action_processing_tests.rs`
**Estimated scope:** Small
**Difficulty learned:** `Just` requires `Clone`; `TestGameState::in_room` already produces a cloneable `GameState`. No new trait bounds needed.

---

#### Task 3.2: Add Proptest for `apply_npc_events`

**Description:**
Add a property test that applies random `NpcEvent` sequences and verifies:
1. `times_met` is monotonically increasing for each NPC
2. `character_state` only references known NPCs

```rust
proptest! {
    #[test]
    fn prop_npc_events_are_monotonic_and_consistent(
        mut state in Just(make_test_state()),
        events in prop::collection::vec(
            prop_oneof![
                Just(NpcEvent { npc_id: "carla".into(), event_type: NpcEventType::Entered }),
                Just(NpcEvent { npc_id: "carla".into(), event_type: NpcEventType::Left }),
            ],
            1..50
        )
    ) {
        let initial_times_met = get_times_met(&state.character_state, "carla");
        apply_npc_events(&mut state, &events);
        
        let enter_count = events.iter().filter(|e| e.npc_id == "carla" && matches!(e.event_type, NpcEventType::Entered)).count() as u32;
        let final_times_met = get_times_met(&state.character_state, "carla");
        
        prop_assert_eq!(
            final_times_met,
            initial_times_met + enter_count,
            "times_met should increase by number of Entered events"
        );
        
        for npc_id in state.character_state.npcs.keys() {
            prop_assert!(state.npcs.contains_key(npc_id));
        }
    }
}
```

**Acceptance criteria:**
- [ ] Property runs 100+ cases without failure
- [ ] `times_met` monotonicity is verified
- [ ] NPC consistency is verified

**Verification:**
- [ ] `cargo test --lib prop_npc_events` passes

**Dependencies:** Task 3.1
**Files touched:** `src/engine/action_processing_tests.rs`
**Estimated scope:** Small
**Difficulty learned:** `NpcEvent` and `NpcEventType` need to be imported. `make_test_state` from the same file is the right test fixture.

---

#### Task 3.3: Add Proptest for `execute_freeaction_impl` End-to-End

**Description:**
Add a property test that calls `execute_freeaction_impl` with randomly generated `FreeActionContext` values (using `MockBackend`) and verifies `assert_state_consistency` passes afterward.

This is the highest-value property because it exercises the full pipeline.

**Acceptance criteria:**
- [ ] Property generates random narration text, input text, and quantifier results
- [ ] `execute_freeaction_impl` is called with `MockBackend`
- [ ] `assert_state_consistency(&state)` passes after each call
- [ ] Property runs 100+ cases without failure

**Verification:**
- [ ] `cargo test --lib prop_execute_freeaction` passes

**Dependencies:** Task 3.2
**Files touched:** `src/engine/action_processing_tests.rs`
**Estimated scope:** Medium
**Difficulty learned:** `FreeActionContext` borrows `world`, `player`, `all_npcs`, `history`, and `llm_backend`. The test fixture must outlive the context. This requires careful lifetime management in the test.

---

### Checkpoint 3: Property Tests

- [ ] 3 new proptest properties pass (100+ cases each)
- [ ] `cargo test --lib prop_` passes
- [ ] `python build.py` passes
- [ ] Test suite time increase ≤10% (proptest adds ~5-10s per property)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Broadening `assert_state_consistency` slows tests significantly | Medium | Feature-gated; `.ok()` swallow means zero overhead without `--features diagnostics` |
| Trait bounds make function signatures unreadable | Medium | Limit traits to 3 functions in `action_processing.rs`; keep `execute_freeaction_impl` on `GameState` |
| Proptest `FreeActionContext` lifetime issues | Medium | Use `Arc` clones for `world`, `player`; borrow `MockBackend` from test fixture |
| `arch-lint` blocks trait imports | Low | Traits live in `model/`; implementations live there too; no cross-layer imports needed |

---

## Open Questions

1. **Should `execute_freeaction_impl` itself use trait bounds?** It touches `movement`, `narrative`, `scene`, `character_state`, and `npcs`. A bound like `&mut impl MovementAccess + NarrativeAccess + SceneAccess + CharacterAccess` is verbose but possible. Recommendation: skip for now; the function is already protected by `assert_state_consistency`.

2. **Should diagnostics assertions return `Result` or panic in tests?** Currently they return `Result` and are propagated. In tests, a failure surfaces as `EngineError::Internal`. This is correct but the error message could be clearer. Recommendation: keep `Result`; the error message already names the invariant.

3. **Should we add a `times_met` monotonicity check to `assert_state_consistency`?** `apply_npc_events` increments `times_met`. A diagnostic check could verify it never decreases. Recommendation: add this as part of Task 1.3.
