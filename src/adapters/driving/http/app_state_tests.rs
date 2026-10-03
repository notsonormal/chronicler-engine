//! Unit tests for AppState helpers.

use std::sync::Arc;

use chrono::Utc;

use crate::adapters::driven::storage::Storage;
use crate::adapters::driving::http::AppState;
use crate::application::ports::text_checker::{CheckResult, TextChecker};
use crate::application::text_check_service::TextCheckService;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::settings::{AppSettings, TextCheckMode};
use crate::error::EngineError;
use crate::test_support::TestAppBuilder;

fn make_test_app_state(llm_storage: Option<Arc<Storage>>) -> AppState {
    let mut builder = TestAppBuilder::default_test();
    if let Some(storage) = llm_storage {
        builder = builder.storage(storage);
    }
    builder.build_service()
}

#[test]
fn test_render_llm_messages_empty() {
    let app_state = make_test_app_state(None);
    let html = app_state.render_llm_messages().unwrap();
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
    let html = app_state.render_llm_messages().unwrap();
    assert!(html.contains("llm-message-list"));
    assert!(html.contains("narrator"));
    assert!(html.contains("OpenRouter"));
    assert!(html.contains("gpt-4"));
    assert!(html.contains("hello"));
}

struct NoopTextChecker;

impl TextChecker for NoopTextChecker {
    fn check(
        &self,
        _text: &str,
        _mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        Ok(None)
    }
}

fn build_app_state() -> AppState {
    let mut app_state = TestAppBuilder::default_test().build_service();
    app_state.text_check_service = Arc::new(TextCheckService::new(Arc::new(NoopTextChecker)));
    app_state
}

#[test]
fn test_settings_reads_stored_row() {
    let app_state = build_app_state();

    app_state
        .settings_service
        .update_settings(|settings| {
            settings.narration_connection_id = "poison-test".to_string();
            Ok(())
        })
        .expect("update_settings should succeed");

    let recovered = app_state.settings();
    assert_eq!(
        recovered
            .expect("settings should read")
            .narration_connection_id,
        "poison-test",
        "settings() should read the stored settings row"
    );
}

#[test]
fn test_settings_survive_a_rebuilt_app_state() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app_state, _storage) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .settings(AppSettings {
            narration_connection_id: "stored-narrator".to_string(),
            ..AppSettings::default()
        })
        .build_service_with_storage();

    let settings = app_state
        .settings_service
        .get_settings()
        .expect("get_settings should succeed");
    assert_eq!(settings.narration_connection_id, "stored-narrator");
}
