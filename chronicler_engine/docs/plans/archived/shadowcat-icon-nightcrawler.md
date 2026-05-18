# Plan: Extract ActionPipelineBackend Trait

## Problem

`ActionPipeline` in `src/application/game_service/action_pipeline.rs` takes `&DefaultGameService` concretely and accesses `.llm_backend` directly. The `GameService` trait exists but is only implemented by `DefaultGameService` and is never used by the pipeline. Deleting `GameService` would have zero impact on the pipeline — it is a hypothetical seam with no leverage.

This forces tests to construct a full `DefaultGameService` (via `with_mock_quantifier`) even when they only need to mock LLM narration and post-generation quantification.

## Goal

Introduce a smaller, purpose-built trait that `ActionPipeline` depends on, turning the dependency into a real seam. Move the concrete `DefaultGameService` glue out of the pipeline module.

## Files to Modify

1. `src/application/game_service/action_pipeline.rs` — Define trait, make `ActionPipeline` generic over it, refactor helper functions
2. `src/application/game_service/actions.rs` — Implement trait for `DefaultGameService`, update `execute_action_impl` call site
3. `src/application/game_service/retry.rs` — Update call sites to `ActionPipeline::new`
4. `src/application/game_service/service.rs` — Keep `GameService` trait as-is (server layer still uses it), no changes required

## Design

### New Trait: `ActionPipelineBackend`

Defined in `action_pipeline.rs` (or a new `pipeline_backend.rs` if preferred). It exposes exactly the three capabilities the pipeline uses:

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

Notes:
- `narrate_action` omits the `agent_name` parameter from the raw `LlmBackend` method; the pipeline always passes `AGENT_NARRATOR`, so the implementation can hard-code it.
- `complete` keeps `agent_name` because both `AGENT_TRIGGER` and potentially others are used.
- `run_post_generation_agents` absorbs the current free function's logic into a trait method, removing the `&DefaultGameService` parameter.

### Pipeline Changes

- `ActionPipeline<'a, B: ActionPipelineBackend>` stores `&'a B` instead of `&'a DefaultGameService`.
- Remove direct access to `self.service.llm_backend` inside pipeline phases; call trait methods instead.
- Remove `run_post_generation_agents` and `reconcile_post_trigger_npcs` free functions from `action_pipeline.rs` (or keep `reconcile_post_trigger_npcs` as a pure function taking `&dyn ActionPipelineBackend`).

### Glue Implementation

In `actions.rs`, implement `ActionPipelineBackend` for `DefaultGameService`:

```rust
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
    }
}
```

### Call Site Updates

- `actions.rs`: `ActionPipeline::new(service, &ctx)` still works because `DefaultGameService` now implements `ActionPipelineBackend`.
- `retry.rs`: Same, no signature changes needed at call sites.

### Test Impact

- Existing tests that construct `DefaultGameService::with_mock_quantifier(...)` continue to work unchanged because `DefaultGameService` implements the new trait.
- **New capability**: Tests can now inject a narrow mock implementing only `ActionPipelineBackend` (~3 methods) instead of constructing a full `DefaultGameService`. This shrinks the test surface and removes the need for `with_mock_quantifier` in pipeline-focused tests.
- No breaking changes to existing test code.

## Validation Steps

1. `cd chronicler_engine && cargo check` — compiles
2. `cd chronicler_engine && cargo test` — all existing tests pass
3. `cd chronicler_engine && python build.py` — full validation (fmt, clippy, tests, coverage)

## Rollback

If issues arise, revert the trait extraction and restore `&DefaultGameService` in `ActionPipeline`. The change is local to the `application/game_service` module.
