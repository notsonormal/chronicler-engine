# Plan: Raise `game_service.rs` Coverage to 80%+ via Refactoring

## Problem

`game_service.rs` has 62.2% line coverage (135/217 lines). The inner thread bodies of `FreeAction` (lines 109-212) and `retry_last_response` (lines 260-310) are opaque to unit tests because the thread spawns before any test can observe the code. Error/edge paths (lock poisoning, room-not-found, LLM failure) are never triggered in tests.

## Solution

Extract `execute_freeaction_impl` — the entire FreeAction inner thread body — as a **synchronous, pure-ish function** in `action_processing.rs`. The function receives `QuantifierResult` as a parameter so it can be tested without any LLM calls or thread spawns.

## Design Decisions

1. **Pass `QuantifierResult` as parameter** — not trait-based. Makes `execute_freeaction_impl` fully testable without LLM or mock backend.
2. **Extract `execute_freeaction_impl` as synchronous function** — contains the entire FreeAction inner thread body but runs on the caller's thread. Allows full-flow unit testing without threading.
3. **`retry_last_response` stays as-is** — simpler and already tested through integration tests.

## Files to Change

| File | Change |
|------|--------|
| `src/engine/action_processing.rs` | Add `execute_freeaction_impl` function + unit tests |
| `src/engine/game_service.rs` | Refactor `FreeAction` branch to use `execute_freeaction_impl` |
| `tests/game_service_tests.rs` | Add error-path unit tests for `game_service.rs` |
| `docs/CHANGELOG.md` | Record coverage improvement |

## Step 1: Add `execute_freeaction_impl` to `action_processing.rs`

New public function signature:

```rust
pub fn execute_freeaction_impl(
    state: &mut GameState,
    text: &str,
    quantifier_result: &crate::narrative::quantifier::QuantifierResult,
    world: &WorldCard,
    map: &MapDef,
    player: &PlayerCard,
    all_npcs: &[NpcCard],
    room_npc_ids: &[String],
    history: &[LogEntry],
) -> Result<(), EngineError>
```

### Logic extracted from `game_service.rs` lines 146-206:

1. Get previous room NPCs from `state.npcs_in_area` (lines 150-152)
2. Call `handle_movement` with quantifier movement result (lines 168-172)
3. Build `current_npcs` from `quantifier_result.npcs.npc_ids` (lines 174-179)
4. `state.add_log(narration_text, None, LogType::Narration)` (line 192)
5. `state.npcs_in_area = current_npcs` (line 193)
6. Call `evaluate_and_narrate_triggers` (lines 197-202)
7. Call `compute_npc_events` then `apply_npc_events` (lines 205-206)

### Note on `is_generating`
`is_generating` reset is **not** handled inside `execute_freeaction_impl` — it remains in the thread spawn code in `game_service.rs`. This keeps `execute_freeaction_impl` cleanly synchronous.

### Unit tests to add:

| Test | Scenario |
|------|----------|
| `test_execute_freeaction_impl_no_movement` | Quantifier returns no movement, verify narration logged |
| `test_execute_freeaction_impl_with_movement` | Quantifier returns destination, verify room change and log entry |
| `test_execute_freeaction_impl_empty_narration` | Handle empty narration gracefully |
| `test_execute_freeaction_impl_updates_npcs_in_area` | Verify `state.npcs_in_area` populated correctly |
| `test_execute_freeaction_impl_triggers_evaluated` | NPC with `TimesMet Eq 0` trigger fires on first encounter |
| `test_execute_freeaction_impl_npc_events_entered` | Verify `Entered` events increment `times_met` and set `currently_meeting` |
| `test_execute_freeaction_impl_npc_events_left` | Verify `Left` events set `currently_meeting = false` |

## Step 2: Refactor `game_service.rs`

Refactor `Action::FreeAction` branch to:
1. Call `determine_npcs_in_room` (the only LLM-dependent part) in the thread
2. Pass the result to `execute_freeaction_impl` (synchronous, testable)
3. Handle `is_generating` reset after `execute_freeaction_impl` returns

```rust
Action::FreeAction(text) => {
    // ... clone data for thread (unchanged) ...
    drop(state_guard);

    let state_for_thread = state.clone();
    thread::spawn(move || {
        // LLM call + quantifier pass (stays in thread)
        let backend = get_llm_backend();
        // ... context building, narrate_action call ...

        let quantifier_result = determine_npcs_in_room(...);

        // Synchronous, fully testable — no thread needed
        let result = execute_freeaction_impl(
            &mut *state_for_thread.lock().unwrap(),
            &text,
            &quantifier_result,
            &world, &map, &player,
            &all_npcs, &room_npc_ids,
            &history,
        );

        if result.is_err() {
            // set error message
        }
        state_for_thread.lock().unwrap().generation_state.is_generating = false;
    });
}
```

## Step 3: Add Error-Path Tests to `game_service_tests.rs`

| Test | Target |
|------|--------|
| `test_execute_action_freeaction_early_return_on_invalid_room` | Room not found path → `is_generating = false` |
| `test_execute_action_freeaction_llm_error_sets_error_message` | LLM failure → error message set |
| `test_execute_quit_guards_against_poisoned_lock` | Lock poisoning at line 52-55 |
| `test_execute_look_guards_against_poisoned_lock` | Lock poisoning for `Look` action |

## Verification

```bash
cd chronicler_engine && python build.py
```

Expected:
- `game_service.rs`: 80%+
- `action_processing.rs`: 90%+

## Status

- [x] Plan created
- [ ] Architecture updated
- [ ] `execute_freeaction_impl` implemented
- [ ] Unit tests added
- [ ] `game_service.rs` refactored
- [ ] Error-path tests added
- [ ] Build passes, coverage verified