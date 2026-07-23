//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt construction orchestration

pub mod assembler;
pub mod budget;
pub mod context;
pub mod types;

pub use assembler::{AssembledPrompt, PromptAssembler};
pub use context::{fit_messages_to_context, make_prompt_context};
pub use types::{NpcContext, PromptContext, PromptLayer};

#[cfg(test)]
mod assembler_tests;
#[cfg(test)]
mod budget_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod types_tests;
