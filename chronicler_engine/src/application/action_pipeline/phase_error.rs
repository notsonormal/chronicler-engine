//! [DOC: docs/system/game_flow.md]
//! Canonical phase-level error type for the action pipeline.

use crate::EngineError;

#[derive(Debug)]
pub enum PhaseError {
    Cancelled,
    NarratorFailed(String),
    PersistFailed {
        label: &'static str,
        source: EngineError,
    },
    TriggerMissing,
    SnapshotMissing,
}
