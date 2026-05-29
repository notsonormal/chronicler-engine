# Plan: Refactor `application_service.rs` God Service and De-trait Application Layer

## Investigation Summary

Confirmed all issues raised in the review comments.

### 1. God Service
`chronicler_engine/src/application/application_service.rs` is **676 lines**. The `ApplicationService` trait defines **20 methods** covering:
- Action processing (`process_action`)
- Retries / retrigger (`retry`, `retrigger`)
- Game CRUD (`create_game`, `switch_game`, `delete_game`, `list_games`, `current_game_id`, `reset`)
- Message editing (`switch_swipe`, `edit_history`, `delete_last`)
- Queries / debug state (`get_generating_status`, `reset_generating_status`, `get_current_game_name`, `list_latest_llm_messages`, `get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state`)

All 20 methods are implemented on the single `DefaultApplicationService` struct in the same file.

### 2. Layer Violations & Misplaced Primitives
- **Layer violation**: `build_fresh_initial_state` (line 649) reaches directly into `crate::engine::logic::find_room_in_world_map` (line 659). The `application` tier should not depend on `engine::logic`.
- **Misplaced primitive**: `GenerationGuard` (lines 637-643), an RAII concurrency primitive, is defined at the bottom of the application service file. It belongs in a concurrency or context utility module.

### 3. Trait Census
| Trait | Real Implementors | Test Mocks | Action |
|---|---|---|---|
| `ApplicationService` | `DefaultApplicationService` (1) | 0 | Delete trait, use concrete struct |
| `GameService` | `DefaultGameService` (1) | 0 | Delete trait, use concrete struct |
| `PromptAssembler` | `LayeredPromptAssembler` (1) | 0 | Delete trait, use concrete struct |
| `ActionPipelineBackend` | `DefaultGameService` (1) | 2 (`MockPipelineBackend`, `MockBackend`) | Delete trait, use concrete struct |
| `Agent` | `QuantifierAgent`, `NarratorAgent` (2) | 1 (`MockAgent`) | Replace with enum |
| `LlmBackend` | `MockBackend`, `DeepSeekBackend` (stub), `OpenRouterBackend`, `OllamaBackend` (4) | Many (e.g. `HighConfidenceBackend`, `ErrBackend`, `RotatingBackend`) | Replace with enum |

All six traits are over-abstracted. The singleton traits add indirection (`Arc<dyn …>`, vtables) with no benefit. `Agent` and `LlmBackend` have known, closed sets of variants, making them ideal candidates for enums (static dispatch, no heap allocation, simpler matching).

## Goal
Split the God service by verb domain, fix the layer violation, and remove unnecessary trait indirection to improve compile-time checking, eliminate dynamic dispatch, and shrink file sizes.

## Detailed Steps

### Phase 1: Split `application_service.rs` by Verb Domain
Create three new modules under `src/application/`:
- **`game_lifecycle.rs`**: `create_game`, `switch_game`, `delete_game`, `list_games`, `current_game_id`, `reset`
- **`message_editing.rs`**: `switch_swipe`, `edit_history`, `delete_last`, `retry`, `retrigger`
- **`query_handlers.rs`**: `get_generating_status`, `reset_generating_status`, `get_current_game_name`, `list_latest_llm_messages`, `get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state`

Keep `application_service.rs` as the thin orchestrator: it retains the `DefaultApplicationService` struct definition, the `process_action` entry point, and any shared private helpers (`load_state`). Re-export the moved methods via `impl DefaultApplicationService` blocks in the submodules.

### Phase 2: Fix Layer Violations & Misplaced Primitives
1. **Move `GenerationGuard`** to a new `src/application/concurrency.rs` (or into `src/application/context.rs` if more appropriate). Update the single use site inside `process_action`.
2. **Move `build_fresh_initial_state`** out of the application tier. The function builds a fresh `GameState` from world data. It should live in `src/engine/bootstrap.rs` (or be absorbed into `src/bootstrap/scenario.rs`, which already legitimately calls `engine::logic`). Remove the `crate::engine::logic` import from `application_service.rs` entirely.

### Phase 3: Remove Singleton Traits
Proceed in dependency order to minimize breakage.

1. **Delete `ApplicationService` trait** (`src/application/application_service.rs`).
   - Convert all `Arc<dyn ApplicationService>` → `Arc<DefaultApplicationService>`.
   - Update `src/server/mod.rs` (HTTP server setup).
   - Update `src/test_support/test_app_builder.rs` (test harness).

2. **Delete `GameService` trait** (`src/application/game_service/service.rs`).
   - Convert all `Arc<dyn GameService>` → `Arc<DefaultGameService>`.
   - Update `DefaultApplicationService` field.
   - Update server and test builder.

3. **Delete `PromptAssembler` trait** (`src/narrative/prompt/assembler.rs`).
   - Convert `Arc<dyn PromptAssembler>` → `Arc<LayeredPromptAssembler>` inside `DefaultGameService`.
   - Update `ActionPipeline` (before it too is de-traited) to accept `&LayeredPromptAssembler`.
   - Remove the `&dyn PromptAssembler` return types.

4. **Delete `ActionPipelineBackend` trait** (`src/application/action_pipeline/pipeline.rs`).
   - The trait currently exists only to let tests mock the pipeline seam. Instead, make `ActionPipeline` take `&DefaultGameService` directly.
   - Update `execute_action_impl`, `retry_last_response_impl`, `retrigger_event_impl` to accept `&DefaultGameService`.
   - Refactor `pipeline_tests.rs` and `actions_tests.rs`: instead of mocking `ActionPipelineBackend`, construct a real `DefaultGameService` with a mock `LlmBackend` (enum variant, see Phase 5) injected via `DefaultGameService::with_backends`.

### Phase 4: Enum-ify `Agent`
1. Define `AgentEnum` in `src/narrative/agents/mod.rs`:
   ```rust
   pub enum AgentEnum {
       Quantifier(QuantifierAgent),
       Narrator(NarratorAgent),
       #[cfg(test)]
       Mock(MockAgent),
   }
   ```
2. Implement the former trait methods (`name`, `phase`, `backend_selector`, `execute`) as `impl AgentEnum`.
3. Update `AgentRegistry` to store `Vec<AgentEnum>` instead of `Vec<Box<dyn Agent>>`.
4. Refactor `registry_tests.rs`:
   - Replace `MockAgent` usage with `NarratorAgent` (it is a NoOp, so it serves the same purpose) or use `AgentEnum::Mock` behind `#[cfg(test)]`.

### Phase 5: Enum-ify `LlmBackend`
1. Define `LlmBackendEnum` in `src/narrative/llm/backend.rs`:
   ```rust
   pub enum LlmBackendEnum {
       Mock(MockBackend),
       DeepSeek(DeepSeekBackend),
       OpenRouter(OpenRouterBackend),
       Ollama(OllamaBackend),
       #[cfg(test)]
       TestMock(TestMockBackend), // if needed
   }
   ```
2. Implement the former `LlmBackend` methods directly on `LlmBackendEnum`.
3. Replace all occurrences:
   - `Box<dyn LlmBackend>` → `LlmBackendEnum`
   - `Arc<dyn LlmBackend>` → `Arc<LlmBackendEnum>`
   - `&dyn LlmBackend` → `&LlmBackendEnum`
4. Refactor test mocks in `quantifier/core_tests.rs` and elsewhere. Instead of ad-hoc structs implementing `LlmBackend`, create pre-configured `MockBackend` instances and wrap them in `LlmBackendEnum::Mock(…)`.

### Phase 6: Validation & Documentation
1. Run `cd chronicler_engine && python build.py` (fmt + clippy + tests + coverage).
2. Verify no behavioral regressions.
3. Update `docs/architecture/system.md` to reflect:
   - New application module layout (`game_lifecycle`, `message_editing`, `query_handlers`).
   - Removal of `GameService`, `ApplicationService`, `ActionPipelineBackend`, `PromptAssembler` traits.
   - Enum-based `Agent` and `LlmBackend` architecture.

## Risk Mitigation
- **No logic changes**: This is a pure structural refactor. Every method body should remain identical except for type signatures.
- **Incremental delivery**: Phases 1-3 can land independently of Phases 4-5 if preferred.
