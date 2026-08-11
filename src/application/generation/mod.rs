//! [DOC: docs/diataxis/reference/game_flow.md]
//! Generation gating and per-game slot orchestration.

pub mod gate;
pub mod guard;
pub mod slot;

pub use gate::GenerationGate;
pub use guard::GenerationGuard;

#[cfg(test)]
mod gate_tests;
#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod slot_tests;
