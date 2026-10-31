use super::llm_backend::LlmBackendType;

#[test]
fn test_from_str_known_variants() {
    assert_eq!(
        "openrouter".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::OpenRouter
    );
    assert_eq!(
        "mock".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::Mock
    );
    assert_eq!(
        "ollama".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::Ollama
    );
    assert_eq!(
        "opencode-go".parse::<LlmBackendType>().unwrap(),
        LlmBackendType::OpenCodeGo
    );
}

#[test]
fn test_wire_ids_are_what_the_connection_form_sends() {
    assert_eq!(LlmBackendType::OpenRouter.wire_id(), "openrouter");
    assert_eq!(LlmBackendType::OpenCodeGo.wire_id(), "opencode-go");
    assert_eq!(LlmBackendType::Ollama.wire_id(), "ollama");
    assert_eq!(LlmBackendType::Mock.wire_id(), "mock");
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
fn test_from_str_deepseek_is_no_longer_a_provider() {
    assert!("deepseek".parse::<LlmBackendType>().is_err());
}

#[test]
fn test_default_is_openrouter() {
    assert_eq!(LlmBackendType::default(), LlmBackendType::OpenRouter);
}

#[test]
fn test_selectable_providers_leave_out_mock() {
    assert_eq!(
        LlmBackendType::SELECTABLE,
        [
            LlmBackendType::OpenRouter,
            LlmBackendType::OpenCodeGo,
            LlmBackendType::Ollama
        ]
    );
}

#[test]
fn test_display_labels() {
    assert_eq!(LlmBackendType::OpenRouter.display_label(), "OpenRouter");
    assert_eq!(LlmBackendType::Ollama.display_label(), "Ollama");
    assert_eq!(LlmBackendType::OpenCodeGo.display_label(), "OpenCode Go");
    assert_eq!(LlmBackendType::Mock.display_label(), "Mock");
}

#[test]
fn test_default_base_urls() {
    assert_eq!(
        LlmBackendType::OpenRouter.default_base_url(),
        "https://openrouter.ai/api/v1"
    );
    assert_eq!(
        LlmBackendType::Ollama.default_base_url(),
        "http://localhost:11434/v1"
    );
    assert_eq!(LlmBackendType::Mock.default_base_url(), "");
    assert_eq!(
        LlmBackendType::OpenCodeGo.default_base_url(),
        "https://opencode.ai/zen/go/v1"
    );
}

#[test]
fn test_default_max_context_tokens() {
    assert_eq!(
        LlmBackendType::OpenRouter.default_max_context_tokens(),
        32768
    );
    assert_eq!(LlmBackendType::Ollama.default_max_context_tokens(), 8192);
    assert_eq!(LlmBackendType::Mock.default_max_context_tokens(), 4096);
    assert_eq!(
        LlmBackendType::OpenCodeGo.default_max_context_tokens(),
        32768
    );
}

#[test]
fn test_only_openrouter_sends_an_attribution_title() {
    assert_eq!(
        LlmBackendType::OpenRouter.attribution_title(),
        Some("Chronicler Engine")
    );
    assert_eq!(LlmBackendType::Ollama.attribution_title(), None);
    assert_eq!(LlmBackendType::Mock.attribution_title(), None);
    assert_eq!(LlmBackendType::OpenCodeGo.attribution_title(), None);
}

#[test]
fn test_api_key_env_vars() {
    assert_eq!(
        LlmBackendType::OpenRouter.api_key_env_var(),
        Some("OPENROUTER_API_KEY")
    );
    assert_eq!(
        LlmBackendType::OpenCodeGo.api_key_env_var(),
        Some("OPENCODE_GO_API_KEY")
    );
    assert_eq!(LlmBackendType::Ollama.api_key_env_var(), None);
    assert_eq!(LlmBackendType::Mock.api_key_env_var(), None);
}

#[test]
fn test_only_opencode_go_names_a_session_header() {
    assert_eq!(
        LlmBackendType::OpenCodeGo.session_header(),
        Some("x-opencode-session")
    );
    assert_eq!(LlmBackendType::OpenRouter.session_header(), None);
    assert_eq!(LlmBackendType::Ollama.session_header(), None);
    assert_eq!(LlmBackendType::Mock.session_header(), None);
}

#[test]
fn test_only_openrouter_supports_routing() {
    assert!(LlmBackendType::OpenRouter.supports_routing());
    assert!(!LlmBackendType::Ollama.supports_routing());
    assert!(!LlmBackendType::OpenCodeGo.supports_routing());
    assert!(!LlmBackendType::Mock.supports_routing());
}
