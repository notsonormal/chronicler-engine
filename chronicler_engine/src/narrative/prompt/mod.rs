//! [DOC: docs/system/prompt_system.md]

pub mod budget;
pub mod builder;
pub mod context;
pub mod sanitize;
pub mod templates;
pub mod types;

pub use context::{fit_messages_to_context, make_prompt_context};
pub use sanitize::sanitize_for_prompt;
pub use types::PromptBuilder;
pub use types::{PromptContext, PromptLayer};

#[cfg(test)]
mod budget_tests;
#[cfg(test)]
mod builder_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod sanitize_tests;
#[cfg(test)]
mod types_tests;
