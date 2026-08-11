use super::llm_backend::LlmBackendType;

#[test]
fn test_from_str_known_variants() {
    assert_eq!(
        LlmBackendType::from("openrouter"),
        LlmBackendType::OpenRouter
    );
    assert_eq!(LlmBackendType::from("deepseek"), LlmBackendType::DeepSeek);
    assert_eq!(LlmBackendType::from("mock"), LlmBackendType::Mock);
    assert_eq!(LlmBackendType::from("ollama"), LlmBackendType::Ollama);
}

#[test]
fn test_from_str_unknown_defaults_to_mock() {
    assert_eq!(LlmBackendType::from("nonsense"), LlmBackendType::Mock);
    assert_eq!(LlmBackendType::from(""), LlmBackendType::Mock);
}

#[test]
fn test_default_is_openrouter() {
    assert_eq!(LlmBackendType::default(), LlmBackendType::OpenRouter);
}

#[test]
fn test_from_env_unset_defaults_to_openrouter() {
    // SAFETY: nextest runs each test in its own process, so mutating the
    // process environment here does not race other tests.
    unsafe {
        std::env::remove_var("LLM_BACKEND");
    }
    assert_eq!(LlmBackendType::from_env(), LlmBackendType::OpenRouter);
}

#[test]
fn test_from_env_reads_variable() {
    // SAFETY: see note in test_from_env_unset_defaults_to_openrouter.
    unsafe {
        std::env::set_var("LLM_BACKEND", "deepseek");
    }
    assert_eq!(LlmBackendType::from_env(), LlmBackendType::DeepSeek);
    unsafe {
        std::env::remove_var("LLM_BACKEND");
    }
}
