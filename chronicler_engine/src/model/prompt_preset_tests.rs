use crate::model::prompt_preset::{PresetType, PromptPreset};

#[test]
fn test_preset_type_system_as_str() {
    assert_eq!(PresetType::System.as_str(), "system");
}

#[test]
fn test_preset_type_quantifier_as_str() {
    assert_eq!(PresetType::Quantifier.as_str(), "quantifier");
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
fn test_preset_type_try_from_unknown_errors() {
    assert!(PresetType::try_from("unknown").is_err());
    assert!(PresetType::try_from("").is_err());
}

#[test]
fn test_prompt_preset_serialization_roundtrip() {
    let preset = PromptPreset {
        id: "test-id".into(),
        name: "Test Name".into(),
        prompt_text: "You are a narrator.".into(),
        is_default: true,
        preset_type: PresetType::System,
    };

    let json = serde_json::to_string(&preset).unwrap();
    let deserialized: PromptPreset = serde_json::from_str(&json).unwrap();

    assert_eq!(preset, deserialized);
}

#[test]
fn test_prompt_preset_deserialization_from_json() {
    let json = r#"{"id":"preset-1","name":"Default","prompt_text":"System prompt.","is_default":true,"preset_type":"System"}"#;
    let preset: PromptPreset = serde_json::from_str(json).unwrap();

    assert_eq!(preset.id, "preset-1");
    assert_eq!(preset.name, "Default");
    assert_eq!(preset.prompt_text, "System prompt.");
    assert!(preset.is_default);
}
