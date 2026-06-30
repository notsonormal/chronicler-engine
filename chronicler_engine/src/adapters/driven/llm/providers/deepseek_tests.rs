use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driven::llm::providers::deepseek::DeepSeekBackend;

#[test]
fn test_deepseek_name() {
    let backend = DeepSeekBackend::default();
    assert_eq!(backend.name(), "DeepSeek");
}

#[test]
fn test_deepseek_complete() {
    let backend = DeepSeekBackend::default();
    let result = backend.complete("test", "system", "user", None);
    assert!(
        result.is_err(),
        "DeepSeek complete should return Err (not yet implemented)"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("not yet implemented")
    );
}

#[test]
fn test_deepseek_all_methods_return_not_implemented() {
    let backend = DeepSeekBackend::default();

    let prompt_result = backend.complete("test", "sys", "user", None);
    assert!(prompt_result.is_err());
}

#[test]
fn test_deepseek_error_message_descriptive() {
    let backend = DeepSeekBackend::default();
    let result = backend.complete("test", "system", "user", None);
    assert!(
        result.is_err(),
        "DeepSeek should return Err, not a placeholder string"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("not yet implemented"),
        "Error message should explain the backend is unimplemented, got: {msg}"
    );
}
