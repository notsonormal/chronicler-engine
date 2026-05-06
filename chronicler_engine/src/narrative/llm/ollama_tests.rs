use crate::narrative::llm::backend::LlmBackend;
use crate::narrative::llm::ollama::OllamaBackend;

#[test]
fn test_ollama_backend_name() {
    let backend = OllamaBackend::default();
    assert_eq!(backend.name(), "Ollama");
}
