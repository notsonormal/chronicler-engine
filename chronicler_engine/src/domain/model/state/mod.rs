//! [DOC: docs/system/game_flow.md]
//! Game state representations (submodule declarations)

pub mod game_state;
pub mod game_state_snapshot;
pub mod generation_status;
pub mod message_types;
pub mod movement;
pub mod narrative_state;
pub mod scene_state;
pub mod trigger_context;

#[cfg(test)]
mod game_state_snapshot_tests;
#[cfg(test)]
mod state_tests;
