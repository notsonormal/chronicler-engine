use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::storage::backend::{Storage, TestOverride};

fn dummy_preset(id: &str, preset_type: PresetType) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: id.to_string(),
        preset_type,
        role: Some("role".to_string()),
        instructions: Some("instructions".to_string()),
        writing_style: None,
        output_format: None,
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
fn test_list_presets_quantifier_only() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("q1", PresetType::Quantifier))
        .unwrap();

    let quantifier = storage.list_presets(PresetType::Quantifier).unwrap();
    assert_eq!(quantifier.len(), 1);
    assert_eq!(quantifier[0].id, "q1");
}

#[test]
fn test_get_preset_found() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("p1", PresetType::System))
        .unwrap();

    let loaded = storage.get_preset("p1").unwrap().unwrap();
    assert_eq!(loaded.id, "p1");
}

#[test]
fn test_get_preset_not_found() {
    let storage = Storage::new_in_memory();
    let loaded = storage.get_preset("nonexistent").unwrap();
    assert!(loaded.is_none());
}

#[test]
fn test_get_preset_system_type() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("sys1", PresetType::System))
        .unwrap();

    let loaded = storage.get_preset("sys1").unwrap().unwrap();
    assert_eq!(loaded.preset_type, PresetType::System);
}

#[test]
fn test_get_preset_quantifier_type() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("quant1", PresetType::Quantifier))
        .unwrap();

    let loaded = storage.get_preset("quant1").unwrap().unwrap();
    assert_eq!(loaded.preset_type, PresetType::Quantifier);
}

#[test]
fn test_save_preset_insert_new() {
    let storage = Storage::new_in_memory();
    let preset = dummy_preset("new1", PresetType::System);
    storage.save_preset(&preset).unwrap();

    let loaded = storage.get_preset("new1").unwrap().unwrap();
    assert_eq!(loaded.id, "new1");
}

#[test]
fn test_save_preset_with_default_flag() {
    let storage = Storage::new_in_memory();
    let mut preset = dummy_preset("default_test", PresetType::System);
    preset.is_default = true;
    storage.save_preset(&preset).unwrap();

    let loaded = storage.get_preset("default_test").unwrap().unwrap();
    assert!(loaded.is_default);
}

#[test]
fn test_save_preset_update_existing() {
    let storage = Storage::new_in_memory();
    let mut preset = dummy_preset("update1", PresetType::System);
    preset.name = "Original".to_string();
    storage.save_preset(&preset).unwrap();

    preset.name = "Updated".to_string();
    storage.save_preset(&preset).unwrap();

    let loaded = storage.get_preset("update1").unwrap().unwrap();
    assert_eq!(loaded.name, "Updated");
}

#[test]
fn test_delete_preset_existing() {
    let storage = Storage::new_in_memory();
    storage
        .save_preset(&dummy_preset("del1", PresetType::System))
        .unwrap();
    storage.delete_preset("del1").unwrap();

    let loaded = storage.get_preset("del1").unwrap();
    assert!(loaded.is_none());
}

#[test]
fn test_delete_preset_nonexistent() {
    let storage = Storage::new_in_memory();
    let result = storage.delete_preset("nonexistent");
    assert!(result.is_ok());
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
