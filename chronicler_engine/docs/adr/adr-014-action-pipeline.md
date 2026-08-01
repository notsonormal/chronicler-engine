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
6. **Borrow Structure** (added on update; survives the ADR-027 trait collapse): phase logic hangs off `PipelineRun<'a>`, a per-call borrow pair of `&ActionPipeline` + `&DefaultApplicationService`. `DefaultApplicationService` is held by `Arc`, is expensive to clone, and must outlive every phase call; threading `app: &DefaultApplicationService` through each phase signature would duplicate the borrow. `PipelineRun::new(self, app)` is constructed once per call, after which phase signatures take only `&self` (the run). Inputs that phases read across boundaries (`world`, `map`, `persona`, `all_npcs`) live in `PipelineInputs` as owned `Arc<...>` / `Vec<...>` rather than borrowing from `GameState`, because `GameState` is mutated across phase boundaries and a stable snapshot is needed while state evolves. The `'a` lifetime ties the run to its borrows; callers (`spawn_blocking` closure, retry continuation) hold the borrowed values for the duration.

## Consequences

### Positive

- **Testability**: Tests can now inject a narrow mock implementing only the backend trait (3 methods) instead of constructing a full game service with mock quantifiers and registries. The public API uses `DefaultGameService::execute_action()` and `DefaultGameService::retry_last_response()` wrapper methods, keeping internal implementation details private.
- **Locality and Readability**: Action orchestration lives in one module, avoiding monoliths.
- **Leverage**: Normal play, main retry, and event retry all use the exact same pipeline phases.
- **Architectural Clarity**: The dependency cycle between game service and pipeline was broken by extracting shared context into a separate module and making the pipeline depend on the backend trait.

### Negative

- **Slight Indirection**: Understanding the pipeline requires looking at both the phase logic and the `ActionPipelineBackend` trait implementation. (After ADR-027, this becomes “understand the phase logic and the direct fields on `ActionPipeline`”.)
- **Borrow-Structure Reading Cost**: phase signatures reference `PipelineRun<'a>` rather than raw parameters; a reader must know the borrow pair exists. Offset by all phase logic living in `phases.rs`.

### Trade-offs

- Chose trait extraction over monolithic function (testability won over locality; partially reversed in ADR-027)
- Chose phase-based design over event-driven composition (simpler control flow; retry and normal paths share phases)
- Chose `PipelineRun<'a>` borrow pair over passing `app`/`pipeline` through each phase (avoids borrow duplication; cost is one indirection layer)

## History

- **2026-08-01**: Facade-deletion work completed (`DefaultApplicationService` and `src/application/application_service.rs` removed). The `PipelineRun<'a>` borrow pair now borrows `&ActionPipeline` only; the former `&DefaultApplicationService` half is gone because orchestration methods moved onto `ActionPipeline` and the remaining collaborators (`GameCatalogue`, `GameViewQuery`, `GenerationGate`, `PersistenceGate`) are accessed directly from `AppState` by HTTP handlers. The body of this ADR records the decision at the time it was made; see current source and `docs/diataxis/reference/game_flow.md` for the live shape.
