use crate::model::{
    agent::{AgentConfig, BackendSelector, ExecutionPhase},
    settings::{AppSettings, Connection, TextCheckMode, TextCheckSettings},
};
use crate::storage::backend::{Operation, Storage, TestOverride};
use crate::storage::db::DbPool;

#[test]
fn test_get_settings_defaults_when_empty() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let settings = storage
        .get_settings()
        .expect("should get settings from empty DB");

    assert_eq!(settings, AppSettings::default());
}

#[test]
fn test_seed_settings_idempotent() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let custom = AppSettings {
        response_length: "custom".into(),
        ..Default::default()
    };

    storage
        .seed_settings(&custom)
        .expect("first seed should succeed");
    storage
        .seed_settings(&custom)
        .expect("second seed should succeed (idempotent)");

    let loaded = storage.get_settings().expect("should get settings");
    assert_eq!(loaded.response_length, "custom");
}

#[test]
fn test_save_then_get_settings_roundtrip() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let custom = AppSettings {
        connections: vec![Connection::new(
            "test",
            "Test",
            crate::model::llm_backend::LlmBackendType::OpenRouter,
        )],
        narration_connection_id: "test".into(),
        quantifier_connection_id: "test".into(),
        response_length: "concise".into(),
        text_check: TextCheckSettings {
            mode: TextCheckMode::Spell,
            enable_auto_check: true,
            ignored_words: vec!["foobar".into()],
        },
        agents: vec![AgentConfig {
            name: "TestAgent".into(),
            agent_type: "PreGeneration".into(),
            enabled: true,
            backend: BackendSelector::UseMain,
            phase: ExecutionPhase::PreGeneration,
        }],
        active_system_prompt_preset_id: "preset-sys".into(),
        active_quantifier_prompt_preset_id: "preset-quant".into(),
    };

    storage.save_settings(&custom).expect("should save");
    let loaded = storage.get_settings().expect("should get");

    assert_eq!(loaded.connections.len(), 1);
    assert_eq!(loaded.connections[0].id, "test");
    assert_eq!(loaded.narration_connection_id, "test");
    assert_eq!(loaded.quantifier_connection_id, "test");
    assert_eq!(loaded.response_length, "concise");
    assert_eq!(loaded.text_check.mode, TextCheckMode::Spell);
    assert_eq!(loaded.text_check.ignored_words, vec!["foobar"]);
    assert_eq!(loaded.agents.len(), 1);
    assert_eq!(loaded.agents[0].name, "TestAgent");
    assert_eq!(loaded.active_system_prompt_preset_id, "preset-sys");
    assert_eq!(loaded.active_quantifier_prompt_preset_id, "preset-quant");
}

#[test]
fn test_save_settings_updates_existing() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let defaults = AppSettings::default();
    storage
        .save_settings(&defaults)
        .expect("should save defaults");

    let modified = AppSettings {
        response_length: "updated".into(),
        ..Default::default()
    };
    storage
        .save_settings(&modified)
        .expect("should save modified");

    let loaded = storage.get_settings().expect("should get");
    assert_eq!(loaded.response_length, "updated");
}

#[test]
fn test_get_settings_deserializes_connections_json() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let conn = Connection {
        id: "conn-1".into(),
        name: "Conn 1".into(),
        provider: crate::model::llm_backend::LlmBackendType::Ollama,
        model: "llama3".into(),
        api_key: Some("secret-key".into()),
        base_url: Some("http://custom:11434".into()),
        single_user_message: true,
        max_tokens: Some(2048),
        max_context_tokens: Some(8192),
    };

    let settings = AppSettings {
        connections: vec![conn.clone()],
        ..Default::default()
    };

    storage.save_settings(&settings).expect("should save");
    let loaded = storage.get_settings().expect("should get");

    assert_eq!(loaded.connections.len(), 1);
    let loaded_conn = &loaded.connections[0];
    assert_eq!(loaded_conn.id, "conn-1");
    assert_eq!(loaded_conn.name, "Conn 1");
    assert_eq!(
        loaded_conn.provider,
        crate::model::llm_backend::LlmBackendType::Ollama
    );
    assert_eq!(loaded_conn.model, "llama3");
    assert_eq!(loaded_conn.api_key, Some("secret-key".into()));
    assert_eq!(loaded_conn.base_url, Some("http://custom:11434".into()));
    assert!(loaded_conn.single_user_message);
    assert_eq!(loaded_conn.max_tokens, Some(2048));
    assert_eq!(loaded_conn.max_context_tokens, Some(8192));
}

#[test]
fn test_get_settings_deserializes_text_check_json() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let text_check = TextCheckSettings {
        mode: TextCheckMode::Grammar,
        enable_auto_check: false,
        ignored_words: vec!["word1".into(), "word2".into()],
    };

    let settings = AppSettings {
        text_check: text_check.clone(),
        ..Default::default()
    };

    storage.save_settings(&settings).expect("should save");
    let loaded = storage.get_settings().expect("should get");

    assert_eq!(loaded.text_check.mode, TextCheckMode::Grammar);
    assert!(!loaded.text_check.enable_auto_check);
    assert_eq!(loaded.text_check.ignored_words, vec!["word1", "word2"]);
}

#[test]
fn test_get_settings_deserializes_agents_json() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let agent = AgentConfig {
        name: "MyAgent".into(),
        agent_type: "Custom".into(),
        enabled: false,
        backend: BackendSelector::UseNamed("special-conn".into()),
        phase: ExecutionPhase::PostGeneration,
    };

    let settings = AppSettings {
        agents: vec![agent.clone()],
        ..Default::default()
    };

    storage.save_settings(&settings).expect("should save");
    let loaded = storage.get_settings().expect("should get");

    assert_eq!(loaded.agents.len(), 1);
    let loaded_agent = &loaded.agents[0];
    assert_eq!(loaded_agent.name, "MyAgent");
    assert_eq!(loaded_agent.agent_type, "Custom");
    assert!(!loaded_agent.enabled);
    assert_eq!(
        loaded_agent.backend,
        BackendSelector::UseNamed("special-conn".into())
    );
    assert_eq!(loaded_agent.phase, ExecutionPhase::PostGeneration);
}

#[test]
fn test_get_settings_failure() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1).with_failure(
        Operation::GetSettings,
        TestOverride::internal("test failure"),
    );

    let result = storage.get_settings();
    assert!(
        result.is_err(),
        "get_settings should fail with test override"
    );
}

#[test]
fn test_save_settings_failure() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1).with_failure(
        Operation::SaveSettings,
        TestOverride::internal("test failure"),
    );

    let settings = AppSettings::default();
    let result = storage.save_settings(&settings);
    assert!(
        result.is_err(),
        "save_settings should fail with test override"
    );
}

#[test]
fn test_settings_table_singleton_constraint() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);

    let settings1 = AppSettings {
        response_length: "first".into(),
        ..Default::default()
    };
    let settings2 = AppSettings {
        response_length: "second".into(),
        ..Default::default()
    };

    // Save twice - the second should REPLACE the first due to CHECK (id = 1) constraint
    storage
        .save_settings(&settings1)
        .expect("first save should succeed");
    storage
        .save_settings(&settings2)
        .expect("second save should succeed (REPLACE)");

    // Verify only the second value persists (singleton behavior)
    let loaded = storage.get_settings().expect("should get settings");
    assert_eq!(
        loaded.response_length, "second",
        "should have the second value (first was replaced)"
    );

    // Save a third time to confirm continued singleton behavior
    let settings3 = AppSettings {
        response_length: "third".into(),
        ..Default::default()
    };
    storage
        .save_settings(&settings3)
        .expect("third save should succeed");
    let loaded2 = storage.get_settings().expect("should get settings");
    assert_eq!(
        loaded2.response_length, "third",
        "singleton constraint maintained across multiple saves"
    );
}
