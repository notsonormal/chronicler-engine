//! [DOC: docs/diataxis/reference/game_flow.md]
//! Generation gating and per-game slot orchestration.

pub mod gate;
pub mod guard;
pub mod slot;

#[cfg(test)]
mod gate_tests;
#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod slot_tests;
