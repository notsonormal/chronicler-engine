# Plan: Break Engine↔Narrative Bidirectional Coupling

## Problem
The engine tier (`action_processing.rs`) imports narrative types (`QuantifierResult`, `PromptBuilder`, `PromptContext`, `LlmBackend`). The narrative tier (`quantifier/core.rs`) calls `crate::engine::logic::get_current_room(state)`. This forms a conceptual dependency cycle that prevents isolating the narrative/agent tier from spatial engine rules.

## Success Criteria
- `narrative/` contains zero imports from `engine/`
- `engine/` contains zero imports from `narrative/`
- `arch-lint.toml` enforces `engine → narrative` denial
- All 631+ tests pass after each phase
- `cargo clippy --all-targets --all-features -D warnings` is clean

## Approach: Four-Phase Decoupling

Each phase is independently committable and testable.

### Phase 1 — Break narrative→engine (spatial query)
**Goal:** `narrative/agents/quantifier/core.rs` no longer calls `engine::logic::get_current_room`.

1. **Move `get_current_room` to `model`**  
   It is a pure state query (`state.map.get_room_by_id(...) || state.movement.dynamic_rooms.get(...)`). It belongs in the innermost tier.  
   - Add `pub fn current_room(&self) -> Result<&Room>` as a method on `GameState` in `model::state`, or add a free function `pub fn get_current_room(state: &GameState) -> Result<&Room>` in `model::state`.
   - Remove `pub fn get_current_room` from `engine::logic`.
   - Update call sites: `engine/action_processing.rs`, `engine/logic_tests.rs`, `engine/state_diagnostics.rs`, `server/fragments/renderers.rs`.

2. **Add `current_room` to `AgentContext`**  
   `model::agent::AgentContext` carries the ambient room for agents that need spatial context.
   - Add `pub current_room: Option<&'a crate::model::map::Room>`.
   - Update all construction sites: `application/game_service/actions.rs` (in `run_post_generation_agents`), test files in `narrative/agents/`.

3. **Pass `&Room` into `determine_npcs_in_room`**  
   - Change signature to accept `current_room: &Room`.
   - In `narrative/agents/quantifier/agent.rs`, pass `ctx.current_room` (unwrap or fallback).
   - In `narrative/agents/quantifier/core.rs`, remove the `crate::engine::logic::get_current_room` call; use the passed `current_room` directly.
   - Update `core_tests.rs` to provide the room explicitly.

**Verify:** `cargo nextest run --test architecture` + `cargo nextest run quantifier` pass.

### Phase 2 — Break engine→narrative (mechanical result types)
**Goal:** `engine/` no longer imports quantifier data types from `narrative/`.

1. **Create `model::quantifier` module** (or `model::scene` if preferred).  
   Move the following from `narrative::agents::quantifier` to `model`:
   - `QuantifierResult`, `QuantifierParseResult`, `MovementParseResult`, `QuantifierConfidence`
   - `NpcEvent`, `NpcEventType`, `NpcEventList`
   - `compute_npc_events` (pure mechanical diff of two ID lists)

2. **Update imports across the codebase**
   - `engine/action_processing.rs` — import from `model::quantifier`
   - `engine/action_processing_tests.rs` — import from `model::quantifier`
   - `application/game_service/actions.rs` — import from `model::quantifier`
   - `narrative/agents/quantifier/mod.rs` — re-export from `model::quantifier` for backward compatibility inside narrative
   - `narrative/agents/quantifier/types.rs` — remove moved types (or re-export)
   - `narrative/agents/quantifier/parser.rs` — remove `compute_npc_events` (or re-export from model)

**Verify:** `cargo nextest run action_processing` + `cargo nextest run quantifier` pass.

### Phase 3 — Break engine→narrative (prompt/LLM types)
**Goal:** `engine/` no longer imports `PromptBuilder`, `PromptContext`, or `LlmBackend`.

1. **Extract trigger prompt building from `engine` to `application`**  
   `build_trigger_prompt_parts`, `build_trigger_request`, and `evaluate_and_narrate_triggers` are orchestration concerns that construct LLM prompts and make LLM calls. They belong in the application tier (which already coordinates LLM generation and game flow).
   - Move `build_trigger_prompt_parts` and `build_trigger_request` to `application/game_service/trigger_continuation.rs` (or inline into `actions.rs` if small enough).
   - Delete `evaluate_and_narrate_triggers` from `engine/action_processing.rs` — it is currently unused in production.

2. **Simplify `FreeActionContext`**  
   Remove fields that are only needed for trigger prompt building:
   - `llm_backend`
   - `response_length`
   - `max_context_tokens`
   - `max_tokens`
   - `user_input` (verify if still needed; if only for trigger building, remove)
   - `world`, `player`, `all_npcs`, `history` (verify if still needed; if only for trigger building, remove)

   Keep only what `execute_freeaction_impl` truly needs for state transitions:
   - `narration_text`
   - `quantifier_result` (now imported from `model`)

3. **Change `TurnResult` and `execute_freeaction_impl`**  
   - Remove `trigger_continuation: Option<TriggerContinuationRequest>` from `TurnResult`.
   - `execute_freeaction_impl` returns only `next_state` and `narration`.
   - Application layer calls `engine::trigger_eval::evaluate_triggers(&next_state)` directly after `execute_freeaction_impl`, then builds the continuation request using the moved `build_trigger_request`.

4. **Update tests**
   - `engine/action_processing_tests.rs`: Remove tests for trigger prompt building (move them to `application/` tests if they exist). Update `FreeActionContext` constructions to use the simplified struct.
   - `application/game_service/retry_tests.rs` and `actions.rs`: Update to build trigger requests in application.

**Verify:** `cargo nextest run action_processing` + `cargo nextest run application` pass.

### Phase 4 — Guardrails
1. **Add `engine → narrative` denial rule in `arch-lint.toml`**
   ```toml
   [[deny-scope-dep]]
   from = "engine"
   to = ["narrative"]
   message = "Engine layer must not depend on narrative layer."
   ```

2. **Run full validation:** `cd chronicler_engine && python build.py`

## Files Touched (estimated)
- `model/agent.rs` — add `current_room` to `AgentContext`
- `model/state.rs` — add `current_room()` method (or new `model::state_query.rs`)
- `model/quantifier.rs` — new module for moved types
- `engine/logic.rs` — remove `get_current_room`
- `engine/action_processing.rs` — remove narrative imports, simplify `FreeActionContext`/`TurnResult`
- `engine/action_processing_tests.rs` — update tests
- `engine/state_diagnostics.rs` — update `get_current_room` call
- `engine/logic_tests.rs` — update `get_current_room` calls
- `narrative/agents/quantifier/core.rs` — accept `&Room` parameter
- `narrative/agents/quantifier/agent.rs` — pass room from `AgentContext`
- `narrative/agents/quantifier/types.rs` — re-export moved types
- `narrative/agents/quantifier/parser.rs` — re-export `compute_npc_events`
- `narrative/agents/quantifier/core_tests.rs` — update tests
- `narrative/agents/quantifier/agent_tests.rs` — update `AgentContext` constructions
- `application/game_service/actions.rs` — populate `AgentContext.current_room`, move trigger building
- `application/game_service/retry.rs` — update imports if needed
- `server/fragments/renderers.rs` — update `get_current_room` call
- `arch-lint.toml` — add `engine → narrative` rule

## Notes & Trade-offs
- **Why not just pass `&Room` everywhere?** Moving `get_current_room` to `model` is cleaner because engine, server, and application all need it; model is the common inner tier.
- **Why move `QuantifierResult` to `model`?** It describes a mechanical state transition (which NPCs are present, where the player moved), not an LLM-specific artifact.
- **Why move trigger building to `application`?** `application` already orchestrates LLM calls (`narrate_action`, `complete`). The engine tier should be pure state-transition logic.
- **Test impact:** `action_processing_tests.rs` will need the most updates because it constructs `FreeActionContext` extensively. The changes are mechanical (remove unused fields).
