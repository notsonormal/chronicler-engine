//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt construction orchestration

pub mod assembler;
pub mod builders;
pub mod prompt_merge;
pub mod sanitize;
pub mod token_budget;
pub mod types;
pub mod utils;

pub use assembler::{AssembledPrompt, PromptAssembler, PromptContext};
pub use types::{NpcContext, PromptLayer};
pub use utils::context::fit_messages_to_context;

#[cfg(test)]
mod assembler_tests;
#[cfg(test)]
mod sanitize_tests;
#[cfg(test)]
mod token_budget_tests;
#[cfg(test)]
mod types_tests;
