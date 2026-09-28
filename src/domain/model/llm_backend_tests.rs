use super::llm_backend::LlmBackendType;

#[test]
fn test_from_str_known_variants() {
    assert_eq!(
        "openrouter".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::OpenRouter
    );
    assert_eq!(
        "deepseek".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::DeepSeek
    );
    assert_eq!(
        "mock".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::Mock
    );
    assert_eq!(
        "ollama".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::Ollama
    );
}

#[test]
fn test_from_str_unknown_is_error() {
    let err = "nonsense".parse::<LlmBackendType>().unwrap_err();
    assert!(
        err.to_string().contains("nonsense"),
        "error should name the rejected value, got: {err}"
    );
    assert!("".parse::<LlmBackendType>().is_err());
}

#[test]
fn test_default_is_openrouter() {
    assert_eq!(LlmBackendType::default(), LlmBackendType::OpenRouter);
}
