//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game state representations (submodule declarations)

pub mod event_record;
pub mod generation_status;
pub mod message_types;
pub mod movement;
pub mod scene_state;
pub mod swipe_state;

#[cfg(test)]
mod generation_status_tests;
#[cfg(test)]
mod swipe_state_tests;
