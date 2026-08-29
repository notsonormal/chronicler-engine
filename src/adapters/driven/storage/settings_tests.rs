use crate::domain::model::{
    agent::{AgentConfig, BackendSelector, ExecutionPhase},
    settings::{AppSettings, LlmProviderConfig, NarratorMode, TextCheckMode, TextCheckSettings},
};
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::adapters::driven::storage::db::DbPool;

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

    let mut custom = AppSettings {
        connections: vec![LlmProviderConfig::new(
            "test",
            "Test",
            crate::domain::model::llm_backend::LlmBackendType::OpenRouter,
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
        ..Default::default()
    };
    let mut novel = custom.mode_preset_registry.bundle_for(NarratorMode::Novel);
    novel.system_prompt_preset_id = "preset-sys".into();
    novel.quantifier_prompt_preset_id = "preset-quant".into();
    novel.impersonate_prompt_preset_id = "preset-imp".into();
    custom.mode_preset_registry.set_bundle(novel);

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
    assert_eq!(
        loaded
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "preset-sys"
    );
    assert_eq!(
        loaded
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .quantifier_prompt_preset_id,
        "preset-quant"
    );
    assert_eq!(
        loaded
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .impersonate_prompt_preset_id,
        "preset-imp"
    );
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

    let conn = LlmProviderConfig {
        id: "conn-1".into(),
        name: "Conn 1".into(),
        provider: crate::domain::model::llm_backend::LlmBackendType::Ollama,
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
        crate::domain::model::llm_backend::LlmBackendType::Ollama
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
    let storage = Storage::new_sqlite(pool, 1)
        .with_failure("get_settings", TestOverride::internal("test failure"));

    let result = storage.get_settings();
    assert!(
        result.is_err(),
        "get_settings should fail with test override"
    );
}

#[test]
fn test_save_settings_failure() {
    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1)
        .with_failure("save_settings", TestOverride::internal("test failure"));

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

    storage
        .save_settings(&settings1)
        .expect("first save should succeed");
    storage
        .save_settings(&settings2)
        .expect("second save should succeed (REPLACE)");

    let loaded = storage.get_settings().expect("should get settings");
    assert_eq!(
        loaded.response_length, "second",
        "should have the second value (first was replaced)"
    );

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

#[test]
fn test_migration_v20_reshapes_registry_and_backfills_flags() {
    use crate::adapters::driven::storage::utils::run_migrations;
    use crate::domain::model::settings::ModePresetRegistry;

    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();

    // Simulate a v19 database: object-keyed registry with user-customized
    // novel-bundle ids, plus an existing system_default preset row (whose
    // allowed_modes column is born all-allowed).
    conn.execute_batch(
        "INSERT INTO settings (id, created_at, updated_at, mode_preset_registry) \
         VALUES (1, 't', 't', '{\"novel\":{\"system_prompt_preset_id\":\"custom_sys\",\"quantifier_prompt_preset_id\":\"custom_quant\",\"impersonate_prompt_preset_id\":\"custom_imp\"},\"interactive_fiction\":{\"system_prompt_preset_id\":\"system_if_default\",\"quantifier_prompt_preset_id\":\"quantifier_default\",\"impersonate_prompt_preset_id\":\"impersonate_default\"}}'); \
         INSERT INTO prompt_presets (id, name, preset_type, is_default, created_at, updated_at) \
         VALUES ('system_default', 'Default', 'system', 1, 't', 't'); \
         PRAGMA user_version = 19;",
    )
    .unwrap();

    run_migrations(&conn).unwrap();

    // Registry reshaped to the mode-tagged list; user values preserved.
    let registry_json: String = conn
        .query_row(
            "SELECT mode_preset_registry FROM settings WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let registry: ModePresetRegistry = serde_json::from_str(&registry_json).unwrap();
    assert_eq!(registry.0.len(), 2);
    let novel = registry.bundle_for(NarratorMode::Novel);
    assert_eq!(novel.mode, NarratorMode::Novel);
    assert_eq!(novel.system_prompt_preset_id, "custom_sys");
    assert_eq!(novel.quantifier_prompt_preset_id, "custom_quant");
    assert_eq!(novel.impersonate_prompt_preset_id, "custom_imp");
    let if_bundle = registry.bundle_for(NarratorMode::InteractiveFiction);
    assert_eq!(if_bundle.system_prompt_preset_id, "system_if_default");

    // Flags: column born all-allowed, system_default tightened to novel-only.
    let allowed: String = conn
        .query_row(
            "SELECT allowed_modes FROM prompt_presets WHERE id = 'system_default'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(allowed, "[\"novel\"]");

    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 20);
}

#[test]
fn test_corrupt_mode_preset_registry_in_db_errors_as_parse() {
    use crate::error::EngineError;

    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool.clone(), 1);
    storage
        .seed_settings(&AppSettings::default())
        .expect("seeding default settings should succeed");
    pool.conn()
        .execute(
            "UPDATE settings SET mode_preset_registry = 'not-json' WHERE id = 1",
            [],
        )
        .unwrap();

    match storage.get_settings() {
        Err(EngineError::Parse(_)) => {}
        other => panic!("expected EngineError::Parse, got {other:?}"),
    }
}
