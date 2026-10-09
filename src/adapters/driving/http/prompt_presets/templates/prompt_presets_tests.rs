use askama::Template;

use crate::adapters::driving::http::prompt_presets::templates::PromptPresetsTemplate;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{AppSettings, NarratorMode};

fn settings_with_active_system(novel: &str, interactive_fiction: &str) -> AppSettings {
    let mut settings = AppSettings::default();
    settings.set_active_preset(PresetType::System, NarratorMode::Novel, novel.into());
    settings.set_active_preset(
        PresetType::System,
        NarratorMode::InteractiveFiction,
        interactive_fiction.into(),
    );
    settings
}

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
        impersonate_presets: vec![],
        options_presets: vec![],
        settings: settings_with_active_system("custom-1", "default"),
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
        impersonate_presets: vec![],
        options_presets: vec![],
        settings: settings_with_active_system("custom-1", "default"),
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
        impersonate_presets: vec![],
        options_presets: vec![],
        settings: settings_with_active_system("other", "other"),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Default"));
}

#[test]
fn test_prompt_presets_template_has_add_forms() {
    let template = PromptPresetsTemplate {
        system_presets: vec![],
        quantifier_presets: vec![],
        impersonate_presets: vec![],
        options_presets: vec![],
        settings: AppSettings::default(),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Add System Prompt Preset"));
    assert!(html.contains("Add Quantifier Prompt Preset"));
    assert!(html.contains(r#"name="preset_type" value="system""#));
    assert!(html.contains(r#"name="preset_type" value="quantifier""#));
}

#[test]
fn test_prompt_presets_template_truncates_preview() {
    let long_text = "a".repeat(200);
    let template = PromptPresetsTemplate {
        system_presets: vec![PromptPreset {
            id: "test".into(),
            name: "Test".into(),
            instructions: Some(long_text.clone()),
            ..Default::default()
        }],
        quantifier_presets: vec![],
        impersonate_presets: vec![],
        options_presets: vec![],
        settings: AppSettings::default(),
    };

    let html = template.render().unwrap();
    assert!(html.contains("Test"));
    // The shared card truncates at 120 chars, so the panel matches an
    // edit-refreshed card instead of showing the full preview.
    assert!(html.contains(&"a".repeat(120)));
    assert!(!html.contains(&"a".repeat(121)));
}
