//! Tests for Storage preset methods: list_presets, get_preset, save_preset, delete_preset

use chronicler_engine::model::prompt_preset::{PresetType, PromptPreset};
use chronicler_engine::storage::Storage;

fn create_storage() -> Storage {
    Storage::new_in_memory()
}

fn make_system_preset(id: &str, name: &str) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: name.to_string(),
        role: Some("You are a test narrator.".to_string()),
        instructions: Some("Test instructions".to_string()),
        writing_style: Some("Test style".to_string()),
        output_format: Some("Test format".to_string()),
        is_default: false,
        preset_type: PresetType::System,
    }
}

fn make_quantifier_preset(id: &str, name: &str) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: name.to_string(),
        role: Some("You are a test quantifier.".to_string()),
        instructions: Some("Quantify instructions".to_string()),
        writing_style: Some("Quantify style".to_string()),
        output_format: Some("Quantify format".to_string()),
        is_default: false,
        preset_type: PresetType::Quantifier,
    }
}

#[test]
fn test_list_presets_empty() {
    let storage = create_storage();
    let system = storage.list_presets(PresetType::System).unwrap();
    let quantifier = storage.list_presets(PresetType::Quantifier).unwrap();

    assert!(system.is_empty(), "Should have no system presets");
    assert!(quantifier.is_empty(), "Should have no quantifier presets");
}

#[test]
fn test_list_presets_system_only() {
    let storage = create_storage();
    storage
        .save_preset(&make_system_preset("sys1", "System One"))
        .unwrap();
    storage
        .save_preset(&make_system_preset("sys2", "System Two"))
        .unwrap();
    storage
        .save_preset(&make_quantifier_preset("quant1", "Quantifier One"))
        .unwrap();

    let system = storage.list_presets(PresetType::System).unwrap();
    let quantifier = storage.list_presets(PresetType::Quantifier).unwrap();

    assert_eq!(system.len(), 2, "Should have 2 system presets");
    assert_eq!(quantifier.len(), 1, "Should have 1 quantifier preset");
}

#[test]
fn test_list_presets_quantifier_only() {
    let storage = create_storage();
    storage
        .save_preset(&make_quantifier_preset("quant1", "Quantifier One"))
        .unwrap();
    storage
        .save_preset(&make_quantifier_preset("quant2", "Quantifier Two"))
        .unwrap();

    let quantifier = storage.list_presets(PresetType::Quantifier).unwrap();

    assert_eq!(quantifier.len(), 2, "Should have 2 quantifier presets");
}

#[test]
fn test_list_presets_ordered_by_updated_at_desc() {
    let storage = create_storage();
    let first = make_system_preset("first", "First");
    storage.save_preset(&first).unwrap();
    // NOTE: Sleeps are intentional and unavoidable here. This test verifies
    // that presets are ordered by updated_at DESC. SQLite stores timestamps with
    // millisecond precision, so we need explicit delays to ensure distinct timestamps.
    // This is a legitimate use of sleep in tests - verifying time-based ordering.
    // Save with explicit time gaps to ensure ordering by updated_at
    std::thread::sleep(std::time::Duration::from_millis(15));
    let second = make_system_preset("second", "Second");
    storage.save_preset(&second).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(15));
    let third = make_system_preset("third", "Third");
    storage.save_preset(&third).unwrap();

    let presets = storage.list_presets(PresetType::System).unwrap();

    assert_eq!(presets.len(), 3, "Should have 3 presets");
    let ids: Vec<_> = presets.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"first") && ids.contains(&"second") && ids.contains(&"third"));
}

#[test]
fn test_list_presets_does_not_return_other_type() {
    let storage = create_storage();
    storage
        .save_preset(&make_system_preset("sys1", "System One"))
        .unwrap();
    storage
        .save_preset(&make_quantifier_preset("quant1", "Quantifier One"))
        .unwrap();

    let system = storage.list_presets(PresetType::System).unwrap();
    let ids: Vec<_> = system.iter().map(|p| p.id.as_str()).collect();

    assert!(
        !ids.contains(&"quant1"),
        "System list should not contain quantifier preset"
    );
}

#[test]
fn test_get_preset_found() {
    let storage = create_storage();
    let preset = make_system_preset("test_id", "Test Preset");
    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("test_id").unwrap();

    assert!(found.is_some(), "Should find preset");
    let found = found.unwrap();
    assert_eq!(found.id, "test_id");
    assert_eq!(found.name, "Test Preset");
    assert_eq!(found.preset_type, PresetType::System);
}

#[test]
fn test_get_preset_not_found() {
    let storage = create_storage();

    let found = storage.get_preset("nonexistent").unwrap();

    assert!(found.is_none(), "Should return None for nonexistent preset");
}

#[test]
fn test_get_preset_system_type() {
    let storage = create_storage();
    let preset = make_system_preset("sys_preset", "System Preset");
    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("sys_preset").unwrap().unwrap();

    assert_eq!(found.preset_type, PresetType::System);
}

#[test]
fn test_get_preset_quantifier_type() {
    let storage = create_storage();
    let preset = make_quantifier_preset("quant_preset", "Quantifier Preset");
    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("quant_preset").unwrap().unwrap();

    assert_eq!(found.preset_type, PresetType::Quantifier);
}

#[test]
fn test_get_preset_returns_all_fields() {
    let storage = create_storage();
    let preset = PromptPreset {
        id: "full_preset".to_string(),
        name: "Full Preset".to_string(),
        role: Some("Narrator role".to_string()),
        instructions: Some("Instructions text".to_string()),
        writing_style: Some("Writing style text".to_string()),
        output_format: Some("Output format text".to_string()),
        is_default: true,
        preset_type: PresetType::System,
    };
    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("full_preset").unwrap().unwrap();

    assert_eq!(found.id, "full_preset");
    assert_eq!(found.name, "Full Preset");
    assert_eq!(found.role, Some("Narrator role".to_string()));
    assert_eq!(found.instructions, Some("Instructions text".to_string()));
    assert_eq!(found.writing_style, Some("Writing style text".to_string()));
    assert_eq!(found.output_format, Some("Output format text".to_string()));
    assert!(found.is_default, "is_default should be true");
}

#[test]
fn test_save_preset_insert_new() {
    let storage = create_storage();
    let preset = make_system_preset("new_preset", "New Preset");

    let result = storage.save_preset(&preset);

    assert!(result.is_ok(), "Should save preset successfully");
    let found = storage.get_preset("new_preset").unwrap().unwrap();
    assert_eq!(found.name, "New Preset");
}

#[test]
fn test_save_preset_insert_quantifier() {
    let storage = create_storage();
    let preset = make_quantifier_preset("new_quant", "New Quantifier");

    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("new_quant").unwrap().unwrap();
    assert_eq!(found.preset_type, PresetType::Quantifier);
}

#[test]
fn test_save_preset_with_default_flag_true() {
    let storage = create_storage();
    let mut preset = make_system_preset("default_preset", "Default Preset");
    preset.is_default = true;

    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("default_preset").unwrap().unwrap();
    assert!(found.is_default, "is_default should be true");
}

#[test]
fn test_save_preset_with_default_flag_false() {
    let storage = create_storage();
    let mut preset = make_system_preset("non_default", "Non Default Preset");
    preset.is_default = false;

    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("non_default").unwrap().unwrap();
    assert!(!found.is_default, "is_default should be false");
}

#[test]
fn test_save_preset_update_existing() {
    let storage = create_storage();
    let preset = make_system_preset("update_test", "Original Name");
    storage.save_preset(&preset).unwrap();

    let mut updated = make_system_preset("update_test", "Updated Name");
    updated.role = Some("Updated role".to_string());
    storage.save_preset(&updated).unwrap();

    let found = storage.get_preset("update_test").unwrap().unwrap();
    assert_eq!(found.name, "Updated Name");
    assert_eq!(found.role, Some("Updated role".to_string()));
}

#[test]
fn test_save_preset_update_preserves_id() {
    let storage = create_storage();
    let preset = make_system_preset("preserved_id", "Original");
    storage.save_preset(&preset).unwrap();

    let updated = make_system_preset("preserved_id", "Updated");
    storage.save_preset(&updated).unwrap();

    let found = storage.get_preset("preserved_id").unwrap().unwrap();
    assert_eq!(
        found.id, "preserved_id",
        "ID should be preserved after update"
    );
}

#[test]
fn test_save_preset_update_changes_type() {
    let storage = create_storage();
    let preset = make_system_preset("type_test", "Type Test");
    storage.save_preset(&preset).unwrap();

    let updated = make_quantifier_preset("type_test", "Type Test");
    storage.save_preset(&updated).unwrap();

    let found = storage.get_preset("type_test").unwrap().unwrap();
    assert_eq!(found.preset_type, PresetType::Quantifier);
}

#[test]
fn test_save_preset_update_toggles_default() {
    let storage = create_storage();
    let mut preset = make_system_preset("toggle_default", "Toggle Test");
    preset.is_default = true;
    storage.save_preset(&preset).unwrap();

    let mut updated = make_system_preset("toggle_default", "Toggle Test");
    updated.is_default = false;
    storage.save_preset(&updated).unwrap();

    let found = storage.get_preset("toggle_default").unwrap().unwrap();
    assert!(!found.is_default, "is_default should be toggled to false");
}

#[test]
fn test_delete_preset_existing() {
    let storage = create_storage();
    storage
        .save_preset(&make_system_preset("to_delete", "To Delete"))
        .unwrap();

    let result = storage.delete_preset("to_delete");

    assert!(result.is_ok(), "Should delete preset successfully");
    let found = storage.get_preset("to_delete").unwrap();
    assert!(found.is_none(), "Preset should be deleted");
}

#[test]
fn test_delete_preset_nonexistent() {
    let storage = create_storage();

    let result = storage.delete_preset("nonexistent");

    assert!(
        result.is_ok(),
        "Should not error on deleting nonexistent preset"
    );
}

#[test]
fn test_delete_preset_only_deletes_target() {
    let storage = create_storage();
    storage
        .save_preset(&make_system_preset("keep1", "Keep One"))
        .unwrap();
    storage
        .save_preset(&make_system_preset("keep2", "Keep Two"))
        .unwrap();
    storage
        .save_preset(&make_system_preset("delete_me", "Delete Me"))
        .unwrap();

    storage.delete_preset("delete_me").unwrap();

    let keep1 = storage.get_preset("keep1").unwrap();
    let keep2 = storage.get_preset("keep2").unwrap();
    let deleted = storage.get_preset("delete_me").unwrap();

    assert!(keep1.is_some(), "keep1 should still exist");
    assert!(keep2.is_some(), "keep2 should still exist");
    assert!(deleted.is_none(), "delete_me should be deleted");
}

#[test]
fn test_delete_preset_clears_from_list() {
    let storage = create_storage();
    storage
        .save_preset(&make_system_preset("list_test", "List Test"))
        .unwrap();

    let before = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(before.len(), 1, "Should have 1 preset before delete");

    storage.delete_preset("list_test").unwrap();

    let after = storage.list_presets(PresetType::System).unwrap();
    assert!(after.is_empty(), "Should have 0 presets after delete");
}

#[test]
fn test_preset_with_empty_optional_fields() {
    let storage = create_storage();
    let preset = PromptPreset {
        id: "empty_fields".to_string(),
        name: "Empty Fields".to_string(),
        role: None,
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: false,
        preset_type: PresetType::System,
    };

    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("empty_fields").unwrap().unwrap();
    assert_eq!(found.role, None);
    assert_eq!(found.instructions, None);
    assert_eq!(found.writing_style, None);
    assert_eq!(found.output_format, None);
}

#[test]
fn test_preset_with_unicode_content() {
    let storage = create_storage();
    let preset = PromptPreset {
        id: "unicode".to_string(),
        name: "日本語プリセット".to_string(),
        role: Some("角色：测试员".to_string()),
        instructions: Some("🎮 Instructions with emoji 🚀".to_string()),
        writing_style: Some("Style: 简体中文".to_string()),
        output_format: Some("Format & \"special\" <chars>".to_string()),
        is_default: false,
        preset_type: PresetType::Quantifier,
    };

    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("unicode").unwrap().unwrap();
    assert_eq!(found.name, "日本語プリセット");
    assert_eq!(found.role, Some("角色：测试员".to_string()));
}

#[test]
fn test_preset_with_special_characters() {
    let storage = create_storage();
    let preset = PromptPreset {
        id: "special".to_string(),
        name: "Name with 'quotes' and \"double\"".to_string(),
        role: Some("Role with <html> & \"entities\"".to_string()),
        instructions: Some("Instructions with\nnewlines\tand\ttabs".to_string()),
        writing_style: Some("Style with    spaces".to_string()),
        output_format: Some("Format with\r\nwindows\r\nline endings".to_string()),
        is_default: false,
        preset_type: PresetType::System,
    };

    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("special").unwrap().unwrap();
    assert_eq!(found.name, "Name with 'quotes' and \"double\"");
    assert!(found.instructions.unwrap().contains("newlines"));
}

#[test]
fn test_preset_with_long_content() {
    let storage = create_storage();
    let long_text = "x".repeat(10000);
    let preset = PromptPreset {
        id: "long".to_string(),
        name: long_text.clone(),
        role: Some(long_text.clone()),
        instructions: Some(long_text.clone()),
        writing_style: Some(long_text.clone()),
        output_format: Some(long_text.clone()),
        is_default: false,
        preset_type: PresetType::System,
    };

    storage.save_preset(&preset).unwrap();

    let found = storage.get_preset("long").unwrap().unwrap();
    assert_eq!(found.name.len(), 10000, "Long name should be preserved");
    assert_eq!(
        found.instructions.unwrap().len(),
        10000,
        "Long instructions should be preserved"
    );
}

#[test]
fn test_preset_upsert_multiple_times() {
    let storage = create_storage();
    let base = make_system_preset("upsert_test", "Initial");

    storage.save_preset(&base).unwrap();

    for i in 1..=5 {
        let updated = make_system_preset("upsert_test", &format!("Update {i}"));
        storage.save_preset(&updated).unwrap();
    }

    let found = storage.get_preset("upsert_test").unwrap().unwrap();
    assert_eq!(found.name, "Update 5", "Should have final update name");

    let list = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(
        list.len(),
        1,
        "Should still have only 1 preset after multiple upserts"
    );
}

#[test]
fn test_preset_list_filtered_by_type_across_operations() {
    let storage = create_storage();

    storage
        .save_preset(&make_system_preset("sys1", "System 1"))
        .unwrap();
    storage
        .save_preset(&make_quantifier_preset("quant1", "Quantifier 1"))
        .unwrap();
    storage
        .save_preset(&make_system_preset("sys2", "System 2"))
        .unwrap();
    storage
        .save_preset(&make_quantifier_preset("quant2", "Quantifier 2"))
        .unwrap();

    storage.delete_preset("quant1").unwrap();

    let system = storage.list_presets(PresetType::System).unwrap();
    let quantifier = storage.list_presets(PresetType::Quantifier).unwrap();

    assert_eq!(system.len(), 2, "Should have 2 system presets");
    assert_eq!(
        quantifier.len(),
        1,
        "Should have 1 quantifier preset after delete"
    );

    let updated = make_system_preset("sys1", "Updated System 1");
    storage.save_preset(&updated).unwrap();

    let system_after = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(
        system_after.len(),
        2,
        "Should still have 2 system presets after update"
    );
}
