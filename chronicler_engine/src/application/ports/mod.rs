//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Application ports: outbound interfaces (driven port traits)

pub mod llm_message_repository;
pub mod llm_provider;
pub mod text_checker;

#[cfg(test)]
mod llm_message_repository_tests;

#[cfg(test)]
mod llm_provider_tests;

#[cfg(test)]
mod text_checker_tests;
