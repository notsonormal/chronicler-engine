use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::utils::settings_defaults;

#[test]
fn test_preset_type_system_as_str() {
    assert_eq!(PresetType::System.as_str(), "system");
}

#[test]
fn test_preset_type_quantifier_as_str() {
    assert_eq!(PresetType::Quantifier.as_str(), "quantifier");
}

#[test]
fn test_preset_type_impersonate_as_str() {
    assert_eq!(PresetType::Impersonate.as_str(), "impersonate");
}

#[test]
fn test_preset_type_try_from_quantifier() {
    assert_eq!(
        PresetType::try_from("quantifier").unwrap(),
        PresetType::Quantifier
    );
}

#[test]
fn test_preset_type_try_from_system() {
    assert_eq!(PresetType::try_from("system").unwrap(), PresetType::System);
}

#[test]
fn test_preset_type_try_from_impersonate() {
    assert_eq!(
        PresetType::try_from("impersonate").unwrap(),
        PresetType::Impersonate
    );
}

#[test]
fn test_preset_type_try_from_unknown_errors() {
    assert!(PresetType::try_from("unknown").is_err());
    assert!(PresetType::try_from("").is_err());
}

#[test]
fn test_prompt_preset_serialization_roundtrip() {
    let preset = PromptPreset {
        id: "test-id".into(),
        name: "Test Name".into(),
        role: Some("You are a narrator.".into()),
        instructions: Some("Be descriptive.".into()),
        is_default: true,
        preset_type: PresetType::System,
        ..Default::default()
    };

    let json = serde_json::to_string(&preset).unwrap();
    let deserialized: PromptPreset = serde_json::from_str(&json).unwrap();

    assert_eq!(preset, deserialized);
}

#[test]
fn test_prompt_preset_impersonate_serialization_roundtrip() {
    let preset = PromptPreset {
        id: "impersonate-id".into(),
        name: "Impersonate".into(),
        role: Some("Write as {{user}}.".into()),
        instructions: Some("{{persona_description}}".into()),
        is_default: true,
        preset_type: PresetType::Impersonate,
        ..Default::default()
    };

    let json = serde_json::to_string(&preset).unwrap();
    let deserialized: PromptPreset = serde_json::from_str(&json).unwrap();

    assert_eq!(preset, deserialized);
    assert_eq!(deserialized.preset_type, PresetType::Impersonate);
}

#[test]
fn test_prompt_preset_deserialization_from_json() {
    let json = r#"{"id":"preset-1","name":"Default","role":"System prompt.","instructions":"Be descriptive.","is_default":true,"preset_type":"System"}"#;
    let preset: PromptPreset = serde_json::from_str(json).unwrap();

    assert_eq!(preset.id, "preset-1");
    assert_eq!(preset.name, "Default");
    assert_eq!(preset.role, Some("System prompt.".into()));
    assert_eq!(preset.instructions, Some("Be descriptive.".into()));
    assert!(preset.is_default);
}

#[test]
fn test_assemble_prompt_text_with_all_sections() {
    let preset = PromptPreset {
        id: "test".into(),
        name: "Test".into(),
        role: Some("You are a narrator.".into()),
        instructions: Some("Be descriptive.".into()),
        writing_style: Some("Past tense.".into()),
        output_format: Some("No GPTisms.".into()),
        ..Default::default()
    };

    let result = preset.assemble_text(&["Rule 1".into()], Some("Keep it short."), None);

    assert!(result.contains("<role>"));
    assert!(result.contains("You are a narrator."));
    assert!(result.contains("<instructions>"));
    assert!(result.contains("Be descriptive."));
    assert!(result.contains("<writing_style>"));
    assert!(result.contains("Past tense."));
    assert!(result.contains("<global_rules>"));
    assert!(result.contains("- Rule 1"));
    assert!(result.contains("<output_format>"));
    assert!(result.contains("No GPTisms."));
    assert!(result.contains("Response Length:"));
    assert!(result.contains("Keep it short."));
}

#[test]
fn test_assemble_prompt_text_skips_empty_sections() {
    let preset = PromptPreset {
        id: "test".into(),
        name: "Test".into(),
        role: Some("Role text.".into()),
        output_format: Some("Output text.".into()),
        ..Default::default()
    };

    let result = preset.assemble_text(&[], None, None);

    assert!(result.contains("<role>"));
    assert!(!result.contains("<instructions>"));
    assert!(!result.contains("<writing_style>"));
    assert!(result.contains("<output_format>"));
    assert!(!result.contains("Response Length:"));
}

#[test]
fn test_assemble_prompt_text_order() {
    let preset = PromptPreset {
        id: "test".into(),
        name: "Test".into(),
        role: Some("ROLE".into()),
        instructions: Some("INSTRUCTIONS".into()),
        writing_style: Some("STYLE".into()),
        output_format: Some("OUTPUT".into()),
        ..Default::default()
    };

    let result = preset.assemble_text(&["RULE".into()], None, None);

    let role_pos = result.find("ROLE").unwrap();
    let inst_pos = result.find("INSTRUCTIONS").unwrap();
    let style_pos = result.find("STYLE").unwrap();
    let rules_pos = result.find("RULE").unwrap();
    let out_pos = result.find("OUTPUT").unwrap();

    assert!(role_pos < inst_pos);
    assert!(inst_pos < style_pos);
    assert!(style_pos < rules_pos);
    assert!(rules_pos < out_pos);
}

fn preset_with_modes(modes: Vec<crate::domain::model::settings::NarratorMode>) -> PromptPreset {
    PromptPreset {
        allowed_modes: modes,
        ..Default::default()
    }
}

#[test]
fn allows_true_only_for_listed_modes() {
    use crate::domain::model::settings::NarratorMode;

    let all = preset_with_modes(settings_defaults::default_allowed_modes());
    assert!(all.allows(NarratorMode::Novel));
    assert!(all.allows(NarratorMode::InteractiveFiction));

    let novel_only = preset_with_modes(vec![NarratorMode::Novel]);
    assert!(novel_only.allows(NarratorMode::Novel));
    assert!(!novel_only.allows(NarratorMode::InteractiveFiction));

    let none = preset_with_modes(Vec::new());
    assert!(!none.allows(NarratorMode::Novel));
    assert!(!none.allows(NarratorMode::InteractiveFiction));
}

#[test]
fn serde_missing_allowed_modes_defaults_to_all_modes() {
    use crate::domain::model::settings::NarratorMode;

    let json = r#"{"id":"p","name":"P","is_default":false,"preset_type":"System"}"#;
    let preset: PromptPreset = serde_json::from_str(json).unwrap();
    assert_eq!(
        preset.allowed_modes,
        vec![NarratorMode::Novel, NarratorMode::InteractiveFiction]
    );
}

#[test]
fn preset_type_marks_options_as_mode_agnostic() {
    assert!(PresetType::System.is_mode_tagged());
    assert!(PresetType::Quantifier.is_mode_tagged());
    assert!(PresetType::Impersonate.is_mode_tagged());
    assert!(!PresetType::Options.is_mode_tagged());
}
