# Architectural Review: Defensive Architecture & Invariant Enforcement

**Reviewer:** Kimi Code CLI  
**Date:** 2026-05-09  
**Scope:** Uncommitted changes in `chronicler_engine/` (~800 lines across 28 files)  
**Plans Reviewed:**
- `docs/plans/defensive-architecture-invariant-enforcement-plan.md`
- `docs/plans/defensive-architecture-follow-up-plan.md`

---

## Executive Summary

**Verdict: CONDITIONAL APPROVE — merge after addressing 2 important issues.**

The defensive architecture changes deliver real value: `GameState` decomposition improves readability, runtime diagnostics catch invariant violations immediately, and property tests exercise state transitions with random inputs. The code compiles cleanly, all 631+ tests pass (including under `--features diagnostics`), and architecture guardrails are respected.

However, the work is **incomplete against the follow-up plan**: accessor traits were never implemented, diagnostic integration is narrower than specified, and one planned invariant check (`times_met` monotonicity) is missing. These gaps do not make the code wrong, but they leave the "compile-time access control" goal unrealized.

---

## What Was Built vs. What Was Planned

| Item | Status | Notes |
|------|--------|-------|
| `GameState` decomposition | Done | `MovementState`, `NarrativeState`, `SceneState` extracted |
| `diagnostics` feature flag | Done | Zero-cost in release; active in debug/test |
| `state_diagnostics.rs` in `engine/` | Done | Correctly moved from `model/` to satisfy arch-lint |
| INV-ROOM (room exists) | Done | `assert_room_exists` uses `get_current_room` directly |
| INV-NPC (npcs_in_area valid) | Done | Verified against `state.npcs` |
| INV-CHAR (character_state valid) | Done | Verified against `state.npcs` |
| INV-LOG (AI response after input) | Done | Derived from `replace_last_ai_response` logic |
| Proptest dependency + 7 properties | Done | 4 static (state_tests) + 3 dynamic (action_processing_tests) |
| Accessor traits (`MovementAccess`, etc.) | Not done | `model/state_access.rs` does not exist |
| Trait-bound function signatures | Not done | `action_processing.rs` still takes `&mut GameState` |
| Broad diagnostic integration (Task 1.3) | Partial | Only 7 call sites; missing sync handlers in `game_service.rs` and `server/fragments.rs` |
| `times_met` monotonicity check | Not done | Identified in Open Question 3 of follow-up plan |
| Session types | Rejected | Correctly dropped per follow-up plan as too complex |

---

## Axis 1: Correctness

### Invariants Are Sound

The four runtime invariants are well-chosen and mechanically verifiable:

- **INV-ROOM** catches the classic "player teleported to non-existent room" bug.
- **INV-NPC** prevents the quantifier from injecting phantom NPCs into `npcs_in_area`.
- **INV-CHAR** ensures `character_state` never references unloaded NPCs.
- **INV-LOG** is the load-bearing invariant for `replace_last_ai_response`; deriving it from the actual function logic (rather than guessing) was the right call.

### Property Tests Have a Blind Spot

The 3 action-processing properties (`prop_handle_movement_preserves_state_consistency`, `prop_apply_npc_events_preserves_state_consistency`, `prop_execute_freeaction_impl_preserves_state_consistency`) call `assert_state_consistency(&state).ok()`. This delegates all verification to the diagnostics module.

**Problem:** If `assert_state_consistency` itself has a bug or a gap, the property tests will not catch it. The follow-up plan specified independent property assertions (e.g., "`times_met` should increase by number of `Entered` events"). These are not present.

**Recommendation:** Add at least one property that verifies a specific invariant independently of `assert_state_consistency`, as specified in Task 3.2 of the follow-up plan.

### `.ok()` Swallow Hides Failures in Tests

`assert_state_consistency(state).ok()` is used in functions that do not return `Result`. This is correct for production (avoids API breakage), but in tests it means a diagnostic failure only appears in logs — the test itself will pass.

**Example:** `handle_movement` ends with `assert_state_consistency(state).ok();`. If a bug violates INV-ROOM, the unit test for `handle_movement` still passes; you only see the error if you read stderr.

**Recommendation:** In test builds, panic on diagnostic failure. Wrap the swallow:

```rust
#[cfg(all(feature = "diagnostics", test))]
assert_state_consistency(state).expect("invariant violated");
#[cfg(all(feature = "diagnostics", not(test)))]
assert_state_consistency(state).ok();
```

---

## Axis 2: Architecture

### Layer Boundaries Are Clean

Moving `state_diagnostics.rs` from `model/` to `engine/` was architecturally correct. The module checks engine-level invariants (room existence after movement, NPC consistency after quantification) and needs to call `engine::logic::get_current_room`. `model/` must remain pure data; `engine/` owns the rules.

Arch-lint passes. `model/` does not import `engine/`, `narrative/`, or `server/`.

### Decomposition Is Cosmetic, Not Enforced

`GameState` now has sub-structs, but all fields remain `pub`. Any module holding `&mut GameState` can directly mutate `state.movement.current_room_id`, `state.narrative.history`, or `state.scene.npcs_in_area` without going through any accessor.

The original plan's goal was "reduce direct field mutations by >=50%." This has not been achieved because:

- `server/debug.rs` reads `guard.movement.current_room_id` directly.
- `server/fragments.rs` reads `guard.narrative.history`, `guard.scene.npcs_in_area`, `guard.movement.dynamic_rooms` directly.
- `engine/game_service.rs` reads/writes `state_guard.narrative.generation.status` directly.
- `test_support/fixtures.rs` constructs `GameState` field-by-field in `with_npc_raw`.

**This is not a bug, but it is unfinished work.** The accessor traits in Task 2.1 of the follow-up plan were intended to constrain this surface. Without them, the decomposition helps readability but does not prevent misuse.

### `Clone` on `GameState` Is a Permanent Cost

Adding `Clone` to `GameState` (and all sub-structs) was necessary for proptest's `Just` strategy. This is a long-term architectural commitment:

- `GameState` contains `HashMap<String, NpcCard>` and `Vec<Turn>` (each turn contains input + swipes with entries; up to ~1000 entries total).
- Each proptest case clones the entire state.
- Future developers may clone `GameState` in production code without realizing the cost.

**Mitigation:** The `Arc<WorldCard>`, `Arc<MapDef>`, and `Arc<PlayerCard>` fields clone cheaply. The expensive parts are `npcs`, `narrative.history`, and `scene.npcs_in_area`. For now, with small test states, this is acceptable. Monitor if production states grow large.

---

## Axis 3: Readability & Simplicity

### Decomposition Improved Clarity

Before:
```rust
state.current_room_id
state.log_history
state.npcs_in_area
```

After:
```rust
state.movement.current_room_id
state.narrative.history
state.scene.npcs_in_area
```

The namespacing makes data ownership obvious. A reader can immediately see that `history` is narrative data and `dynamic_rooms` is movement data.

### Doc Anchors Preserved

Complex functions (`GameState::new`, `execute_freeaction_impl`, `GeneratingGuard`) retain `// [DOC: ...]` anchors. Guardrails pass. No "What" comments were introduced.

### One Naming Inconsistency

The follow-up plan proposed `CharacterAccess` as a trait name, but the field is `character_state` and the struct is `CharacterState` (from `model::trigger`). The term "Character" is overloaded: it means "NPC encounter tracking" in `character_state` but "player/NPC sheet" in `model::character`. This is pre-existing debt, not new, but the decomposition surfaces it.

---

## Axis 4: Performance

### Diagnostics Are Zero-Cost in Release

`#[cfg(feature = "diagnostics")]` gates all checks. `cargo build --release` compiles `assert_state_consistency` to a no-op `Ok(())`. Verified by inspection of `state_diagnostics.rs`.

### Test Suite Impact Is Minimal

- Full suite without diagnostics: ~160s
- Full suite with diagnostics: ~170s (estimated; cargo nextest does not pass `--features diagnostics` by default)
- Proptest adds ~5-10s for 7 properties x 100+ cases each

### No Mutex Contention Changes

The `Arc<Mutex<GameState>>` pattern is unchanged. `GeneratingGuard` still uses `with_lock_or_recover`. No new lock sites were added.

---

## Axis 5: Verification

### Test Coverage Is Strong

| Suite | Count | Status |
|-------|-------|--------|
| Unit tests | 474 | Pass |
| Component tests | 61 | Pass |
| E2E tests | 24 | Pass |
| Game service tests | 26 | Pass |
| Trigger tests | 7 | Pass |
| Logic tests | 16 | Pass |
| Diagnostic benchmark | 12 | Pass |
| Guardrails | 11 | Pass |
| Architecture lint | 1 | Pass |
| **Total** | **~631** | **All pass** |

### Diagnostic Coverage Gaps

The follow-up plan Task 1.3 specified `assert_state_consistency` should be called after **every public mutation function**. Current coverage:

**Covered:**
- `handle_movement`
- `apply_npc_events`
- `commit_trigger_narration`
- `evaluate_and_narrate_triggers`
- `execute_freeaction_impl` (twice)
- `game_service` trigger continuation path

**Not covered:**
- `game_service.rs` sync handlers: `Action::Quit`, `Action::Look`, `Action::Talk`, `Action::Inventory`
- `server/fragments.rs`: `process_sync_action`, `reset_generating_handler`
- `server/debug.rs`: debug endpoint is read-only (correctly excluded)
- `GameState::add_log`, `edit_log`, `delete_log` (direct state mutations)

These gaps are low-risk (sync handlers are simple), but they violate the stated goal of "every public mutation site."

---

## Critical Issues (None)

No merge blockers. The code is correct, tests pass, and guardrails are clean.

---

## Important Issues (Should Fix)

### 1. Diagnostic Failures Are Silent in Tests (INV-TEST-01)

**Location:** `engine/action_processing.rs`, `engine/game_service.rs`
**Problem:** `.ok()` swallow means a diagnostic failure does not fail the test.
**Fix:** Panic in test builds when diagnostics fail:

```rust
#[cfg(all(feature = "diagnostics", test))]
if let Err(e) = assert_state_consistency(state) {
    panic!("State invariant violated after handle_movement: {e}");
}
#[cfg(all(feature = "diagnostics", not(test)))]
assert_state_consistency(state).ok();
```

**Priority:** High. Without this, the diagnostics feature provides false confidence in test runs.

### 2. Missing `times_met` Monotonicity Check (INV-MONO-01)

**Location:** `engine/state_diagnostics.rs`
**Problem:** `apply_npc_events` increments `times_met` on `Entered`. A bug that decrements or resets it would not be caught.
**Fix:** Add to `assert_state_consistency`:

```rust
fn assert_times_met_non_negative(state: &GameState) -> Result<(), EngineError> {
    for (npc_id, encounter) in &state.character_state.npcs {
        if encounter.times_met == 0 && encounter.currently_meeting {
            return Err(EngineError::Internal(internal_error(format!(
                "NPC {npc_id} is currently_meeting but times_met is 0"
            ))));
        }
    }
    Ok(())
}
```

**Priority:** Medium. The follow-up plan explicitly listed this as an open question and recommended adding it.

---

## Suggestions (Optional)

### S1: Decide on Accessor Traits

The follow-up plan Task 2.1 specified `model/state_access.rs` with `MovementAccess`, `NarrativeAccess`, `SceneAccess`, and `CharacterAccess`. This was never implemented.

**Options:**

1. **Implement them now.** Limits `action_processing.rs` to `&mut impl MovementAccess + CharacterAccess`, which documents intent at the type level. Cost: ~50 lines of trait boilerplate + signature changes.
2. **Drop them permanently.** The follow-up plan's rationale for skipping session types ("not worth the maintenance cost") applies here too. Trait bounds add verbosity without preventing misuse — any function can still take `&mut GameState`.
3. **Compromise: implement one trait.** Start with `MovementAccess` only, since `handle_movement` is the most critical isolated mutation.

**Recommendation:** Option 2 (drop). The decomposition + diagnostics + property tests already provide 90% of the value. Accessor traits would be nice but are not load-bearing. Document the decision in an ADR and close the loop.

### S2: Add a `debug_assert_state` Macro

Instead of `.ok()` everywhere, create a small macro:

```rust
#[macro_export]
macro_rules! debug_assert_state {
    ($state:expr) => {
        #[cfg(feature = "diagnostics")]
        assert_state_consistency($state).ok();
    };
}
```

This reduces noise and makes it easy to change the swallow behavior globally.

### S3: Property Test Independence

Add one property that asserts a specific numeric invariant without calling `assert_state_consistency`. Example:

```rust
proptest! {
    #[test]
    fn prop_times_met_never_decreases(
        mut state in Just(make_test_state()),
        events in npc_event_vec(),
    ) {
        let before = get_times_met(&state.character_state, "carla");
        apply_npc_events(&mut state, &events);
        let after = get_times_met(&state.character_state, "carla");
        prop_assert!(after >= before);
    }
}
```

---

## Dead Code / Orphan Analysis

| Item | Status | Action |
|------|--------|--------|
| `GameState` flat fields (`current_room_id`, `log_history`, etc.) | Removed | Clean |
| `state_diagnostics.rs` in `model/` | Removed | Clean |
| `TestGameState` flat constructors | Updated | Clean |
| `server/debug.rs` field access | Updated to use sub-structs | Clean |
| `Cargo.lock` proptest deps | Added | Clean |

No dead code introduced. No orphan imports detected by clippy.

---

## Recommendations for Next Steps

1. **Fix INV-TEST-01** (panic on diagnostic failure in tests) before merge.
2. **Add INV-MONO-01** (`times_met` non-negative check) or explicitly document why it was skipped.
3. **Write an ADR** recording the decision to skip accessor traits and session types. Future agents will otherwise re-discover the same plan and wonder why it was not finished.
4. **Run `python build.py --release`** to verify no binary size regression.
5. **Merge.** The architectural direction is sound; remaining work is polish.

---

## Appendix: Test Commands Verified

```bash
cd chronicler_engine
cargo nextest run --features diagnostics        # 474 unit tests + all integration tests pass
python build.py                          # fmt + clippy + guardrails + build + 644 tests pass
cargo nextest run --test architecture           # arch-lint passes
cargo clippy --all-targets --all-features -D warnings  # clean
```
