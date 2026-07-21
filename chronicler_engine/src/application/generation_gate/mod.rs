//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! GenerationGate — `is_generating` cache (ADR-030) + per-game slot orchestration.

pub mod gate;
pub mod slot;

pub use gate::GenerationGate;
pub use slot::{GenerationSlot, release_owned_slot};
