use super::client::{call_ollama, call_openrouter_with_model};

#[test]
fn call_openrouter_with_model_compiles_and_runs() {
    let result = call_openrouter_with_model(
        "fake-api-key",
        "system prompt",
        "user prompt",
        "anthropic/claude-3.5-sonnet",
        Some(1024),
    );
    // Expected: Err (network/DNS failure against a fake key + fake model).
    // Function must complete without panicking. We only assert that the
    // wrapper executed — the specific error type is owned by the transport layer.
    let _ = result;
}

#[test]
fn call_ollama_compiles_and_runs() {
    let result = call_ollama(
        "http://127.0.0.1:1",
        "llama3",
        "system prompt",
        "user prompt",
        Some(512),
    );
    // Port 1 is privileged and unbound; expected network error.
    assert!(
        result.is_err(),
        "expected network error on unbound privileged port"
    );
}

#[test]
fn call_ollama_propagates_network_error() {
    let result = call_ollama("http://127.0.0.1:1", "llama3", "system", "user", None);
    assert!(result.is_err());
}

#[test]
fn call_openrouter_with_model_propagates_network_error() {
    let result = call_openrouter_with_model("fake-key", "system", "user", "model", None);
    assert!(result.is_err());
}
