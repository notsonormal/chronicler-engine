# Action Pipeline

## Objective

The action pipeline orchestrates the FreeAction lifecycle: pre-snapshot, narrate, post-generation agents, engine commit, trigger continuation, finalize. It unifies the normal action flow and the retry flows. Pipeline phases run synchronously inside a `spawn_blocking` task; the pipeline instance is constructed once at startup and shared by `Arc`.

## Phase Flow

```mermaid
flowchart TD
    Start([execute_action_impl called]) --> Pre[phase_pre_main_snapshot]
    Pre -->|status=Generating<br/>phase=Narrating| Narrate[phase_narrate]
    Narrate -->|cancel?| CC1((cancel checkpoint))
    CC1 -->|cancelled| HC1[handle_cancellation]
    CC1 -->|continue| EmptyCheck{empty response?}
    EmptyCheck -->|yes| ErrEmpty[error_return: empty]
    EmptyCheck -->|no| PostGen[phase_post_generation]
    PostGen -->|phase=Quantifying<br/>run agents| Commit[phase_engine_commit]
    Commit -->|Err| ErrEngine[status=Error, return Ok]
    Commit -->|Ok ActionResult| TriggerReq{build_trigger_request?}
    TriggerReq -->|Some| TriggerCont[phase_trigger_continuation_llm_call]
    TriggerReq -->|None| Finalize
    TriggerCont -->|cancel?| CC2((cancel checkpoint))
    CC2 -->|cancelled| HC2[handle_cancellation]
    CC2 -->|continue| TriggerEmpty{empty response?}
    TriggerEmpty -->|yes| ErrTrigger[status=Error, return Ok]
    TriggerEmpty -->|no| Reconcile[reconcile_post_trigger_npcs]
    Reconcile --> Finalize
    TriggerCont -->|all clear| Finalize[phase_finalize]
    Finalize -->|status=Idle<br/>phase=default| End([Ok])
    HC1 --> End
    HC2 --> End
    ErrEmpty --> End
    ErrEngine --> End
    ErrTrigger --> End
```

Cancel checkpoints (red diamonds): the pipeline α-checks `app.current_game_id()` against the started id at three boundaries (after the narrator call returns, at the start of trigger continuation, after the trigger LLM call returns). Each returns `Err(PhaseError::Cancelled)` on mismatch — see [Cancellation](#cancellation).

Error-return checkpoints (orange): `phase_narrate` for missing room / empty response / LLM error, `phase_trigger_continuation_llm_call` for trigger LLM error or empty response, `phase_engine_commit` for engine errors. Each phase returns `Err(PhaseError::...)`; the orchestrator's `finalize_phase_error` consumes each by setting `GenerationStatus::Error(msg)` + persisting + returning `Ok(())`.

## Error Model

The state field IS the error channel: `state.narrative.input_buffer.status` carries phase failures, and the UI polls it via `get_generating_status`. The pipeline `Result` only signals cancellation. `run_from_input` consumes every non-`Cancelled` variant via `finalize_phase_error`, which loads fresh state (the in-memory `state` was moved into the phase call), sets `GenerationStatus::Error(msg)` from the variant, runs `phase_finalize`, and returns `Ok(())` — so state is always persisted and the UI shows the error through the existing polling path.

`PhaseError` variants:

- **`PhaseError::Cancelled`** — the only `PhaseError` variant propagated to the caller; constructed by `handle_cancellation` (load_or_fresh → Idle → persist → `Err(Cancelled)`). All other variants are consumed by `finalize_phase_error`.
- **`PhaseError::NarratorFailed(String)`** — constructed by `error_return` (the narrator failure path of `phase_narrate` for missing room / empty response / LLM error). `error_return` preserves side effects: sets `GenerationStatus::Error(msg)`, persists state, then returns `Err(NarratorFailed(msg))`. The orchestrator's match arm consumes it via `finalize_phase_error`.
- **`PhaseError::PersistFailed { label, source }`** — constructed by `persist_snapshot_or_err` at four snapshot sites (`pre-main`, `pre-event`, `post-trigger`, `post-engine`) and at the `phase_engine_commit` call site in `run_from_input` (wrapped from `EngineError`). After ticket 03, all four sites are consumed by `finalize_phase_error` (set Error, persist, `Ok(())`) — previously the pre-event site `?`-propagated.
- **`PhaseError::TriggerMissing` / `PhaseError::SnapshotMissing`** — constructed by `retry_event_continuation`: `TriggerMissing` when `state.narrative.last_trigger` is absent on an event-retry; `SnapshotMissing` reserved for future snapshot-absent retry paths.
- **`PhaseError::FetchFailed(String)`** — constructed by `retry_event_continuation` when `ActionPipeline::load_world_bundle` fails on an event-retry precondition (game/world/persona lookup). Payload is `e.to_string()`, preserving canonical `EngineError` displays (e.g. `Game not found: 999`). Distinct from `NarratorFailed` because the failure occurred before the narrator LLM call — different recovery policy (precondition check vs mid-flow LLM failure).

`phase_finalize` resets `status` to `Idle` and `phase` to default UNLESS `status` is already `Error`, guaranteeing the UI never sees a stuck `Generating` after the pipeline returns. `finalize_phase_error` always sets `Error(...)` before calling `phase_finalize`, so the error persists. The retry path splits: **postcondition** failures returned from `retry_event_continuation` / `retry_main_narration` (any non-`Cancelled` `PhaseError`) propagate up to `retry_last_response_impl` / `retrigger_event_impl`'s outer `match` and are consumed by `ActionPipeline::finalize_phase_error` (same seam as `run_from_input`); **precondition** failures inside `retry_last_response_impl` before any phase runs (anchor missing, snapshot missing, load error, no input) persist `Error(...)` via the private `retry_persist_error` helper, which writes `Error` + `save_state` directly without `phase_finalize` (the heal path on the next action `heal_stale_generating` resets the status for this case).

## Cancellation

Two independent cancellation mechanisms:

- **In-phase α-check** (`PipelineRun::check_game_unchanged(started_for)`) — game-id mismatch, NOT a shutdown token. Three sites: after the narrator call returns (before `add_message`); at the start of trigger continuation (before persisting the pre-event snapshot); after the trigger LLM call returns (before commit). `phase_pre_main_snapshot` has no α-check. Reset / `switch_game` / `delete_game` may run while a generation is in flight; the α-check rejects stale results at these boundaries.
- **Pre-spawn shutdown gate** — `app.is_shutting_down()` is checked at the HTTP entry boundary only (retry/retrigger handlers in `application::message_editing`, process-action handler in `generation_gate`). It is never called inside a phase fn.

`handle_cancellation` is the single cleanup point for `Err(PhaseError::Cancelled)`: load fresh state, set `Idle`, clear `phase`, persist, return `Err(PhaseError::Cancelled)`. The retry orchestrators (`retry_last_response_impl`, `retrigger_event_impl`) match `Cancelled` inline at the bottom of their body (mirror of `handle_cancellation`'s load_or_fresh → Idle → persist) and route other `Err(PhaseError)` variants through `ActionPipeline::finalize_phase_error`.

`GenerationGuard::Drop` releases the registry slot only if the caller still owns it (no-op if superseded by a younger generation).

## Retry

Retry re-enters the pipeline without duplicating phase logic:

- **Main retry** (`retry_main_narration`) calls `pipeline.run_from_input` — the same path as a normal action. Soft-deletes messages after the anchor, re-runs narration + quantifier + trigger, preserves the old narration as a swipe.
- **Event retry** re-runs the trigger continuation phase against the restored snapshot state, then re-quantifies NPCs from the new continuation text. Regenerates only the continuation text using the stored `StoredTriggerContext` (carried by `state.narrative.last_trigger`); does not rerun main narration or quantification.

Anchor: `app.find_retry_anchor(&messages)` locates the message whose snapshot the retry restores. The old target message is moved to `state.narrative.retry_target` and re-appended to history after the engine-commit (and before the trigger continuation runs) via `state.narrative.retry_target.take()`.

## Document References

- [ADR-014: Action Pipeline Architecture](../adr/adr-014-action-pipeline.md) — original decision + borrow structure rationale
- [ADR-027: Hexagonal Architecture Migration](../adr/adr-027-hexagonal-architecture-migration.md) — pipeline lives in `application/`; ports/traits collapsed
- [ADR-032: PhaseError](../adr/adr-032-phaseerror.md) — error variant handling and retry cleanup duplication
- [system/game_flow.md](./game_flow.md) — `GenerationPhase` + `GenerationStatus` phase table
- [system/llm_processing.md](./llm_processing.md) — LLM recorder + agent registry contracts used by the pipeline
- [diagnostics/error_catalog.md](../diagnostics/error_catalog.md) — error variants the pipeline may surface (room-not-found, empty response, LLM transport failures)
- [architecture/rust_technical.md](../architecture/rust_technical.md) — `spawn_blocking` offload rationale (sync services, no async traits)
