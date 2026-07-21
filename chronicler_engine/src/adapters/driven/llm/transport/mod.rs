//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! LLM client interface

#![allow(unused_imports)]

mod client;
mod request;
mod response;

pub use response::{extract_content_from_response, handle_response, parse_chat_response};
pub use client::{call_chat_completions, call_openrouter_with_model, call_ollama};

pub use request::ChatCompletionResult;

pub(crate) use request::next_request_id;

#[cfg(test)]
mod request_tests;

#[cfg(test)]
mod response_tests;
