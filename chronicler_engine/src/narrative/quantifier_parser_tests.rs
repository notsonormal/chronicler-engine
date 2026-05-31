//! Tests for the quantifier response parser.
//!
//! These tests verify that the parser correctly handles the exact JSON structures
//! returned by the quantifier LLM, including edge cases that previously caused
//! false positive "Low confidence" messages.
use crate::narrative::agents::quantifier::parser::parse_quantifier_response_with_movement;
use crate::model::quantifier::{MovementType, QuantifierConfidence};

#[test]
fn test_parse_valid_json_with_null_movement() {
    // Exact JSON from database that was causing false positive
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#;
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response_with_movement(response, &known_ids, &[]);

    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert!(result.movement.movement_type.is_none());
}

#[test]
fn test_parse_valid_json_with_entering_movement() {
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "entering", "destination": "mansion_drive"}}"#;
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response_with_movement(response, &known_ids, &[]);

    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(
        result.movement.destination,
        Some("mansion_drive".to_string())
    );
}

#[test]
fn test_parse_json_code_fence() {
    // Response with markdown code fence
    let response = r#"```json
{
"npcs_in_room": [
"carla"
],
"movement": {
"type": null
}
}
```"#;
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response_with_movement(response, &known_ids, &[]);

    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
}
