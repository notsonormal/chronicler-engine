use chronicler_engine::model::llm_message::LlmMessage;
use chronicler_engine::storage::db::DbPool;
use chronicler_engine::storage::llm_message_storage::{LlmMessageStorage, SqliteLlmMessageStorage};

fn create_storage() -> SqliteLlmMessageStorage {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    SqliteLlmMessageStorage::new(pool)
}

#[test]
fn test_sqlite_save_and_list() {
    let storage = create_storage();
    let msg = LlmMessage::new(
        "narrator",
        "OpenRouter",
        "gpt-4",
        "system prompt",
        "user prompt",
        "{\"model\":\"gpt-4\"}",
        "{\"content\":\"hello\"}",
        "hello",
        None::<String>,
    );
    storage.save(&msg).unwrap();

    let list = storage.list_latest(50).unwrap();
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
    let msg = LlmMessage::new(
        "quantifier",
        "Ollama",
        "llama3",
        "sys",
        "user",
        "req",
        "error body",
        "",
        Some("HTTP 500"),
    );
    storage.save(&msg).unwrap();

    let list = storage.list_latest(50).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].error_message, Some("HTTP 500".to_string()));
}

#[test]
fn test_sqlite_global_cap_prunes_oldest() {
    let storage = create_storage();
    for i in 0..55 {
        let msg = LlmMessage::new(
            "narrator",
            "OpenRouter",
            format!("model-{i}"),
            "sys",
            "user",
            "req",
            "res",
            "parsed",
            None::<String>,
        );
        storage.save(&msg).unwrap();
    }

    let list = storage.list_latest(50).unwrap();
    assert_eq!(list.len(), 50);
    // Should be ordered oldest-first (newest-last) due to reverse() in list_latest
    assert_eq!(list[0].model_name, "model-5");
    assert_eq!(list[49].model_name, "model-54");
}

#[test]
fn test_sqlite_list_latest_limit() {
    let storage = create_storage();
    for i in 0..10 {
        let msg = LlmMessage::new(
            "narrator",
            "OpenRouter",
            format!("model-{i}"),
            "sys",
            "user",
            "req",
            "res",
            "parsed",
            None::<String>,
        );
        storage.save(&msg).unwrap();
    }

    let list = storage.list_latest(3).unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].model_name, "model-7");
    assert_eq!(list[1].model_name, "model-8");
    assert_eq!(list[2].model_name, "model-9");
}

#[test]
fn test_sqlite_empty_list() {
    let storage = create_storage();
    let list = storage.list_latest(50).unwrap();
    assert!(list.is_empty());
}
