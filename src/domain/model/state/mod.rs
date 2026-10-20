//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game state representations (submodule declarations)

pub mod generation_status;
pub mod message_types;
pub mod movement;
pub mod scene_state;
pub mod swipe_state;
pub mod trigger_context;

#[cfg(test)]
mod generation_status_tests;
#[cfg(test)]
mod swipe_state_tests;
#[cfg(test)]
mod trigger_context_tests;
