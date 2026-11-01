//! Tests for the Character transitions the Game rules derive from a Quantifier result.

use super::quantifier::{CharacterTransitions, CharacterTransitionType, QuantifierResult};
use crate::test_support::fixtures::TestCharacter;

#[test]
fn character_transitions_from_diff_empty_previous() {
    let previous: Vec<String> = vec![];
    let current = vec!["carla".to_string()];

    let result = CharacterTransitions::from_diff(&previous, &current);

    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].character_id, "carla");
    assert_eq!(
        result.events[0].event_type,
        CharacterTransitionType::Entered
    );
}

#[test]
fn character_transitions_from_diff_no_changes() {
    let previous = vec!["carla".to_string()];
    let current = vec!["carla".to_string()];

    let result = CharacterTransitions::from_diff(&previous, &current);

    assert!(result.events.is_empty());
}

#[test]
fn character_transitions_from_diff_character_left() {
    let previous = vec!["carla".to_string()];
    let current: Vec<String> = vec![];

    let result = CharacterTransitions::from_diff(&previous, &current);

    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].character_id, "carla");
    assert_eq!(result.events[0].event_type, CharacterTransitionType::Left);
}

#[test]
fn character_transitions_from_diff_mixed() {
    let previous = vec!["carla".to_string(), "derek".to_string()];
    let current = vec!["carla".to_string(), "gabriella".to_string()];

    let result = CharacterTransitions::from_diff(&previous, &current);

    assert_eq!(result.events.len(), 2);
    let entered = result
        .events
        .iter()
        .find(|e| e.character_id == "gabriella")
        .expect("Gabriella should be in events");
    assert_eq!(entered.event_type, CharacterTransitionType::Entered);
    let left = result
        .events
        .iter()
        .find(|e| e.character_id == "derek")
        .expect("Derek should be in events");
    assert_eq!(left.event_type, CharacterTransitionType::Left);

    assert!(!result.events.iter().any(|e| e.character_id == "carla"));
}

#[test]
fn character_transitions_from_diff_multiple_entered() {
    let previous: Vec<String> = vec![];
    let current = vec!["carla".to_string(), "gabriella".to_string()];

    let result = CharacterTransitions::from_diff(&previous, &current);

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .all(|e| e.event_type == CharacterTransitionType::Entered)
    );
    assert!(result.events.iter().any(|e| e.character_id == "carla"));
    assert!(result.events.iter().any(|e| e.character_id == "gabriella"));
}

#[test]
fn character_transitions_from_diff_both_empty() {
    let previous: Vec<String> = vec![];
    let current: Vec<String> = vec![];

    let result = CharacterTransitions::from_diff(&previous, &current);

    assert!(result.events.is_empty());
}

#[test]
fn character_transitions_from_diff_all_left() {
    let previous = vec!["carla".to_string(), "gabriella".to_string()];
    let current: Vec<String> = vec![];

    let result = CharacterTransitions::from_diff(&previous, &current);

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .all(|e| e.event_type == CharacterTransitionType::Left)
    );
}

#[test]
fn keeping_present_gives_the_ids_in_order_and_no_destination() {
    let present = vec![
        TestCharacter::named("carla", "Carla"),
        TestCharacter::named("anna", "Anna"),
    ];

    let result = QuantifierResult::keeping_present(&present);

    assert_eq!(
        result.present,
        vec!["carla".to_string(), "anna".to_string()]
    );
    assert_eq!(result.destination, None);
}

#[test]
fn keeping_present_with_no_one_present_gives_the_empty_result() {
    assert_eq!(
        QuantifierResult::keeping_present(&[]),
        QuantifierResult::default()
    );
}
