//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Application-layer utility modules.

pub mod llm_provider;
pub mod sanitize;
pub mod slot;
pub mod spawn;
pub mod token_budget;

#[cfg(test)]
mod sanitize_tests;

#[cfg(test)]
mod slot_tests;

#[cfg(test)]
mod token_budget_tests;
