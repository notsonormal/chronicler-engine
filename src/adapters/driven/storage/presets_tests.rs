use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::adapters::driven::storage::{Storage, TestOverride};

fn dummy_preset(id: &str, preset_type: PresetType) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: id.to_string(),
        preset_type,
        role: Some("role".to_string()),
        instructions: Some("instructions".to_string()),
        writing_style: None,
        output_format: None,
        allowed_modes: crate::domain::model::utils::settings_defaults::default_allowed_modes(),
        is_default: false,
    }
}

#[test]
fn test_list_presets_filters_by_type() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("s1", PresetType::System))
        .unwrap();
    storage
        .save_preset(&dummy_preset("q1", PresetType::Quantifier))
        .unwrap();

    let system = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(system.len(), 1);
    assert_eq!(system[0].id, "s1");
}

#[test]
fn test_list_presets_quantifier_returns_correct_id() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("q1", PresetType::Quantifier))
        .unwrap();

    let quantifier = storage.list_presets(PresetType::Quantifier).unwrap();
    assert_eq!(quantifier.len(), 1);
    assert_eq!(quantifier[0].id, "q1");
}

#[test]
fn test_get_preset_not_found() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("p1", PresetType::System))
        .unwrap();

    let loaded = storage.get_preset("p1").unwrap().unwrap();
    assert_eq!(loaded.id, "p1");
}

#[test]
fn test_list_presets_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("list_presets", TestOverride::internal("list failed"));

    let result = storage.list_presets(PresetType::System);
    assert!(result.is_err());
}

#[test]
fn test_save_preset_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("save_preset", TestOverride::config("save failed"));

    let result = storage.save_preset(&dummy_preset("fail", PresetType::System));
    assert!(result.is_err());
}

#[test]
fn test_get_preset_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("get_preset", TestOverride::internal("get failed"));

    let result = storage.get_preset("any");
    assert!(result.is_err());
}

#[test]
fn test_delete_preset_failure() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    handle.set("delete_preset", TestOverride::config("delete failed"));

    let result = storage.delete_preset("any");
    assert!(result.is_err());
}

#[test]
fn test_allowed_modes_round_trip_sqlite() {
    use crate::adapters::driven::storage::db::DbPool;
    use crate::domain::model::settings::NarratorMode;

    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool, 1);
    let mut preset = dummy_preset("m1", PresetType::System);
    preset.allowed_modes = vec![NarratorMode::Novel];
    storage.save_preset(&preset).unwrap();

    let loaded = storage.get_preset("m1").unwrap().unwrap();
    assert_eq!(loaded.allowed_modes, vec![NarratorMode::Novel]);
}

#[test]
fn test_corrupt_allowed_modes_in_db_errors_as_parse() {
    use crate::adapters::driven::storage::db::DbPool;
    use crate::error::EngineError;

    let pool = DbPool::new(":memory:").unwrap();
    let storage = Storage::new_sqlite(pool.clone(), 1);
    storage
        .save_preset(&dummy_preset("m2", PresetType::System))
        .unwrap();
    pool.conn()
        .execute(
            "UPDATE prompt_presets SET allowed_modes = 'not-json' WHERE id = 'm2'",
            [],
        )
        .unwrap();

    match storage.get_preset("m2") {
        Err(EngineError::Parse(_)) => {}
        other => panic!("expected EngineError::Parse, got {other:?}"),
    }
}
