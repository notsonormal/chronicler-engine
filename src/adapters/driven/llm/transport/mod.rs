//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM client interface

mod utils;

pub use utils::client::{call_openrouter_with_model, call_ollama};

pub use utils::request::ChatCompletionResult;
