//! Integration tests for LLM message persistence: save/list, error-message preservation, global-cap pruning, and pagination across a real SQLite-backed `Storage`.

use chrono::Utc;
use chronicler_engine::domain::model::llm_message::LlmMessage;
use chronicler_engine::adapters::driven::storage::llm_messages::LLM_MESSAGE_RETENTION_LIMIT;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::test_support::fixtures::llm_message;

use crate::fixtures::create_test_storage;

fn create_storage() -> Storage {
    create_test_storage(1)
}

#[test]
fn test_sqlite_save_and_list() {
    let storage = create_storage();
    let msg = LlmMessage {
        id: 0,
        agent_name: "narrator".to_string(),
        backend_name: "OpenRouter".to_string(),
        model_name: "gpt-4".to_string(),
        system_prompt: "system prompt".to_string(),
        user_prompt: "user prompt".to_string(),
        raw_request_json: "{\"model\":\"gpt-4\"}".to_string(),
        raw_response_json: "{\"content\":\"hello\"}".to_string(),
        parsed_response: "hello".to_string(),
        error_message: None,
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].agent_name, "narrator");
    assert_eq!(list[0].backend_name, "OpenRouter");
    assert_eq!(list[0].model_name, "gpt-4");
    assert_eq!(list[0].system_prompt, "system prompt");
    assert_eq!(list[0].user_prompt, "user prompt");
    assert_eq!(list[0].raw_request_json, "{\"model\":\"gpt-4\"}");
    assert_eq!(list[0].raw_response_json, "{\"content\":\"hello\"}");
    assert_eq!(list[0].parsed_response, "hello");
    assert!(list[0].error_message.is_none());
}

#[test]
fn test_sqlite_error_message_preserved() {
    let storage = create_storage();
    let msg = LlmMessage {
        id: 0,
        agent_name: "quantifier".to_string(),
        backend_name: "Ollama".to_string(),
        model_name: "llama3".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: "req".to_string(),
        raw_response_json: "error body".to_string(),
        parsed_response: String::new(),
        error_message: Some("HTTP 500".to_string()),
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].error_message, Some("HTTP 500".to_string()));
}

#[test]
fn test_sqlite_global_cap_prunes_oldest() {
    let storage = create_storage();
    for i in 0..55 {
        let msg = LlmMessage {
            id: 0,
            agent_name: "narrator".to_string(),
            backend_name: "OpenRouter".to_string(),
            model_name: format!("model-{i}"),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            raw_request_json: "req".to_string(),
            raw_response_json: "res".to_string(),
            parsed_response: "parsed".to_string(),
            error_message: None,
            created_at: Utc::now(),
        };
        storage.save_llm_message(&msg).unwrap();
    }

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 50);
    // Should be ordered oldest-first (newest-last) due to reverse() in list_latest
    assert_eq!(list[0].model_name, "model-5");
    assert_eq!(list[49].model_name, "model-54");
}

#[test]
fn test_sqlite_list_latest_limit() {
    let storage = create_storage();
    for i in 0..10 {
        let msg = LlmMessage {
            id: 0,
            agent_name: "narrator".to_string(),
            backend_name: "OpenRouter".to_string(),
            model_name: format!("model-{i}"),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            raw_request_json: "req".to_string(),
            raw_response_json: "res".to_string(),
            parsed_response: "parsed".to_string(),
            error_message: None,
            created_at: Utc::now(),
        };
        storage.save_llm_message(&msg).unwrap();
    }

    let list = storage.list_latest_llm_messages(3).unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].model_name, "model-7");
    assert_eq!(list[1].model_name, "model-8");
    assert_eq!(list[2].model_name, "model-9");
}

#[test]
fn test_sqlite_empty_list() {
    let storage = create_storage();
    let list = storage.list_latest_llm_messages(50).unwrap();
    assert!(list.is_empty());
}

// The reader that derives per-agent health folds one whole window, so the
// window must cover every agent, not just the agents of its newest rows.
#[test]
fn test_sqlite_whole_window_holds_every_roles_newest_attempt() {
    let storage = create_storage();
    storage
        .save_llm_message(&llm_message(
            "narrator",
            "old-model",
            Some("old failure"),
            -20,
        ))
        .unwrap();
    storage
        .save_llm_message(&llm_message("narrator", "new-model", None, -15))
        .unwrap();
    storage
        .save_llm_message(&llm_message("quantifier", "q-model", Some("q failure"), -5))
        .unwrap();

    let window = storage
        .list_latest_llm_messages(LLM_MESSAGE_RETENTION_LIMIT)
        .unwrap();

    assert_eq!(window.len(), 3, "retention keeps every saved row");
    assert!(
        window
            .iter()
            .any(|m| m.agent_name == "narrator" && m.model_name == "new-model"),
        "the narrator's newer attempt must be in the window"
    );
    assert!(
        window.iter().any(
            |m| m.agent_name == "narrator" && m.error_message.as_deref() == Some("old failure")
        ),
        "the narrator's superseded attempt stays in the window"
    );
    assert!(
        window.iter().any(
            |m| m.agent_name == "quantifier" && m.error_message.as_deref() == Some("q failure")
        ),
        "a role whose newest row is not the window's newest row must still be in it"
    );
}
