//! [DOC: docs/diataxis/reference/game_flow.md]
//! Domain model utility modules.

pub mod game_name;
pub(crate) mod name_uniqueness;
pub mod scenario_defaults;
pub mod settings_defaults;
pub mod world_defaults;
pub mod xml;

pub(crate) use name_uniqueness::name_is_available;

#[cfg(test)]
mod xml_tests;
