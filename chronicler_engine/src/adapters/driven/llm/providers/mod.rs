//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! LLM provider implementations

pub mod deepseek;
pub mod mock;
pub mod ollama;
pub mod openrouter;

#[cfg(test)]
mod deepseek_tests;
#[cfg(test)]
mod mock_tests;
#[cfg(test)]
mod ollama_tests;
#[cfg(test)]
mod openrouter_tests;

pub use deepseek::DeepSeekBackend;
pub use mock::MockBackend;
pub use ollama::OllamaBackend;
pub use openrouter::OpenRouterBackend;
