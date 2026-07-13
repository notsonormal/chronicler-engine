//! Tests for `fragment_renderers.rs` (response helpers are in `response_tests.rs`).

use std::sync::Arc;

use chrono::Utc;

use crate::application::ports::llm_message_repository::LlmMessage;
use crate::adapters::driven::storage::Storage;
use crate::adapters::driving::http::fragments::render_llm_messages;
use crate::test_support::TestAppBuilder;

fn make_test_app_state(
    llm_storage: Option<Arc<Storage>>,
) -> crate::adapters::driving::http::AppState {
    let mut builder = TestAppBuilder::default_test();
    if let Some(storage) = llm_storage {
        builder = builder.storage(storage);
    }
    builder.build_app_state()
}

#[test]
fn test_render_llm_messages_empty() {
    let app_state = make_test_app_state(None);
    let html = render_llm_messages(&app_state).unwrap();
    assert!(html.contains("llm-message-list"));
    assert!(html.contains("No LLM messages yet"));
}

#[test]
fn test_render_llm_messages_with_data() {
    let llm_storage = Arc::new(Storage::new_in_memory());
    let msg = LlmMessage {
        id: 0,
        agent_name: "narrator".to_string(),
        backend_name: "OpenRouter".to_string(),
        model_name: "gpt-4".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: "req".to_string(),
        raw_response_json: "res".to_string(),
        parsed_response: "hello".to_string(),
        error_message: None,
        created_at: Utc::now(),
    };
    llm_storage.save_llm_message(&msg).unwrap();

    let app_state = make_test_app_state(Some(llm_storage));
    let html = render_llm_messages(&app_state).unwrap();
    assert!(html.contains("llm-message-list"));
    assert!(html.contains("narrator"));
    assert!(html.contains("OpenRouter"));
    assert!(html.contains("gpt-4"));
    assert!(html.contains("hello"));
}
