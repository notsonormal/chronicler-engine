//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt construction orchestration

pub mod assembler;
pub mod builders;
pub mod types;
pub mod utils;

pub use assembler::{AssembledPrompt, PromptAssembler};
pub use utils::context::fit_messages_to_context;
pub use types::{NpcContext, PromptContext, PromptLayer};

#[cfg(test)]
mod assembler_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod types_tests;
