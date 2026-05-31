// Allow unused imports: these are intentional re-exports for the public API
// They appear unused locally but are consumed by external modules (ollama.rs, openrouter.rs, backend.rs)
#![allow(unused_imports)]

// [DOC: docs/system/llm_processing.md]

mod client;
mod request;
mod response;

// Re-export public API for backward compatibility
pub(crate) use request::{build_request_payload, configure_request, DEFAULT_MAX_TOKENS};
pub(crate) use response::{extract_content_from_response, handle_response, parse_chat_response};
pub(crate) use client::{call_chat_completions, call_openrouter_with_model, call_ollama};

// Re-export for tests
pub use request::ChatCompletionResult;

// Re-export next_request_id for tests
pub(crate) use request::next_request_id;

#[cfg(test)]
mod tests;
