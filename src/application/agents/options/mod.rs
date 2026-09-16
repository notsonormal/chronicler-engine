//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options agent system — generates the pickable next-action option set.
pub mod agent;
pub mod prompt;
pub(crate) mod types;
pub(crate) mod utils;

pub use agent::OptionsAgent;
pub use prompt::OptionsPromptBuilder;
pub use utils::orchestration::generate_options;
pub use utils::parser::parse_options;

#[cfg(test)]
mod agent_tests;
#[cfg(test)]
mod prompt_tests;
