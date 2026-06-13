use std::path::PathBuf;
use std::sync::Mutex;

use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::{AppSettings, Connection};
use crate::settings::{get_settings_path, load_settings};

/// Mutex to serialize tests that mutate the in-memory settings database.
static SETTINGS_DB_LOCK: Mutex<()> = Mutex::new(());

/// Helper to run tests that mutate the settings database.
/// The lock prevents concurrent tests from interfering with each other's in-memory DB.
fn with_isolated_settings<F, R>(f: F) -> R
where
    F: FnOnce(&PathBuf) -> R,
{
    let _lock = SETTINGS_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = std::env::temp_dir().join(format!("chronicler_settings_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let path = tmp.join("settings.json");
    let _ = std::fs::create_dir_all(&tmp);
    let result = f(&path);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

#[test]
fn test_get_settings_path_default() {
    let path = get_settings_path();
    assert_eq!(path, PathBuf::from("data").join("settings.json"));
}

#[test]
fn test_load_settings_valid_file() {
    with_isolated_settings(|_path| {
        let pool = crate::storage::db::DbPool::new(":memory:").unwrap();
        let storage = crate::storage::Storage::new_sqlite(pool, 1);

        let custom = AppSettings {
            connections: vec![Connection::new("test", "Test", LlmBackendType::OpenRouter)],
            narration_connection_id: "test".into(),
            quantifier_connection_id: "test".into(),
            response_length: "flexible".into(),
            ..Default::default()
        };
        custom.save(&storage).expect("should save");

        let loaded = load_settings(&storage).expect("should load");
        assert_eq!(loaded.narration_connection_id, "test");
        assert_eq!(loaded.connections.len(), 1);
    });
}

#[test]
fn test_save_settings_roundtrip() {
    with_isolated_settings(|_path| {
        let pool = crate::storage::db::DbPool::new(":memory:").unwrap();
        let storage = crate::storage::Storage::new_sqlite(pool, 1);

        let settings = AppSettings {
            connections: vec![Connection {
                id: "conn-1".into(),
                name: "Conn 1".into(),
                provider: LlmBackendType::OpenRouter,
                model: "model-a".into(),
                api_key: Some("key-a".into()),
                base_url: None,
                single_user_message: false,
                max_tokens: None,
                max_context_tokens: None,
            }],
            narration_connection_id: "conn-1".into(),
            quantifier_connection_id: "conn-1".into(),
            response_length: "flexible".into(),
            ..Default::default()
        };
        settings.save(&storage).expect("should save");

        let loaded = load_settings(&storage).expect("should load");
        assert_eq!(loaded.narration_connection_id, "conn-1");
        assert_eq!(loaded.connections[0].model, "model-a");
    });
}

#[test]
fn test_connection_resolve_api_key() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: Some("direct-key".into()),
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    assert_eq!(conn.resolve_api_key(), Some("direct-key".into()));

    let conn_no_key = Connection {
        api_key: None,
        ..conn
    };
    unsafe {
        std::env::set_var("OPENROUTER_API_KEY", "env-key");
    }
    assert_eq!(conn_no_key.resolve_api_key(), Some("env-key".into()));
    unsafe {
        std::env::remove_var("OPENROUTER_API_KEY");
    }
    assert_eq!(conn_no_key.resolve_api_key(), None);
}

#[test]
fn test_connection_resolve_base_url() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::Ollama,
        model: "model".into(),
        api_key: None,
        base_url: Some("http://custom:11434".into()),
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    assert_eq!(conn.resolve_base_url(), "http://custom:11434");

    let conn_default = Connection {
        base_url: None,
        ..conn
    };
    assert_eq!(conn_default.resolve_base_url(), "http://localhost:11434/v1");
}

#[test]
fn test_connection_resolve_max_context_tokens_defaults() {
    assert_eq!(
        Connection::new("t", "T", LlmBackendType::Ollama).resolve_max_context_tokens(),
        8192
    );
    assert_eq!(
        Connection::new("t", "T", LlmBackendType::OpenRouter).resolve_max_context_tokens(),
        32768
    );
    assert_eq!(
        Connection::new("t", "T", LlmBackendType::DeepSeek).resolve_max_context_tokens(),
        32768
    );
    assert_eq!(
        Connection::new("t", "T", LlmBackendType::Mock).resolve_max_context_tokens(),
        4096
    );
}

#[test]
fn test_connection_resolve_max_context_tokens_override() {
    let mut conn = Connection::new("t", "T", LlmBackendType::Ollama);
    conn.max_context_tokens = Some(16384);
    assert_eq!(conn.resolve_max_context_tokens(), 16384);

    conn.max_context_tokens = Some(2048);
    assert_eq!(conn.resolve_max_context_tokens(), 2048);
}
