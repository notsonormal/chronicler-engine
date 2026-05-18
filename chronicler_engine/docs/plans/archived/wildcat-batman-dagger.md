# Plan: Reorganize game_service Module + Extract ActionPipelineBackend Trait

## Problem

Two related architectural issues in `src/application/game_service/`:

1. **Hypothetical seam**: `ActionPipeline` takes `&DefaultGameService` concretely and reaches into `.llm_backend` directly. The `GameService` trait exists but only at the server boundary; the pipeline bypasses it entirely. Deleting `GameService` would have zero impact on the pipeline.

2. **Overloaded module**: `game_service/` mixes three concerns:
   - Service boundary (`GameService` trait, `DefaultGameService`)
   - Shared infrastructure (`GameServiceContext`, persistence helpers)
   - Action-processing workflows (`ActionPipeline`, `execute_action_impl`, retry logic)

A naive reorganization (moving pipeline to its own module) creates an architectural circularity: `action_pipeline → game_service` (for context/helpers) and `game_service → action_pipeline` (for trait implementation + entry functions).

## Goal

Restructure the modules so each has a single, clear responsibility, and introduce a real trait seam between `DefaultGameService` and `ActionPipeline`.

## What Problem Does ActionPipelineBackend Solve?

Currently `ActionPipeline` needs `&DefaultGameService`. To test the pipeline, you must construct a full `DefaultGameService` — which means providing an `llm_backend` (mock or real) **and** an `agent_registry` (with quantifier agents configured). The test surface is the entire service.

`ActionPipelineBackend` is a **narrow, purpose-built interface** that exposes only the three capabilities the pipeline actually uses:

1. `narrate_action` — call the LLM to narrate a player action
2. `complete` — call the LLM to generate a trigger continuation
3. `run_post_generation_agents` — run post-generation quantifier agents

With this trait:
- **Tests inject a mock** that implements only these 3 methods. No backends, no registries, no storage.
- **The pipeline depends on an abstraction**, not a concrete service struct.
- **`DefaultGameService` becomes an adapter** — it owns the real backends and registry, and wires them to the trait interface.

This is the "real seam with genuine leverage" described in the original suggestion. Without it, the `GameService` trait is a boundary decoration; with it, the pipeline has a testable contract.

### Alternative: Pass Dependencies Directly (No Trait)

Instead of a trait, `ActionPipeline` could take `&dyn LlmBackend` and `&AgentRegistry` directly:

```rust
pub struct ActionPipeline<'a> {
    llm_backend: &'a dyn LlmBackend,
    agent_registry: &'a AgentRegistry,
    ctx: &'a GameServiceContext,
}
```

Trade-offs:
- **Simpler** — no new trait, no glue `impl` block.
- **Less flexible for tests** — `AgentRegistry` is a concrete struct, not a trait. Tests must construct a real registry with mock agents inside it. You cannot easily stub the entire "run post-generation agents" behavior in one go.
- **More coupling** — the pipeline knows about two concrete concepts (`LlmBackend` trait + `AgentRegistry` struct) instead of one abstract concept.

The trait approach is recommended because it gives a single, mockable seam.

## New Module Layout

```
src/application/
  mod.rs
  context.rs              # GameServiceContext + persistence helpers (moved from game_service/)
  game_service.rs         # GameService trait + DefaultGameService + impl ActionPipelineBackend
  action_pipeline/
    mod.rs
    pipeline.rs           # ActionPipelineBackend trait + ActionPipeline<B>
    actions.rs            # execute_action_impl<B: ActionPipelineBackend>
    retry.rs              # retry_last_response_impl<B: ActionPipelineBackend>
    retry_tests.rs
```

## Why This Breaks the Cycle

- `action_pipeline/` imports `GameServiceContext` and helpers from `application::context`.
- `game_service.rs` imports `ActionPipelineBackend` and entry functions from `application::action_pipeline`.
- Dependency arrow is one-way: `action_pipeline → context ← game_service`. No cycle.

## Detailed Changes

### 1. Create `src/application/context.rs`

Move `GameServiceContext` (from `game_service/context.rs`) and all helpers (from `game_service/helpers.rs`) into this file. Update visibility as needed. The server currently imports `persist_new_messages` from `application::game_service`; after the move it will import from `application::context`.

### 2. Create `src/application/action_pipeline/mod.rs`

Declare the submodule and re-export public items:

```rust
pub mod pipeline;
pub mod actions;
pub mod retry;

pub use pipeline::{ActionPipeline, ActionPipelineBackend, ActionOutcome};
pub use actions::execute_action_impl;
pub use retry::retry_last_response_impl;
```

### 3. `src/application/action_pipeline/pipeline.rs`

Move `action_pipeline.rs` content here with these changes:

- Define the new trait:

```rust
pub trait ActionPipelineBackend: Send + Sync {
    fn narrate_action(
        &self,
        context: &PromptContext,
    ) -> Result<LlmCallResult, EngineError>;

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError>;

    fn run_post_generation_agents(
        &self,
        state: &GameState,
        player_input: &str,
        main_response: &str,
        result: &mut QuantifierResult,
    );
}
```

- Make `ActionPipeline<'a, B: ActionPipelineBackend>` generic over the trait.
- Replace all direct `self.service.llm_backend` access with trait method calls.
- Replace `run_post_generation_agents(service, ...)` with `self.service.run_post_generation_agents(...)`.
- Convert `reconcile_post_trigger_npcs` into a private method on `ActionPipeline` (it is only called from `ActionPipeline` methods).
- Keep `default_quantifier_result` and `build_trigger_request` as free functions in this module.

### 4. `src/application/action_pipeline/actions.rs`

Move `actions.rs` content here. Change the signature:

```rust
pub fn execute_action_impl<B: ActionPipelineBackend>(
    backend: &B,
    ctx: GameServiceContext,
    input: String,
    _player_name: String,
) {
    let mut state = load_state(&ctx);
    state.narrative.last_trigger = None;
    let pipeline = ActionPipeline::new(backend, &ctx);
    // ... rest unchanged
}
```

### 5. `src/application/action_pipeline/retry.rs`

Move `retry.rs` content here. Make functions generic:

```rust
pub fn retry_last_response_impl<B: ActionPipelineBackend>(backend: &B, ctx: GameServiceContext) { ... }
pub(crate) fn retry_event_continuation<B: ActionPipelineBackend>(backend: &B, ctx: &GameServiceContext, state: GameState) { ... }
pub(crate) fn retry_main_narration<B: ActionPipelineBackend>(backend: &B, ctx: &GameServiceContext, state: GameState, input_text: String) { ... }
```

### 6. `src/application/game_service.rs`

Consolidate what is currently `game_service/mod.rs` + `game_service/service.rs` into a single file (or keep `mod.rs` + `service.rs` if preferred). Implement the trait glue:

```rust
use crate::application::action_pipeline::ActionPipelineBackend;
use crate::narrative::llm::backend::AGENT_NARRATOR;

impl ActionPipelineBackend for DefaultGameService {
    fn narrate_action(&self, context: &PromptContext) -> Result<LlmCallResult, EngineError> {
        self.llm_backend.narrate_action(AGENT_NARRATOR, context)
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        self.llm_backend.complete(agent_name, system_prompt, user_prompt, max_tokens)
    }

    fn run_post_generation_agents(
        &self,
        state: &GameState,
        player_input: &str,
        main_response: &str,
        result: &mut QuantifierResult,
    ) {
        // body moved from current free function
        let agent_ctx = AgentContext { ... };
        for agent in self.agent_registry.agents_for_phase(ExecutionPhase::PostGeneration) {
            // ...
        }
    }
}
```

Update `GameService` impl to call generic entry functions:

```rust
impl GameService for DefaultGameService {
    fn execute_action(&self, ctx: GameServiceContext, input: String, player_name: String) {
        crate::application::action_pipeline::execute_action_impl(self, ctx, input, player_name);
    }

    fn retry_last_response(&self, ctx: GameServiceContext) {
        crate::application::action_pipeline::retry_last_response_impl(self, ctx);
    }
}
```

### 7. `src/application/mod.rs`

Update to declare the new modules:

```rust
pub mod action_pipeline;
pub mod context;
pub mod game_service;
```

### 8. Server imports

Update `src/server/fragments/actions.rs` (and any other server files) that import `persist_new_messages` from `application::game_service` to import from `application::context` instead.

## Test Impact

- **Zero breaking changes** to existing tests. `DefaultGameService` still implements `GameService`, and all existing constructors (`new`, `with_storage`, `with_backends`, `with_mock_quantifier`) remain unchanged.
- **New capability**: Tests can now inject a narrow mock implementing only `ActionPipelineBackend` (~3 methods) instead of constructing a full `DefaultGameService`. This shrinks the test surface for pipeline-focused tests.
- `helpers_tests.rs` moves with `context.rs` and continues to test the same functions.
- `retry_tests.rs` moves into `action_pipeline/`.

## Validation Steps

1. `cd chronicler_engine && cargo check` — compiles after module moves
2. `cd chronicler_engine && cargo test` — all existing tests pass
3. `cd chronicler_engine && python build.py` — full validation (fmt, clippy, tests, coverage)

## Rollback

If issues arise, revert file moves and restore `game_service/mod.rs` with all submodules. The changes are localized to `src/application/`.
