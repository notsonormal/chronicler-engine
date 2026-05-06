use crate::narrative::quantifier::parser::{
    compute_npc_events, extract_movement_from_text, parse_quantifier_response,
    parse_quantifier_response_with_movement,
};
use crate::narrative::quantifier::types::{
    MovementType, NpcEventType, QuantifierConfidence, RoomInfo,
};

#[test]
fn test_parse_json_response() {
    let response = r#"{"npcs_in_room": ["carla", "gabriella"]}"#;
    let known_ids = vec![
        "carla".to_string(),
        "gabriella".to_string(),
        "guard".to_string(),
    ];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_json_with_markdown_code_fence() {
    let response = "```json\n{\"npcs_in_room\": [\"carla\"]}\n```";
    let known_ids = vec!["carla".to_string(), "gabriella".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_json_embedded_in_text() {
    let response = "Based on the context, the NPCs present are:\n{\"npcs_in_room\": [\"carla\"]}\nThat's who I see.";
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_text_fallback() {
    let response = "Both Carla and Gabriella are in the room.";
    let known_ids = vec![
        "carla".to_string(),
        "gabriella".to_string(),
        "guard".to_string(),
    ];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_parse_filters_unknown_npcs() {
    let response = r#"{"npcs_in_room": ["carla", "harry", "gabriella"]}"#;
    let known_ids = vec!["carla".to_string(), "gabriella".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_empty_response() {
    let response = r#"{"npcs_in_room": []}"#;
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert!(result.npc_ids.is_empty());
    assert_eq!(result.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_completely_invalid_response() {
    let response = "I don't understand the question.";
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert!(result.npc_ids.is_empty());
    assert_eq!(result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_parse_text_fallback_case_insensitive() {
    let response = "Carla follows the player into the hall.";
    let known_ids = vec!["carla".to_string()];

    let result = parse_quantifier_response(response, &known_ids);
    assert_eq!(result.npc_ids, vec!["carla"]);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_parse_json_empty_array() {
    let response = r#"{"npcs_in_room": []}"#;
    let known_ids: Vec<String> = vec![];

    let result = parse_quantifier_response(response, &known_ids);
    assert!(result.npc_ids.is_empty());
}

#[test]
fn test_quantifier_confidence_levels() {
    let high_result =
        parse_quantifier_response(r#"{"npcs_in_room": ["carla"]}"#, &["carla".to_string()]);
    assert_eq!(high_result.confidence, QuantifierConfidence::High);

    let medium_result = parse_quantifier_response("Carla is here.", &["carla".to_string()]);
    assert_eq!(medium_result.confidence, QuantifierConfidence::Medium);

    let low_result = parse_quantifier_response("Random text.", &["carla".to_string()]);
    assert_eq!(low_result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_parse_quantifier_response_no_movement() {
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, None);
}

#[test]
fn test_parse_movement_entering_with_destination() {
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "entering", "destination": "entrance_hall"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "entrance_hall".to_string(),
            name: "Entrance Hall".to_string(),
        }],
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(
        result.movement.destination,
        Some("entrance_hall".to_string())
    );
}

#[test]
fn test_parse_movement_leaving_with_destination() {
    let response =
        r#"{"npcs_in_room": ["carla"], "movement": {"type": "leaving", "destination": "tavern"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "tavern".to_string(),
            name: "Tavern".to_string(),
        }],
    );

    assert_eq!(result.movement.movement_type, Some(MovementType::Leaving));
    assert_eq!(result.movement.destination, Some("tavern".to_string()));
}

#[test]
fn test_parse_movement_no_movement_field() {
    let response = r#"{"npcs_in_room": ["carla"]}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, None);
}

#[test]
fn test_parse_movement_empty_movement_object() {
    let response = r#"{"npcs_in_room": ["carla"], "movement": {}}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, None);
}

#[test]
fn test_parse_movement_unknown_type_becomes_none() {
    let response = r#"{"npcs_in_room": ["carla"], "movement": {"type": "teleporting", "destination": "castle"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "castle".to_string(),
            name: "Castle".to_string(),
        }],
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, Some("castle".to_string()));
}

#[test]
fn test_parse_movement_case_insensitive_type() {
    let response =
        r#"{"npcs_in_room": ["carla"], "movement": {"type": "ENTERING", "destination": "hall"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string()],
        &[RoomInfo {
            id: "hall".to_string(),
            name: "Hall".to_string(),
        }],
    );

    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
}

#[test]
fn test_parse_movement_text_fallback_no_movement() {
    let response = "carla is standing in the room.";
    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Medium);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_parse_movement_json_invalid_npcs_filtered() {
    let response = r#"{"npcs_in_room": ["carla", "unknown_npc", "gabriella"], "movement": {"type": "entering"}}"#;

    let result = parse_quantifier_response_with_movement(
        response,
        &["carla".to_string(), "gabriella".to_string()],
        &[],
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
}

#[test]
fn test_parse_response_json_with_movement_null_type() {
    let response =
        r#"{"npcs_in_room": ["carla"], "movement": {"type": null, "destination": "hall"}}"#;

    let result = parse_quantifier_response_with_movement(response, &["carla".to_string()], &[]);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, None);
    assert_eq!(result.movement.destination, Some("hall".to_string()));
}

#[test]
fn test_compute_npc_events_empty_previous() {
    let previous: Vec<String> = vec![];
    let current = vec!["carla".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].npc_id, "carla");
    assert_eq!(result.events[0].event_type, NpcEventType::Entered);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_compute_npc_events_no_changes() {
    let previous = vec!["carla".to_string()];
    let current = vec!["carla".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert!(result.events.is_empty());
    assert_eq!(result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_compute_npc_events_npc_left() {
    let previous = vec!["carla".to_string()];
    let current: Vec<String> = vec![];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].npc_id, "carla");
    assert_eq!(result.events[0].event_type, NpcEventType::Left);
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_compute_npc_events_mixed() {
    let previous = vec!["carla".to_string(), "derek".to_string()];
    let current = vec!["carla".to_string(), "gabriella".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 2);

    let entered = result
        .events
        .iter()
        .find(|e| e.npc_id == "gabriella")
        .expect("Gabriella should be in events");
    assert_eq!(entered.event_type, NpcEventType::Entered);

    let left = result
        .events
        .iter()
        .find(|e| e.npc_id == "derek")
        .expect("Derek should be in events");
    assert_eq!(left.event_type, NpcEventType::Left);

    assert!(!result.events.iter().any(|e| e.npc_id == "carla"));
}

#[test]
fn test_compute_npc_events_multiple_entered() {
    let previous: Vec<String> = vec![];
    let current = vec!["carla".to_string(), "gabriella".to_string()];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .all(|e| e.event_type == NpcEventType::Entered)
    );
    assert!(result.events.iter().any(|e| e.npc_id == "carla"));
    assert!(result.events.iter().any(|e| e.npc_id == "gabriella"));
}

#[test]
fn test_compute_npc_events_both_empty() {
    let previous: Vec<String> = vec![];
    let current: Vec<String> = vec![];

    let result = compute_npc_events(&previous, &current);

    assert!(result.events.is_empty());
    assert_eq!(result.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_compute_npc_events_all_left() {
    let previous = vec!["carla".to_string(), "gabriella".to_string()];
    let current: Vec<String> = vec![];

    let result = compute_npc_events(&previous, &current);

    assert_eq!(result.events.len(), 2);
    assert!(
        result
            .events
            .iter()
            .all(|e| e.event_type == NpcEventType::Left)
    );
    assert_eq!(result.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_extract_movement_from_text_entering() {
    let rooms = vec![RoomInfo {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
    }];
    let result = extract_movement_from_text("I enter the kitchen", &rooms);
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_from_text_leaving() {
    let rooms = vec![RoomInfo {
        id: "hall".to_string(),
        name: "Hall".to_string(),
    }];
    let result = extract_movement_from_text("I leave the hall", &rooms);
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_from_text_walk_into() {
    let rooms = vec![RoomInfo {
        id: "garden".to_string(),
        name: "Garden".to_string(),
    }];
    let result = extract_movement_from_text("I walk into the garden", &rooms);
    assert!(result.is_some());
}

#[test]
fn test_extract_movement_from_text_head_to() {
    let rooms = vec![RoomInfo {
        id: "tower".to_string(),
        name: "Tower".to_string(),
    }];
    let result = extract_movement_from_text("I head to the tower", &rooms);
    assert!(result.is_some());
}

#[test]
fn test_extract_movement_destination_not_found() {
    let rooms = vec![RoomInfo {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
    }];
    let result = extract_movement_from_text("I go somewhere else", &rooms);
    assert!(result.map(|r| r.destination.is_none()).unwrap_or(true));
}

#[test]
fn test_extract_movement_from_text_empty() {
    let rooms = vec![];
    let result = extract_movement_from_text("", &rooms);
    assert!(result.is_none());
}

#[test]
fn test_extract_movement_case_insensitive() {
    let rooms = vec![RoomInfo {
        id: "FOREST".to_string(),
        name: "Forest".to_string(),
    }];
    let result = extract_movement_from_text("I ENTER the FOREST", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_go_out() {
    let rooms = vec![RoomInfo {
        id: "outside".to_string(),
        name: "Outside".to_string(),
    }];
    let result = extract_movement_from_text("I go out", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_walk_out() {
    let rooms = vec![RoomInfo {
        id: "garden".to_string(),
        name: "Garden".to_string(),
    }];
    let result = extract_movement_from_text("I walk out to the garden", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_head_out() {
    let rooms = vec![RoomInfo {
        id: "courtyard".to_string(),
        name: "Courtyard".to_string(),
    }];
    let result = extract_movement_from_text("I head out to the courtyard", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_exits() {
    let rooms = vec![RoomInfo {
        id: "hallway".to_string(),
        name: "Hallway".to_string(),
    }];
    let result = extract_movement_from_text("I exit to the hallway", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Leaving));
}

#[test]
fn test_extract_movement_travel_to() {
    let rooms = vec![RoomInfo {
        id: "village".to_string(),
        name: "Village".to_string(),
    }];
    let result = extract_movement_from_text("I travel to the village", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_go_into() {
    let rooms = vec![RoomInfo {
        id: "cave".to_string(),
        name: "Cave".to_string(),
    }];
    let result = extract_movement_from_text("I go into the cave", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().movement_type, Some(MovementType::Entering));
}

#[test]
fn test_extract_movement_destination_found() {
    let rooms = vec![RoomInfo {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
    }];
    let result = extract_movement_from_text("I walk to the kitchen", &rooms);
    assert!(result.is_some());
    assert_eq!(result.unwrap().destination, Some("Kitchen".to_string()));
}
