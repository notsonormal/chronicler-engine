//! [DOC: docs/system/game_flow.md]
//! Game state representations (re-exports from submodules)

pub mod game_state;
pub mod generation_status;
pub mod message_types;
pub mod movement;
pub mod narrative_state;
pub mod scene_state;
pub mod trigger_context;

pub use generation_status::*;
pub use message_types::*;
pub use movement::*;
pub use trigger_context::*;
pub use narrative_state::*;
pub use scene_state::*;
pub use game_state::*;

#[cfg(test)]
mod state_tests;
