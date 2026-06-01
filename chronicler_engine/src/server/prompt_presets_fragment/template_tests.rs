use askama::Template;

use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::server::prompt_presets_fragment::PromptPresetsTemplate;

#[test]
fn test_prompt_presets_template_renders_system_presets() {
    let template = PromptPresetsTemplate {
        system_presets: vec![
            PromptPreset {
                id: "default".into(),
                name: "Default".into(),
                instructions: Some("You are a narrator.".into()),
                is_default: true,
                preset_type: PresetType::System,
                ..Default::default()
            },
            PromptPreset {
                id: "custom-1".into(),
                name: "Custom".into(),
                instructions: Some("You are a custom narrator.".into()),
                ..Default::default()
            },
        ],
        quantifier_presets: vec![],
        active_system_id: "custom-1".into(),
        active_quantifier_id: "default".into(),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Default"));
    assert!(html.contains("Custom"));
    assert!(html.contains("System Prompts"));
    assert!(html.contains("Quantifier Prompts"));
}

#[test]
fn test_prompt_presets_template_shows_active_badge() {
    let template = PromptPresetsTemplate {
        system_presets: vec![PromptPreset {
            id: "custom-1".into(),
            name: "Custom".into(),
            instructions: Some("Custom prompt.".into()),
            ..Default::default()
        }],
        quantifier_presets: vec![],
        active_system_id: "custom-1".into(),
        active_quantifier_id: "default".into(),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Active"));
    assert!(html.contains("custom-1"));
}

#[test]
fn test_prompt_presets_template_shows_default_badge() {
    let template = PromptPresetsTemplate {
        system_presets: vec![PromptPreset {
            id: "default".into(),
            name: "Default".into(),
            instructions: Some("Default prompt.".into()),
            is_default: true,
            preset_type: PresetType::System,
            ..Default::default()
        }],
        quantifier_presets: vec![],
        active_system_id: "other".into(),
        active_quantifier_id: "default".into(),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Default"));
}

#[test]
fn test_prompt_presets_template_has_add_forms() {
    let template = PromptPresetsTemplate {
        system_presets: vec![],
        quantifier_presets: vec![],
        active_system_id: "default".into(),
        active_quantifier_id: "default".into(),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Add System Prompt Preset"));
    assert!(html.contains("Add Quantifier Prompt Preset"));
    assert!(html.contains(r#"name="preset_type" value="system""#));
    assert!(html.contains(r#"name="preset_type" value="quantifier""#));
}

#[test]
fn test_prompt_presets_template_shows_full_preview() {
    let long_text = "a".repeat(200);
    let template = PromptPresetsTemplate {
        system_presets: vec![PromptPreset {
            id: "test".into(),
            name: "Test".into(),
            instructions: Some(long_text.clone()),
            ..Default::default()
        }],
        quantifier_presets: vec![],
        active_system_id: "default".into(),
        active_quantifier_id: "default".into(),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Test"));
    assert!(html.contains(&"a".repeat(150)));
}
