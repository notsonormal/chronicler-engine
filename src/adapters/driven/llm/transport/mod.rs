//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM client interface

mod utils;

pub use utils::client::call_chat_completions;

pub use utils::request::{ChatCompletionRequest, ChatCompletionResult};
