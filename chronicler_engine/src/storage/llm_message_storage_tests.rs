use crate::model::llm_message::LlmMessage;
use crate::storage::llm_message_storage::InMemoryLlmMessageStorage;
use crate::storage::llm_message_storage::LlmMessageStorage;

#[test]
fn test_in_memory_save_and_list() {
    let storage = InMemoryLlmMessageStorage::new();
    let msg = LlmMessage::new(
        "narrator",
        "OpenRouter",
        "gpt-4",
        "sys",
        "user",
        "req",
        "res",
        "parsed",
        None::<String>,
    );
    storage.save(&msg).unwrap();

    let list = storage.list_latest(50).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].agent_name, "narrator");
    assert_eq!(list[0].backend_name, "OpenRouter");
    assert_eq!(list[0].model_name, "gpt-4");
}

#[test]
fn test_in_memory_ring_buffer_cap() {
    let storage = InMemoryLlmMessageStorage::new();
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
    // Oldest (0-4) should have been evicted
    assert_eq!(list[0].model_name, "model-5");
    assert_eq!(list[49].model_name, "model-54");
}

#[test]
fn test_in_memory_list_latest_limit() {
    let storage = InMemoryLlmMessageStorage::new();
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
fn test_in_memory_empty() {
    let storage = InMemoryLlmMessageStorage::new();
    assert!(storage.is_empty());
    assert_eq!(storage.len(), 0);
    let list = storage.list_latest(50).unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_in_memory_len_updates() {
    let storage = InMemoryLlmMessageStorage::new();
    assert_eq!(storage.len(), 0);

    let msg = LlmMessage::new(
        "narrator",
        "OpenRouter",
        "gpt-4",
        "sys",
        "user",
        "req",
        "res",
        "parsed",
        None::<String>,
    );
    storage.save(&msg).unwrap();
    assert_eq!(storage.len(), 1);

    storage.save(&msg).unwrap();
    assert_eq!(storage.len(), 2);
}
