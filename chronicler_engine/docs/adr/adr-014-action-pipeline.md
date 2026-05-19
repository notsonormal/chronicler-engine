# ADR-014: Action Pipeline Architecture

## Status

**Accepted** — Implemented during the 2026-05 refactor.

## Context

Previously, the action execution flow (e.g., `execute_freeaction_pipeline`) was a monolithic function handling snapshot persistence, LLM calls, post-generation agent dispatch, engine triggers, cancellation, and error handling. Furthermore, the retry logic for events duplicated this exact trigger continuation flow without sharing the code, making the logic difficult to maintain and test. 

Additionally, the pipeline depended directly on the concrete `DefaultGameService` and its `llm_backend` and `agent_registry`, meaning any tests targeting the pipeline required spinning up the full service infrastructure (backends, registries, storage). The existing `GameService` trait only existed at the server boundary and did not act as a testable seam for the pipeline itself.

## Decision

We extracted an `ActionPipeline` module that explicitly models the game flow phases, unifying the normal action flow and retry flows, and introduced an `ActionPipelineBackend` trait to serve as a narrow, testable seam.

1. **Phase-Based Pipeline**: `ActionPipeline` explicitly models the documented game flow phases via private phase methods (e.g., `phase_pre_main_snapshot`, `phase_narrate`, `phase_post_generation`, `phase_engine_commit`, `phase_trigger_build_request`, `phase_trigger_continuation`, `phase_post_trigger_reconcile`, and `phase_finalize`).
2. **Unified Action and Retry Flows**: Both normal play and retry flows (main narration retry, event continuation retry) utilize the same `ActionPipeline` phase methods, eliminating duplicated logic. 
3. **`ActionPipelineBackend` Trait**: Instead of depending on `DefaultGameService`, the pipeline depends on this narrow trait, which exposes only three capabilities: `narrate_action`, `complete` (for triggers), and `run_post_generation_agents`.
4. **Concrete Implementation Adapter**: `DefaultGameService` implements `ActionPipelineBackend`, serving as an adapter that owns the real backends and registry, and wires them to the trait interface.
5. **Strict Error Handling and Cancellation**:
   - **Early Errors** (before engine commit): Loads the latest state from storage, sets `Error` status, and saves.
   - **Late Errors** (after engine commit): Uses the current in-memory state, sets `Error` status, and saves.
   - **Cancellation**: Checks `cancel_token` at three specific boundaries (post-narration, pre-trigger, post-trigger), resetting to `Idle` and saving.

## Consequences

### Positive

- **Testability**: Tests can now inject a narrow mock implementing only `ActionPipelineBackend` (3 methods) instead of constructing a full `DefaultGameService` with mock quantifiers and registries.
- **Locality and Readability**: Action orchestration lives in one file (`action_pipeline/pipeline.rs`), avoiding monoliths.
- **Leverage**: Normal play, main retry, and event retry all use the exact same pipeline phases.
- **Architectural Clarity**: The dependency cycle between `game_service` and `action_pipeline` was broken by moving shared context to an `application/context.rs` module and making the pipeline depend on the `ActionPipelineBackend` trait.

### Negative

- **Slight Indirection**: Understanding the pipeline requires looking at both the phase logic and the `ActionPipelineBackend` trait implementation.
