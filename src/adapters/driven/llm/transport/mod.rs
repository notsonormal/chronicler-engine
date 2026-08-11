//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! LLM client interface

#![allow(unused_imports)]

mod utils;

pub use utils::response::{extract_content_from_response, handle_response, parse_chat_response};
pub use utils::client::{call_chat_completions, call_openrouter_with_model, call_ollama};

pub use utils::request::ChatCompletionResult;

pub(crate) use utils::request::next_request_id;
