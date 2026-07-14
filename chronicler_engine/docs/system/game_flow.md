# Specification: Game Flow

**Scope:** This document specifies the **runtime control flow** — the phase sequence from player input through LLM generation, quantification, and trigger evaluation back to UI update.

## The Game Flow

```mermaid
flowchart TD
    Start["Start Game"]
    Init["1. Initialize\nLoad world, set player, render UI, start polling"]
    Await["2. Await Input\nStatus: Ready"]
    Process["3. Process Action\nParse, validate, log, spawn LLM task"]
    Narrate["Main Narration\nBuild prompt, LLM, Save to DB"]
    Quantify["Quantifier\nDetect movement and NPC triggers"]
    TriggerEval["Trigger Evaluation\nIf match: continuation narration"]
    Quantify2["Post-Event Quantifier\nUpdate NPC presence"]
    Poll["Polling Update\nClient refreshes via HTMX"]

    Start --> Init --> Await --> Process --> Narrate --> Quantify --> TriggerEval
    TriggerEval --> Quantify2 --> Poll
    Poll -.-> Await
```

Main narration builds a comprehensive prompt using the layered prompt system.

### Granular Status Phases

During LLM processing, the UI displays granular status phases instead of a single "Thinking..." message:

| Phase | Display Text | Endpoint Value | When Active |
|-------|-------------|----------------|-------------|
| `Narrating` | "Generating narration..." | `narrating` | During main LLM narration — narration saved to DB before quantifier runs |
| `Quantifying` | "Quantifying scene..." | `quantifying` | During post-narration quantifier analysis — narration visible, metadata pending |
| `GeneratingEvent` | "Generating event..." | `generating-event` | During trigger continuation narration — fires after trigger evaluation |

- The persisted status field (`Idle` / `Generating` / `Error`) drives UI disable state for the polled render — see **Two State Channels** below for how it relates to the spawn-side concurrency gate
- The frontend maps endpoint values (`narrating`, `quantifying`, `generating-event`) to human-readable text
- An optimistic "Thinking..." is shown immediately on form submit before the first poll response

### Two State Channels

Generation state uses two complementary signals:

- **Persisted status field** — survives panics and process restarts via DB snapshots; drives UI display through the polled status fragment.
- **Process-local atomic flag** — cleared on panic by an RAII guard; gates spawn-side concurrency to prevent double-spawn within a single process.

The status field must persist across crashes so the UI can recover and show correct state. The atomic flag is a fast lock for handlers that race with in-flight generation; it has no role in UI display.

Self-healing stale-state recovery (see Error Model below) only inspects the status field; the atomic flag is always `false` after a successful process exit.


### Retry Flow

Retrying a response (via the right swipe arrow on the last message) branches based on whether the last AI-generated content was a **main narration** or a **trigger event continuation**:

```mermaid
flowchart TD
    Start["User clicks retry"]
    Check{"Event response?"}
    Main["Main Retry Path"]
    Event["Event Retry Path"]
    MainAnchor["Find anchor message\nLoad snapshot"]
    MainDel["Soft-delete messages after anchor"]
    MainPreserve["Preserve old as swipe"]
    MainRestore["Restore snapshot state"]
    ReRunMain["Full Regeneration\nNarration + Quantifier + Trigger"]
    EventAnchor["Find anchor message\nLoad snapshot"]
    EventDel["Soft-delete event messages"]
    EventPreserve["Preserve old event as swipe"]
    EventRestore["Restore snapshot state"]
    ReRunEvent["Trigger Continuation Only"]
    Update["Update UI"]

    Start --> Check
    Check -->|No| Main
    Check -->|Yes| Event
    Main --> MainAnchor --> MainDel --> MainPreserve --> MainRestore --> ReRunMain --> Update
    Event --> EventAnchor --> EventDel --> EventPreserve --> EventRestore --> ReRunEvent --> Update
```

**Retry semantics**:
- **Main retry** soft-deletes messages after the anchor input and re-runs phases 4 → 4.5 → 5 → 5.5 (full regeneration). Old narration preserved as a swipe. On success, prior swipes migrate to the new message and soft-deletes are purged; on failure, soft-deletes are restored.
- **Event retry** regenerates only the continuation text from the anchor snapshot using stored `StoredTriggerContext` prompts — does not rerun main narration or quantification.
- **Retrigger Event** (on a narration swipe with `last_trigger` and no following event messages) runs `ActionPipeline::phase_trigger_continuation()` → `reconcile_post_trigger_npcs()` → `phase_finalize()` from the restored snapshot state without rerunning the main narration.
- Snapshots are standalone — no `base_snapshot_id` chain. Each message and each `Swipe` carries its own `snapshot_id` of the state captured after it was created; switching swipes restores that exact state.
- If the anchor message has no `snapshot_id` or the snapshot is missing, retry fails gracefully.

### Error Model

Pipeline errors (LLM failures, empty responses, room-not-found, etc.) follow a unified pattern:
1. Set `state.narrative.input_buffer.status = GenerationStatus::Error(message)` on the current `GameState`
2. Persist via `save_state()` (or `save_message_and_snapshot()` if a system message was added)
3. Return `Ok(state)` / `Ok(())` — phase failure is signalled via `state.narrative.input_buffer.status`, not via the `Err` path of the pipeline return type

The caller checks `state.narrative.input_buffer.status.error_message()` to decide whether to continue to later phases or skip straight to `phase_finalize`. This ensures:
- State is always persisted (never lost to an `Err` that skips `save_state`)
- The UI shows the error via the existing `GenerationStatus::Error` polling path
- `phase_finalize` always runs; the `is_generating` projection is cleared by `GenerationGuard::Drop` on the registry path (only if no other game's slot is still `Generating`)

**Cancellation** produces `Err(PhaseError::Cancelled)`. `PhaseError::PersistFailed` also propagates via `Err` from snapshot sites, but only `Err(PhaseError::Cancelled)` is matched by callers — other variants rely on `GenerationStatus::Error` already written to state. `PhaseError` is errors-only; success is `Ok(())`.

**Stale-Generating recovery**: If `is_generating` is `false` but persisted status is still `Generating` (e.g., after a panic), `process_action` resets status to `Idle` before proceeding. The per-game registry is also self-healed on the same path: a `Generating` slot for `current_game_id()` whose projection atomic is false is cleared to `Idle`. `GenerationGuard::Drop` releases the registry slot if it still owns it (no-op if superseded by a younger generation) and stores the projection atomic to `false` only when no other game's slot is generating.

## Document References

- [ADR-006: Quantifier-Driven Game Systems](../adr/adr-006-quantifier-systems.md) — quantifier detects NPCs + movement after narration
- [ADR-008: SQLite Snapshot Persistence](../adr/adr-008-sqlite-snapshot-persistence.md) — `GameStateSnapshot` table + `persist_snapshot_or_err` pattern
- [ADR-010: Concurrency and Generation Gate Model](../adr/adr-010-concurrency-generation-gate.md) — `is_generating` dual-source invariant + `GenerationGuard`
- [ADR-017: Message Swipes](../adr/adr-017-message-swipes.md) — swipe semantics for retry of last AI message
- [system/prompt_system.md](./prompt_system.md) — layered prompt composition + per-layer token budget
- [system/llm_processing.md](./llm_processing.md) — LLM call logging + agent registry contracts
- [system/triggers.md](./triggers.md) — trigger evaluation rules + `NpcEncounterLog`
- [system/dashboard.md](./dashboard.md) — polling endpoints + status display
- [system/narration_engine.md](./narration_engine.md) — Game Master role + behavioral constraints


