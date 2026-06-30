//! [DOC: docs/system/llm_processing.md]
//! Application ports: outbound interfaces (driven port traits)

pub mod llm_message_repository;
pub mod llm_provider;

#[cfg(test)]
mod llm_provider_tests;
