# ADR-014: Action Pipeline Architecture

**Date:** 2026-05-19
**Status:** Accepted — partially superseded by ADR-027 (ActionPipelineBackend trait collapsed into direct fields; phase-based pipeline design remains in force)

## Context

Previously, the action execution flow (e.g., `execute_freeaction_pipeline`) was a monolithic function handling snapshot persistence, LLM calls, post-generation agent dispatch, engine triggers, cancellation, and error handling. Furthermore, the retry logic for events duplicated this exact trigger continuation flow without sharing the code, making the logic difficult to maintain and test. 

Additionally, the pipeline depended directly on the concrete `DefaultGameService` and its `llm_backend` and `agent_registry`, meaning any tests targeting the pipeline required spinning up the full service infrastructure (backends, registries, storage). The existing `GameService` trait only existed at the server boundary and did not act as a testable seam for the pipeline itself.

## Decision

We extracted an `ActionPipeline` module that explicitly models the game flow phases, unifying the normal action flow and retry flows, and introduced an `ActionPipelineBackend` trait to serve as a narrow, testable seam.

1. **Phase-Based Pipeline**: The pipeline explicitly models documented game flow phases as discrete steps (snapshot, narration, post-generation agent dispatch, engine commit, trigger evaluation, trigger continuation, reconciliation, finalization).
2. **Unified Action and Retry Flows**: Both normal play and retry flows (main narration retry, event continuation retry) utilize the same pipeline phase methods, eliminating duplicated logic. 
3. **Narrow Backend Trait**: The pipeline depends on a trait that exposes only three capabilities: narrate an action, complete a trigger continuation, and run post-generation agents. The concrete game service implements this trait.
4. **Concrete Implementation Adapter**: The game service implements the backend trait, owning the real backends and registry and wiring them to the trait interface.
5. **Strict Error Handling and Cancellation**:
   - **Early Errors** (before engine commit): Loads the latest state from storage, sets error status, and saves.
   - **Late Errors** (after engine commit): Uses the current in-memory state, sets error status, and saves.
   - **Cancellation**: Checked at stage boundaries (post-narration, pre-trigger, post-trigger), resetting to idle and saving.

## Consequences

### Positive

- **Testability**: Tests can now inject a narrow mock implementing only the backend trait (3 methods) instead of constructing a full game service with mock quantifiers and registries. The public API uses `DefaultGameService::execute_action()` and `DefaultGameService::retry_last_response()` wrapper methods, keeping internal implementation details private.
- **Locality and Readability**: Action orchestration lives in one module, avoiding monoliths.
- **Leverage**: Normal play, main retry, and event retry all use the exact same pipeline phases.
- **Architectural Clarity**: The dependency cycle between game service and pipeline was broken by extracting shared context into a separate module and making the pipeline depend on the backend trait.

### Negative

- **Slight Indirection**: Understanding the pipeline requires looking at both the phase logic and the `ActionPipelineBackend` trait implementation.

### Trade-offs

- Chose trait extraction over monolithic function (testability won over locality)
- Chose phase-based design over event-driven composition (simpler control flow; retry and normal paths share phases)
