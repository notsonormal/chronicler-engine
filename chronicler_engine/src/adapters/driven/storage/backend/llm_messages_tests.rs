use chrono::Utc;
use crate::application::ports::llm_message_repository::LlmMessage;
use crate::adapters::driven::storage::backend::{Storage, TestOverride};
use crate::test_support::sqlite_storage;

fn dummy_llm_message(model: &str) -> crate::application::ports::llm_message_repository::LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: "narrator".to_string(),
        backend_name: "test".to_string(),
        model_name: model.to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: "req".to_string(),
        raw_response_json: "res".to_string(),
        parsed_response: "parsed".to_string(),
        error_message: None,
        created_at: Utc::now(),
    }
}

#[test]
fn test_save_llm_message_in_memory() {
    let storage = Storage::new_in_memory();
    let msg = dummy_llm_message("model1");
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 1);
}

#[test]
fn test_save_llm_message_sqlite() {
    let storage = sqlite_storage().unwrap();
    let msg = dummy_llm_message("model1");
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 1);
}

#[test]
fn test_list_latest_llm_messages_empty() {
    let storage = Storage::new_in_memory();
    let list = storage.list_latest_llm_messages(50).unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_list_latest_llm_messages_single() {
    let storage = Storage::new_in_memory();
    storage.save_llm_message(&dummy_llm_message("m1")).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].model_name, "m1");
}

#[test]
fn test_list_latest_llm_messages_multiple() {
    let storage = Storage::new_in_memory();
    storage.save_llm_message(&dummy_llm_message("m1")).unwrap();
    storage.save_llm_message(&dummy_llm_message("m2")).unwrap();
    storage.save_llm_message(&dummy_llm_message("m3")).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 3);
}

#[tokio::test]
async fn test_list_latest_llm_messages_ordered_by_created_at() {
    let storage = Storage::new_in_memory();
    storage
        .save_llm_message(&dummy_llm_message("first"))
        .unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    storage
        .save_llm_message(&dummy_llm_message("second"))
        .unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list[0].model_name, "first");
    assert_eq!(list[1].model_name, "second");
}

#[test]
fn test_list_latest_llm_messages_limit_applied() {
    let storage = Storage::new_in_memory();
    for i in 0..10 {
        storage
            .save_llm_message(&dummy_llm_message(&format!("m{i}")))
            .unwrap();
    }

    let list = storage.list_latest_llm_messages(3).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn test_llm_message_cap_prunes_oldest() {
    let storage = sqlite_storage().unwrap();
    for i in 0..55 {
        storage
            .save_llm_message(&dummy_llm_message(&format!("model-{i}")))
            .unwrap();
    }

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 50);
    assert_eq!(list[0].model_name, "model-5");
    assert_eq!(list[49].model_name, "model-54");
}

#[test]
fn test_llm_message_cap_prunes_exact_50() {
    let storage = Storage::new_in_memory();
    for i in 0..50 {
        storage
            .save_llm_message(&dummy_llm_message(&format!("model-{i}")))
            .unwrap();
    }

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 50);
    assert_eq!(list[0].model_name, "model-0");
    assert_eq!(list[49].model_name, "model-49");
}

#[test]
fn test_llm_message_cap_prunes_in_memory() {
    let storage = Storage::new_in_memory();
    for i in 0..55 {
        storage
            .save_llm_message(&dummy_llm_message(&format!("model-{i}")))
            .unwrap();
    }

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list.len(), 50);
    assert_eq!(list[0].model_name, "model-5");
}

#[test]
fn test_llm_message_all_fields_preserved() {
    let storage = Storage::new_in_memory();
    let msg = LlmMessage {
        id: 0,
        agent_name: "agent".to_string(),
        backend_name: "backend".to_string(),
        model_name: "model".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: "req".to_string(),
        raw_response_json: "res".to_string(),
        parsed_response: "parsed".to_string(),
        error_message: Some("error".to_string()),
        created_at: Utc::now(),
    };

    storage.save_llm_message(&msg).unwrap();
    let list = storage.list_latest_llm_messages(50).unwrap();

    assert_eq!(list[0].agent_name, "agent");
    assert_eq!(list[0].backend_name, "backend");
    assert_eq!(list[0].model_name, "model");
    assert_eq!(list[0].system_prompt, "sys");
    assert_eq!(list[0].user_prompt, "user");
    assert_eq!(list[0].error_message, Some("error".to_string()));
}

#[test]
fn test_llm_message_agent_name() {
    let storage = Storage::new_in_memory();
    let msg = LlmMessage {
        id: 0,
        agent_name: "custom_agent".to_string(),
        backend_name: String::new(),
        model_name: "m".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: None,
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list[0].agent_name, "custom_agent");
}

#[test]
fn test_llm_message_backend_name() {
    let storage = Storage::new_in_memory();
    let msg = LlmMessage {
        id: 0,
        agent_name: String::new(),
        backend_name: "custom_backend".to_string(),
        model_name: "m".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: None,
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list[0].backend_name, "custom_backend");
}

#[test]
fn test_llm_message_model_name() {
    let storage = Storage::new_in_memory();
    let msg = LlmMessage {
        id: 0,
        agent_name: String::new(),
        backend_name: String::new(),
        model_name: "gpt-4".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: None,
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list[0].model_name, "gpt-4");
}

#[test]
fn test_llm_message_system_prompt() {
    let storage = Storage::new_in_memory();
    let msg = LlmMessage {
        id: 0,
        agent_name: String::new(),
        backend_name: String::new(),
        model_name: "m".to_string(),
        system_prompt: "You are a helpful assistant".to_string(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: None,
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list[0].system_prompt, "You are a helpful assistant");
}

#[test]
fn test_llm_message_user_prompt() {
    let storage = Storage::new_in_memory();
    let msg = LlmMessage {
        id: 0,
        agent_name: String::new(),
        backend_name: String::new(),
        model_name: "m".to_string(),
        system_prompt: String::new(),
        user_prompt: "Hello, AI".to_string(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: None,
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list[0].user_prompt, "Hello, AI");
}

#[test]
fn test_llm_message_error_message_some() {
    let storage = Storage::new_in_memory();
    let msg = LlmMessage {
        id: 0,
        agent_name: String::new(),
        backend_name: String::new(),
        model_name: "m".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: Some("API timeout".to_string()),
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(list[0].error_message, Some("API timeout".to_string()));
}

#[test]
fn test_llm_message_raw_json_blobs() {
    let storage = Storage::new_in_memory();
    let large_request = "{\"key\": \"value\"}".repeat(100);
    let large_response = "{\"response\": \"data\"}".repeat(100);

    let msg = LlmMessage {
        id: 0,
        agent_name: String::new(),
        backend_name: String::new(),
        model_name: "m".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: large_request,
        raw_response_json: large_response,
        parsed_response: String::new(),
        error_message: None,
        created_at: Utc::now(),
    };
    storage.save_llm_message(&msg).unwrap();

    let list = storage.list_latest_llm_messages(50).unwrap();
    assert!(list[0].raw_request_json.len() > 1000);
    assert!(list[0].raw_response_json.len() > 1000);
}

#[test]
fn test_save_llm_message_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("save_llm_message", TestOverride::internal("save failed"));

    let msg = dummy_llm_message("m1");
    let result = storage.save_llm_message(&msg);
    assert!(result.is_err());
}

#[test]
fn test_list_latest_llm_messages_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set(
        "list_latest_llm_messages",
        TestOverride::config("list failed"),
    );

    let result = storage.list_latest_llm_messages(50);
    assert!(result.is_err());
}
