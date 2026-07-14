# Action Pipeline

## Objective

The action pipeline orchestrates the FreeAction lifecycle: pre-snapshot, narrate, post-generation agents, engine commit, trigger continuation, finalize. It unifies the normal action flow and the retry flows. Pipeline phases run synchronously inside a `spawn_blocking` task; the pipeline instance is constructed once at startup and shared by `Arc`. See [architecture/rust_technical.md](../architecture/rust_technical.md) §Sync services + `spawn_blocking` offload for the runtime rationale.

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

Error-return checkpoints (orange): `phase_narrate` for missing room / empty response / LLM error, `phase_trigger_continuation_llm_call` for trigger LLM error or empty response, `phase_engine_commit` for engine errors. All set `status = GenerationStatus::Error(...)` and return `Ok(())`.

## Error Model

The state field IS the error channel: `state.narrative.input_buffer.status` carries phase failures, and the UI polls it via `get_generating_status`. The pipeline `Result` only signals cancellation. `run_from_input` finalizes and returns `Ok(())` whenever `status.error_message()` is set, so state is always persisted and the UI shows the error through the existing polling path.

Two error variants travel via `Result<_, PhaseError>`:

- **`PhaseError::Cancelled`** — the only `PhaseError` variant propagated to the caller; all other variants are absorbed because `status` already carries the error.
- **`PhaseError::PersistFailed { label, source }`** — constructed by `persist_snapshot_or_err` at four snapshot sites (`pre-main`, `pre-event`, `post-trigger`, `post-engine`); the helper writes `GenerationStatus::Error(...)` + persists state before returning the `Err`, so the UI surface stays consistent even when `run_from_input` returns `Err(...)` upstream. Only the pre-event site propagates the error to the caller; the other three call `persist_snapshot_or_err(...).is_err()` and swallow.

The remaining variants (`NarratorFailed(String)`, `TriggerMissing`, `SnapshotMissing`) are reserved — not constructed in `src/`.

`phase_finalize` resets `status` to `Idle` and `phase` to default UNLESS `status` is already `Error`, guaranteeing the UI never sees a stuck `Generating` after the pipeline returns. The retry path does not call `phase_finalize`: `save_retry_error` writes `Error(...)` and persists state directly; the heal path on the next action (`heal_stale_generating`) is what resets `status` for the retry case.

## Cancellation

Two independent cancellation mechanisms:

- **In-phase α-check** (`PipelineRun::check_game_unchanged(started_for)`) — game-id mismatch, NOT a shutdown token. Three sites: after the narrator call returns (before `add_message`); at the start of trigger continuation (before persisting the pre-event snapshot); after the trigger LLM call returns (before commit). `phase_pre_main_snapshot` has no α-check. Reset / `switch_game` / `delete_game` may run while a generation is in flight; the α-check rejects stale results at these boundaries.
- **Pre-spawn shutdown gate** — `app.is_shutting_down()` is checked at the HTTP entry boundary only (retry/retrigger handlers in `application::message_editing`, process-action handler in `generation_gate`). It is never called inside a phase fn.

`handle_cancellation` is the single cleanup point: load fresh state, set `Idle`, clear `phase`, persist, return `Err(PhaseError::Cancelled)`. The retry side also runs inline cleanup, duplicating the central path.

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
