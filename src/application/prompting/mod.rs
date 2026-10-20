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
pub use types::NpcContext;
