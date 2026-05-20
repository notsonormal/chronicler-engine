use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::storage::db::DbPool;
use crate::storage::models::prompt_preset::DbPromptPreset;
use crate::storage::prompt_preset_storage::{
    PromptPresetStorage, SqlitePromptPresetStorage, from_db,
};

fn create_sqlite_storage() -> SqlitePromptPresetStorage {
    let pool = DbPool::new(":memory:").unwrap();
    SqlitePromptPresetStorage::new(pool)
}

#[test]
fn test_sqlite_save_and_get() {
    let storage = create_sqlite_storage();
    let preset = PromptPreset {
        id: "test-1".into(),
        name: "Test".into(),
        prompt_text: "Hello.".into(),
        is_default: false,
        preset_type: PresetType::System,
    };

    storage.save(&preset).unwrap();
    let result = storage.get("test-1").unwrap();

    assert!(result.is_some());
    let fetched = result.unwrap();
    assert_eq!(fetched.id, "test-1");
    assert_eq!(fetched.name, "Test");
    assert_eq!(fetched.prompt_text, "Hello.");
    assert!(!fetched.is_default);
}

#[test]
fn test_sqlite_get_nonexistent() {
    let storage = create_sqlite_storage();
    let result = storage.get("missing").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_sqlite_list_filters_by_type() {
    let storage = create_sqlite_storage();
    let system = PromptPreset {
        id: "sys-1".into(),
        name: "System".into(),
        prompt_text: "Sys.".into(),
        is_default: false,
        preset_type: PresetType::System,
    };
    let quantifier = PromptPreset {
        id: "quant-1".into(),
        name: "Quantifier".into(),
        prompt_text: "Quant.".into(),
        is_default: false,
        preset_type: PresetType::Quantifier,
    };

    storage.save(&system).unwrap();
    storage.save(&quantifier).unwrap();

    let system_list = storage.list(PresetType::System).unwrap();
    assert_eq!(system_list.len(), 1);
    assert_eq!(system_list[0].id, "sys-1");

    let quant_list = storage.list(PresetType::Quantifier).unwrap();
    assert_eq!(quant_list.len(), 1);
    assert_eq!(quant_list[0].id, "quant-1");
}

#[test]
fn test_sqlite_save_updates_existing() {
    let storage = create_sqlite_storage();
    let preset = PromptPreset {
        id: "update-1".into(),
        name: "Original".into(),
        prompt_text: "Original text.".into(),
        is_default: false,
        preset_type: PresetType::System,
    };
    storage.save(&preset).unwrap();

    let updated = PromptPreset {
        id: "update-1".into(),
        name: "Updated".into(),
        prompt_text: "Updated text.".into(),
        is_default: true,
        preset_type: PresetType::Quantifier,
    };
    storage.save(&updated).unwrap();

    let result = storage.get("update-1").unwrap().unwrap();
    assert_eq!(result.name, "Updated");
    assert_eq!(result.prompt_text, "Updated text.");
    assert!(result.is_default);

    // Type should also have changed
    let system_list = storage.list(PresetType::System).unwrap();
    assert!(system_list.is_empty());
    let quant_list = storage.list(PresetType::Quantifier).unwrap();
    assert_eq!(quant_list.len(), 1);
}

#[test]
fn test_sqlite_delete() {
    let storage = create_sqlite_storage();
    let preset = PromptPreset {
        id: "del-1".into(),
        name: "To Delete".into(),
        prompt_text: "Bye.".into(),
        is_default: false,
        preset_type: PresetType::System,
    };
    storage.save(&preset).unwrap();
    assert!(storage.get("del-1").unwrap().is_some());

    storage.delete("del-1").unwrap();
    assert!(storage.get("del-1").unwrap().is_none());
}

#[test]
fn test_from_db_maps_is_default() {
    let db_default = DbPromptPreset {
        id: "d".into(),
        name: "D".into(),
        preset_type: "system".into(),
        prompt_text: "T".into(),
        is_default: 1,
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let preset = from_db(db_default);
    assert!(preset.is_default);

    let db_non_default = DbPromptPreset {
        id: "nd".into(),
        name: "ND".into(),
        preset_type: "quantifier".into(),
        prompt_text: "T".into(),
        is_default: 0,
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let preset = from_db(db_non_default);
    assert!(!preset.is_default);
}
