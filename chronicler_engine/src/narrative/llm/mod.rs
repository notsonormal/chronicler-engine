pub mod backend;
pub mod deepseek;
pub mod mock;
pub mod ollama;
pub mod openrouter;

#[cfg(test)]
mod backend_tests;
#[cfg(test)]
mod deepseek_tests;
#[cfg(test)]
mod mock_tests;
#[cfg(test)]
mod ollama_tests;

pub use backend::{
    LlmBackend, LlmBackendType, get_llm_backend, get_llm_backend_for,
    get_llm_backend_with_settings, merge_single_user_message,
};
pub use deepseek::DeepSeekBackend;
pub use mock::MockBackend;
pub use ollama::OllamaBackend;
pub use openrouter::OpenRouterBackend;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests;
