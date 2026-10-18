//! [DOC: docs/diataxis/reference/game_flow.md]
//! Domain model utility modules.

pub mod game_name;
pub mod game_rules;
pub mod generation_record;
pub(crate) mod name_uniqueness;
pub mod scenario_defaults;
pub mod settings_defaults;
pub mod world_defaults;
pub mod xml;

pub(crate) use name_uniqueness::name_is_available;

#[cfg(test)]
mod game_rules_tests;
#[cfg(test)]
mod generation_record_tests;
#[cfg(test)]
mod xml_tests;
