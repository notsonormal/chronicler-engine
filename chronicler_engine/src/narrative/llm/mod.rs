pub mod backend;
pub mod deepseek;
pub mod mock;
pub mod ollama;
pub mod openrouter;
pub mod sanitize;

#[cfg(test)]
mod backend_tests;
#[cfg(test)]
mod deepseek_tests;
#[cfg(test)]
mod mock_tests;
#[cfg(test)]
mod ollama_tests;
#[cfg(test)]
mod openrouter_tests;
#[cfg(test)]
mod sanitize_tests;

pub use backend::{LlmBackend, LlmBackendType, get_llm_backend_for, merge_single_user_message};
pub use deepseek::DeepSeekBackend;
pub use mock::MockBackend;
pub use ollama::OllamaBackend;
pub use openrouter::OpenRouterBackend;

#[cfg(test)]
mod tests;
