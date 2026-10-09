//! [DOC: docs/diataxis/reference/game_flow.md]
//! Canonical phase-level error type for the action pipeline.

use crate::domain::model::state::generation_status::GenerationFailure;
use crate::EngineError;

#[derive(Debug)]
pub enum PhaseError {
    /// Pipeline cancelled via `CancellationToken`; partial artifacts discarded, state rolled back.
    Cancelled,
    /// Narrator LLM call failed; the classified failure is what the status display renders.
    NarratorFailed(GenerationFailure),
    /// Persistence gate rejected the write at `label`; `source` is the underlying `EngineError`.
    PersistFailed {
        label: &'static str,
        source: EngineError,
    },
    /// Trigger id referenced by the scenario was absent from the trigger index.
    TriggerMissing,
    /// Snapshot expected at this phase was absent from storage.
    SnapshotMissing,
    /// Retry precondition fetch failed (world/persona/npc/game bundle lookup).
    FetchFailed(GenerationFailure),
}
