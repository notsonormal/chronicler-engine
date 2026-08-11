//! Tests for `PromptPresetService`.

use std::sync::Arc;

use crate::application::prompt_preset_service::PromptPresetService;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};

type Storage = crate::adapters::driven::storage::Storage;

fn make_service() -> (PromptPresetService, Arc<Storage>) {
    let storage = Arc::new(Storage::new_in_memory());
    let service = PromptPresetService::new(Arc::clone(&storage));
    (service, storage)
}

#[test]
fn test_preset_crud_roundtrip() {
    let (service, _storage) = make_service();
    let preset = PromptPreset {
        id: "preset-1".to_string(),
        name: "Test Preset".to_string(),
        role: Some("Test role".to_string()),
        instructions: Some("Test instructions".to_string()),
        preset_type: PresetType::System,
        ..Default::default()
    };

    assert!(service.get_preset("preset-1").unwrap().is_none());

    service.save_preset(&preset).unwrap();
    let loaded = service.get_preset("preset-1").unwrap().unwrap();
    assert_eq!(loaded.name, "Test Preset");

    let system_presets = service.list_presets(PresetType::System).unwrap();
    assert_eq!(system_presets.len(), 1);
    assert_eq!(system_presets[0].id, "preset-1");

    let quantifier_presets = service.list_presets(PresetType::Quantifier).unwrap();
    assert!(quantifier_presets.is_empty());

    service.delete_preset("preset-1").unwrap();
    assert!(service.get_preset("preset-1").unwrap().is_none());
}
