use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::{AppSettings, Connection};
use crate::settings::{get_settings_path, load_settings};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard<'a> {
    key: &'a str,
    previous: Option<String>,
}

impl<'a> EnvGuard<'a> {
    fn set(key: &'a str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }

    fn unset(key: &'a str) -> Self {
        let previous = std::env::var(key).ok();
        unsafe { std::env::remove_var(key) };
        Self { key, previous }
    }
}

impl<'a> Drop for EnvGuard<'a> {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => unsafe { std::env::set_var(self.key, v) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

fn with_isolated_settings<F, R>(f: F) -> R
where
    F: FnOnce(&PathBuf) -> R,
{
    let _lock = ENV_LOCK.lock().unwrap();
    let tmp = std::env::temp_dir().join(format!("chronicler_settings_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let path = tmp.join("settings.json");
    let _ = std::fs::create_dir_all(&tmp);
    let _guard = EnvGuard::set("CHRONICLER_SETTINGS_PATH", path.to_str().unwrap());
    let result = f(&path);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

#[test]
fn test_get_settings_path_default() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::unset("CHRONICLER_SETTINGS_PATH");
    let path = get_settings_path();
    assert_eq!(path, PathBuf::from("data").join("settings.json"));
}

#[test]
fn test_get_settings_path_env_override() {
    let _lock = ENV_LOCK.lock().unwrap();
    let custom_path = "custom/settings.json";
    let _guard = EnvGuard::set("CHRONICLER_SETTINGS_PATH", custom_path);
    let path = get_settings_path();
    assert_eq!(path, PathBuf::from(custom_path));
}

#[test]
fn test_load_settings_missing_file_creates_defaults() {
    with_isolated_settings(|path| {
        assert!(!path.exists());

        let settings = load_settings().expect("should create defaults");
        assert_eq!(settings.connections.len(), 3);
        assert_eq!(settings.narration_connection_id, "openrouter-gpt-4o-mini");

        // Verify file was created
        assert!(path.exists());

        // Clean up for other tests
        let _ = std::fs::remove_file(path);
    });
}

#[test]
fn test_load_settings_valid_file() {
    with_isolated_settings(|_path| {
        let custom = AppSettings {
            connections: vec![Connection::new("test", "Test", LlmBackendType::OpenRouter)],
            narration_connection_id: "test".into(),
            quantifier_connection_id: "test".into(),
            response_length: "flexible".into(),
            ..Default::default()
        };
        custom.save().expect("should save");

        let loaded = load_settings().expect("should load");
        assert_eq!(loaded.narration_connection_id, "test");
        assert_eq!(loaded.connections.len(), 1);
    });
}

#[test]
fn test_load_settings_invalid_json() {
    with_isolated_settings(|path| {
        // Ensure parent directory exists
        let _ = std::fs::create_dir_all(path.parent().unwrap());

        let mut file = std::fs::File::create(path).expect("should create file");
        write!(file, "not json").expect("should write");
        drop(file);

        let result = load_settings();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse settings"));
    });
}

#[test]
fn test_save_settings_roundtrip() {
    with_isolated_settings(|_path| {
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
        settings.save().expect("should save");

        let loaded = load_settings().expect("should load");
        assert_eq!(loaded.narration_connection_id, "conn-1");
        assert_eq!(loaded.connections[0].model, "model-a");
    });
}

#[test]
fn test_save_settings_creates_parent_directory() {
    with_isolated_settings(|path| {
        let _ = std::fs::remove_dir_all(path.parent().unwrap());

        let settings = AppSettings::default();
        let result = settings.save();
        // Should fail because parent directory does not exist
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to write settings"));
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
    // Without env var, should be None
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
