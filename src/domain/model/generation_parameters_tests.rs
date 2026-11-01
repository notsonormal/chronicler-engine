use serde_json::json;

use crate::domain::model::generation_parameters::{CallSettings, GenerationParameters};

#[test]
fn parameters_read_from_a_json_object_with_missing_fields() {
    let parameters: GenerationParameters =
        serde_json::from_value(json!({"temperature": 0.7, "custom": {"top_k": 40}})).unwrap();

    assert_eq!(parameters.temperature, Some(0.7));
    assert_eq!(parameters.max_tokens, None);
    assert_eq!(parameters.custom["top_k"], json!(40));
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
