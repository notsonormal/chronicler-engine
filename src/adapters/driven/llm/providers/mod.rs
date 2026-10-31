//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM provider implementations

pub mod chat_completions;
pub mod mock;

#[cfg(test)]
mod chat_completions_tests;
#[cfg(test)]
mod mock_tests;

pub use chat_completions::ChatCompletionsBackend;
pub use mock::MockBackend;
