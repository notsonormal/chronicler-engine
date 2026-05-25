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

fn preset(id: &str, name: &str, instructions: &str, preset_type: PresetType) -> PromptPreset {
    PromptPreset {
        id: id.into(),
        name: name.into(),
        instructions: Some(instructions.into()),
        preset_type,
        ..Default::default()
    }
}

#[test]
fn test_sqlite_save_and_get() {
    let storage = create_sqlite_storage();
    let preset = preset("test-1", "Test", "Hello.", PresetType::System);

    storage.save(&preset).unwrap();
    let result = storage.get("test-1").unwrap();

    assert!(result.is_some());
    let fetched = result.unwrap();
    assert_eq!(fetched.id, "test-1");
    assert_eq!(fetched.name, "Test");
    assert_eq!(fetched.instructions, Some("Hello.".into()));
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
    let system = preset("sys-1", "System", "Sys.", PresetType::System);
    let quantifier = preset("quant-1", "Quantifier", "Quant.", PresetType::Quantifier);

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
    let original = preset("update-1", "Original", "Original text.", PresetType::System);
    storage.save(&original).unwrap();

    let mut updated = preset(
        "update-1",
        "Updated",
        "Updated text.",
        PresetType::Quantifier,
    );
    updated.is_default = true;
    storage.save(&updated).unwrap();

    let result = storage.get("update-1").unwrap().unwrap();
    assert_eq!(result.name, "Updated");
    assert_eq!(result.instructions, Some("Updated text.".into()));
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
    let preset = preset("del-1", "To Delete", "Bye.", PresetType::System);
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
        role: None,
        instructions: Some("T".into()),
        writing_style: None,
        output_format: None,
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
        role: None,
        instructions: Some("T".into()),
        writing_style: None,
        output_format: None,
        is_default: 0,
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let preset = from_db(db_non_default);
    assert!(!preset.is_default);
}
