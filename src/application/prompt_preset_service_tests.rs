//! Tests for `PromptPresetService`.

use std::sync::Arc;

use crate::application::errors::ApplicationError;
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

fn system_preset(id: &str, name: &str) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: name.to_string(),
        preset_type: PresetType::System,
        ..Default::default()
    }
}

#[test]
fn test_save_preset_refuses_duplicate_name_in_same_category() {
    let (service, _storage) = make_service();
    service.save_preset(&system_preset("one", "Alpha")).unwrap();

    let error = service
        .save_preset(&system_preset("two", "Alpha"))
        .expect_err("a duplicate name in the same category must be refused");

    assert!(matches!(error, ApplicationError::Validation(_)));
}

#[test]
fn test_save_preset_refuses_case_and_space_variant() {
    let (service, _storage) = make_service();
    service.save_preset(&system_preset("one", "Alpha")).unwrap();

    let error = service
        .save_preset(&system_preset("two", "  alpha  "))
        .expect_err("a name differing only in case and space must be refused");

    assert!(matches!(error, ApplicationError::Validation(_)));
}

#[test]
fn test_save_preset_allows_same_name_in_another_category() {
    let (service, _storage) = make_service();
    service.save_preset(&system_preset("one", "Alpha")).unwrap();

    let mut quantifier = system_preset("two", "Alpha");
    quantifier.preset_type = PresetType::Quantifier;
    service
        .save_preset(&quantifier)
        .expect("a different category may reuse the name");
}

#[test]
fn test_save_preset_update_keeps_own_name() {
    let (service, storage) = make_service();
    service.save_preset(&system_preset("one", "Alpha")).unwrap();

    let mut updated = system_preset("one", "Alpha");
    updated.instructions = Some("Changed.".to_string());
    service
        .save_preset(&updated)
        .expect("keeping the preset's own name must succeed");

    let stored = storage.get_preset("one").unwrap().unwrap();
    assert_eq!(stored.instructions.as_deref(), Some("Changed."));
}

#[test]
fn test_next_copy_name_skips_taken_names() {
    let (service, _storage) = make_service();
    let source = system_preset("source", "Alpha");
    service.save_preset(&source).unwrap();

    assert_eq!(service.next_copy_name(&source).unwrap(), "Alpha (Copy)");

    service
        .save_preset(&system_preset("copy-1", "Alpha (Copy)"))
        .unwrap();
    assert_eq!(service.next_copy_name(&source).unwrap(), "Alpha (Copy 2)");

    service
        .save_preset(&system_preset("copy-2", "Alpha (Copy 2)"))
        .unwrap();
    assert_eq!(service.next_copy_name(&source).unwrap(), "Alpha (Copy 3)");
}
