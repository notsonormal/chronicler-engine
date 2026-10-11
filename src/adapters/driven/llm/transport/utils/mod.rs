//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM transport implementation helpers.

pub mod client;
pub mod request;
pub mod response;

#[cfg(test)]
mod request_tests;

#[cfg(test)]
mod response_tests;
