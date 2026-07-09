//! [DOC: docs/system/game_flow.md]
//! GenerationGate — owns the per-process cancellation token + `is_generating`
//! cache (ADR-030) + slot-orchestration around `process_action`.
//! (T2 ticket 03 — façade-first carve-out from DefaultApplicationService.)

pub mod gate;

pub use gate::GenerationGate;
