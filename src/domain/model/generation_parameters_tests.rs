use serde_json::json;

use crate::domain::model::generation_parameters::{CallSettings, GenerationParameters};
use crate::domain::model::settings::DEFAULT_MAX_TOKENS;

#[test]
fn parameters_default_to_nothing_set() {
    let parameters = GenerationParameters::default();

    assert_eq!(parameters.max_tokens, None);
    assert_eq!(parameters.temperature, None);
    assert_eq!(parameters.top_p, None);
    assert!(parameters.custom.is_empty());
}

#[test]
fn parameters_read_from_a_json_object_with_missing_fields() {
    let parameters: GenerationParameters =
        serde_json::from_value(json!({"temperature": 0.7, "custom": {"top_k": 40}})).unwrap();

    assert_eq!(parameters.temperature, Some(0.7));
    assert_eq!(parameters.max_tokens, None);
    assert_eq!(parameters.custom["top_k"], json!(40));
}

#[test]
fn parameters_roundtrip_through_json() {
    let parameters = GenerationParameters {
        max_tokens: Some(900),
        temperature: Some(1.1),
        top_p: Some(0.9),
        custom: json!({"min_p": 0.05}).as_object().unwrap().clone(),
    };

    let text = serde_json::to_string(&parameters).unwrap();

    assert_eq!(
        serde_json::from_str::<GenerationParameters>(&text).unwrap(),
        parameters
    );
}

#[test]
fn refused_custom_key_names_each_key_the_engine_owns() {
    for key in ["model", "messages", "stream", "max_tokens"] {
        let mut parameters = GenerationParameters::default();
        parameters.custom.insert(key.to_string(), json!(1));

        assert_eq!(parameters.refused_custom_key(), Some(key), "key {key}");
    }
}

#[test]
fn refused_custom_key_is_none_for_other_keys() {
    let mut parameters = GenerationParameters::default();
    parameters.custom.insert("top_k".to_string(), json!(40));
    parameters.custom.insert("min_p".to_string(), json!(0.05));

    assert_eq!(parameters.refused_custom_key(), None);
}

#[test]
fn call_settings_default_sends_nothing() {
    let settings = CallSettings::default();

    assert_eq!(settings.max_tokens, DEFAULT_MAX_TOKENS);
    assert_eq!(settings.temperature, None);
    assert_eq!(settings.top_p, None);
    assert!(settings.custom.is_empty());
    assert_eq!(settings.game, None);
}

#[test]
fn call_settings_carry_the_parameters_with_the_resolved_ceiling_and_game() {
    use crate::domain::model::game::GameId;

    let parameters = GenerationParameters {
        max_tokens: Some(900),
        temperature: Some(0.5),
        top_p: Some(0.8),
        custom: json!({"top_k": 20}).as_object().unwrap().clone(),
    };

    let settings = CallSettings::of(&parameters, 512, GameId(7));

    assert_eq!(settings.max_tokens, 512);
    assert_eq!(settings.temperature, Some(0.5));
    assert_eq!(settings.top_p, Some(0.8));
    assert_eq!(settings.custom["top_k"], json!(20));
    assert_eq!(settings.game, Some(GameId(7)));
}
